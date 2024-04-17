use std::{thread, time::Duration};

use crate::{
    behaviour::Behaviour,
    bridge::{coordinator::Coordinator, peer::Peer},
    command::Message,
    Config,
};
use libp2p::{
    autonat, dcutr, identify,
    identity::{ed25519, Keypair},
    kad::{self, store::MemoryStore, NoKnownPeers},
    noise, ping, relay,
    swarm::behaviour::toggle::Toggle,
    tls, yamux, StreamProtocol, SwarmBuilder,
};
use thiserror::Error;
use tokio::{runtime::Runtime, sync::mpsc};

pub(crate) const IDENTIFY_PROTOCOL_VERSION: &str = "/orcanet/id/1.0.0";
pub(crate) const KAD_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/orcanet/kad/1.0.0");
pub(crate) const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60 * 10);

pub fn spawn(config: Config) -> Result<Peer, BridgeError> {
    let Config {
        peer_tcp_port,
        boot_nodes,
        coordinator_thread_name,
        file_ttl,
        public_address,
    } = config;

    // TODO: use the zeroize crate for zeroing memory after move of public/priv key
    let keypair = Keypair::from(ed25519::Keypair::generate());

    let swarm = SwarmBuilder::with_existing_identity(keypair.clone())
        .with_tokio()
        .with_tcp(
            Default::default(),
            (tls::Config::new, noise::Config::new),
            yamux::Config::default,
        )
        .map_err(|err| BridgeError::Tcp(err.to_string()))?
        .with_dns()
        .map_err(|err| BridgeError::Dns(err.to_string()))?
        .with_relay_client(
            (tls::Config::new, noise::Config::new),
            yamux::Config::default,
        )
        .map_err(|err| BridgeError::RelayClient(err.to_string()))?
        .with_behaviour(|key, relay_client| {
            let peer_id = key.public().to_peer_id();
            let kad = {
                let mut kad_config = kad::Config::default();
                kad_config
                    .set_protocol_names(vec![KAD_PROTOCOL_NAME])
                    .set_provider_record_ttl(Some(file_ttl));
                kad::Behaviour::with_config(peer_id, MemoryStore::new(peer_id), kad_config)
            };
            let identify = {
                let config =
                    identify::Config::new(IDENTIFY_PROTOCOL_VERSION.to_owned(), key.public());
                identify::Behaviour::new(config)
            };
            let ping = {
                let config = ping::Config::new();
                ping::Behaviour::new(config)
            };
            let autonat = {
                let config = autonat::Config {
                    boot_delay: Duration::from_secs(3),
                    ..Default::default()
                };
                autonat::Behaviour::new(peer_id, config)
            };

            let relay_server = {
                let config = relay::Config::default();
                let relay_server = relay::Behaviour::new(peer_id, config);
                Toggle::from(Some(relay_server))
            };
            let relay_client = Toggle::from(Some(relay_client));
            let dcutr = Toggle::from(Some(dcutr::Behaviour::new(peer_id)));
            Behaviour {
                kad,
                identify,
                ping,
                autonat,
                relay_client,
                relay_server,
                dcutr,
            }
        })
        .map_err(|_| BridgeError::Behaviour)?
        .with_swarm_config(|config| config.with_idle_connection_timeout(TIMEOUT))
        .build();
    let (command_sender, command_receiver) = mpsc::unbounded_channel::<Message>();
    let (peer_init_tx, peer_init_rx) = std::sync::mpsc::channel::<anyhow::Result<Peer>>();
    thread::Builder::new()
        .name(coordinator_thread_name)
        .spawn(move || {
            // TODO: maybe allow in future allow user to pass in # of worker threads they want to use
            // here
            Runtime::new().unwrap().block_on(async move {
                let peer_id = *swarm.local_peer_id();
                let maybe_coordinator = Coordinator::new(
                    swarm,
                    public_address,
                    boot_nodes,
                    peer_tcp_port,
                    command_receiver,
                );
                match maybe_coordinator {
                    Ok(coordinator) => {
                        peer_init_tx
                            .send(Ok(Peer::new(peer_id, command_sender, keypair)))
                            .expect("send to succeed");
                        drop(peer_init_tx);
                        coordinator.run().await;
                    }
                    Err(err) => {
                        peer_init_tx.send(Err(err)).expect("send to succeed");
                    }
                }
            });
        })
        .expect("thread to spawn");
    peer_init_rx
        .recv()
        .expect("to receive some kind of response for initializing a peer")
        .map_err(|err| BridgeError::PeerInitializationFailed(err.to_string()))
    // let (boot_tx, boot_rx) = mpsc::channel();
    // thread::Builder::new()
    //     .name(coordinator_thread_name)
    //     .spawn(move || {
    //         Runtime::new().unwrap().block_on(async move {
    //             if let Err(err) = swarm
    //                 .listen_on(
    //                     Multiaddr::empty()
    //                         .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
    //                         .with(Protocol::Tcp(peer_tcp_port)),
    //                 )
    //                 .map_err(|err| BridgeError::InitialListen(err.to_string()))
    //             {
    //                 boot_tx.send(Err(err)).expect("send to succeed");
    //                 drop(boot_tx);
    //                 return;
    //             }

    //             if let Some(boot_nodes) = &boot_nodes {
    //                 let kad_boot_nodes = boot_nodes.get_kad_addrs();
    //                 for (peer_id, ip) in kad_boot_nodes {
    //                     swarm.behaviour_mut().kad.add_address(&peer_id, ip.clone());
    //                     // NOTE: seems like autonat is auto adding servers in?
    //                     // swarm.behaviour_mut().autonat.add_server(peer_id, Some(ip));
    //                 }
    //                 swarm
    //                     .behaviour_mut()
    //                     .kad
    //                     .bootstrap()
    //                     .expect("bootstrap to succeed");

    //                 if let Err(err) = time::timeout(BOOTING_TIMEOUT, async {
    //                     let booting = Cell::new(true);
    //                     while booting.get() {
    //                         let event = swarm.select_next_some().await;
    //                         let mut handler =
    //                             handler::bootup::BootupHandler::new(&mut swarm, &booting);
    //                         handler.handle_event(event);
    //                     }
    //                 })
    //                 .await
    //                 .map_err(|err| BridgeError::Booting(err.to_string()))
    //                 {
    //                     boot_tx.send(Err(err)).expect("send to succeed");
    //                     return;
    //                 }

    //                 match swarm.behaviour_mut().autonat.nat_status() {
    //                     autonat::NatStatus::Public(addr) => {
    //                         info!("{addr:?}");
    //                     }
    //                     autonat::NatStatus::Private => {
    //                         for node in boot_nodes.iter() {
    //                             let addr = node.clone().with(Protocol::P2pCircuit);
    //                             println!("{:?}", addr);
    //                             warn!("{:?}", swarm.listen_on(addr));
    //                         }
    //                     }
    //                     autonat::NatStatus::Unknown => todo!(),
    //                 }
    //                 let mut interval = time::interval(Duration::from_secs(30));
    //                 loop {
    //                     select! {
    //                         _ = interval.tick() => {
    //                             swarm.behaviour_mut().kad.bootstrap().expect("bootstrap to succeed");
    //                         }
    //                         event = swarm.select_next_some() => {
    //                             if !matches!(
    //                                 event,
    //                                 SwarmEvent::Behaviour(BehaviourEvent::Kad(
    //                                     kad::Event::OutboundQueryProgressed {
    //                                         result: QueryResult::Bootstrap(_),
    //                                         ..
    //                                     },
    //                                 ))
    //                             ) {
    //                                 info!("{:?}", event);
    //                             }
    //                             match event {
    //                                 SwarmEvent::Behaviour(BehaviourEvent::Identify(Event::Received {
    //                                     peer_id,
    //                                     info
    //                                 })) => {

    //                                     if info.protocols.contains(&KAD_PROTOCOL_NAME) {
    //                                         for addr in info.listen_addrs {
    //                                             swarm.behaviour_mut().kad.add_address(&peer_id, addr);
    //                                         }
    //                                     }
    //                                 }
    //                                 SwarmEvent::Behaviour(BehaviourEvent::Dcutr(_)) => {
    //                                     todo!()
    //                                 }
    //                                 _ => {}
    //                             }
    //                             // if let SwarmEvent::Behaviour(BehaviourEvent::Identify(Event::Received {
    //                             //     peer_id,
    //                             //     info,
    //                             // })) = event
    //                             // {
    //                             //     if info.protocols.contains(&KAD_PROTOCOL_NAME) {
    //                             //         for addr in info.listen_addrs {
    //                             //             swarm.behaviour_mut().kad.add_address(&peer_id, addr);
    //                             //         }
    //                             //     }
    //                             // }
    //                         }
    //                     }
    //                 }
    //                 boot_tx.send(Ok(())).expect("send to succeed");
    //                 tokio::time::sleep(Duration::from_secs(60000000)).await;
    //             } else {
    //                 boot_tx.send(Ok(())).expect("send to succeed");

    //                 // TODO: this will be assumed to be public since this means it's a genesis node
    //                 // since it has no boot nodes
    //             }
    //         });
    //     })
    //     .expect("thread to spawn");

    // boot_rx.recv().expect("recv to succeed")?;
    // thread::sleep(Duration::from_secs(3000000));
}

#[derive(Debug, Clone, Error)]
pub enum BridgeError {
    #[error("TCP failed to initialize: {0}")]
    Tcp(String),
    #[error("DNS failed to initialize: {0}")]
    Dns(String),
    #[error("Relay client failed to initialize: {0}")]
    RelayClient(String),
    #[error("Behaviour failed to initialize!")]
    Behaviour,
    #[error("Initial bootstrap failed {0}!")]
    InitialBootstrap(#[from] NoKnownPeers),
    #[error("Initial Listening failed {0}!")]
    InitialListen(String),
    #[error("Booting failed {0}!")]
    Booting(String),
    #[error("Peer initialization failed!")]
    PeerInitializationFailed(String),
}

mod coordinator;
pub mod peer;
