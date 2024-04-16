use std::{cell::Cell, net::Ipv4Addr, sync::mpsc, thread, time::Duration};

use crate::{
    behaviour::Behaviour,
    handler::{self, EventHandler},
    Config,
};
use futures::StreamExt;
use libp2p::{
    autonat, dcutr, identify,
    kad::{self, store::MemoryStore, NoKnownPeers},
    multiaddr::Protocol,
    noise, ping, relay,
    swarm::behaviour::toggle::Toggle,
    tls, yamux, Multiaddr, StreamProtocol, SwarmBuilder,
};
use log::{error, info};
use thiserror::Error;
use tokio::{runtime::Runtime, time};

pub(crate) const IDENTIFY_PROTOCOL_VERSION: &str = "/orcanet/id/1.0.0";
pub(crate) const KAD_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/orcanet/kad/1.0.0");
pub(crate) const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60 * 10);

pub(crate) const BOOTING_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(60);

pub fn spawn(config: Config) -> Result<(), BridgeError> {
    let Config {
        peer_tcp_port,
        boot_nodes,
        coordinator_thread_name,
        file_ttl,
    } = config;

    let mut swarm = SwarmBuilder::with_new_identity()
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
    let (boot_tx, boot_rx) = mpsc::channel();
    thread::Builder::new()
        .name(coordinator_thread_name)
        .spawn(move || {
            Runtime::new().unwrap().block_on(async move {
                if let Err(err) = swarm
                    .listen_on(
                        Multiaddr::empty()
                            .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
                            .with(Protocol::Tcp(peer_tcp_port)),
                    )
                    .map_err(|err| BridgeError::InitialListen(err.to_string()))
                {
                    boot_tx.send(Err(err)).expect("send to succeed");
                    drop(boot_tx);
                    return;
                }

                if let Some(boot_nodes) = &boot_nodes {
                    let boot_nodes = boot_nodes.get_kad_addrs();
                    for (peer_id, ip) in boot_nodes {
                        swarm.behaviour_mut().kad.add_address(&peer_id, ip);
                    }
                    swarm
                        .behaviour_mut()
                        .kad
                        .bootstrap()
                        .expect("bootstrap to succeed");

                    if let Err(err) = time::timeout(BOOTING_TIMEOUT, async {
                        let booting = Cell::new(true);
                        while booting.get() {
                            let event = swarm.select_next_some().await;
                            let mut handler =
                                handler::bootup::BootupHandler::new(&mut swarm, &booting);
                            handler.handle_event(event);
                        }
                    })
                    .await
                    .map_err(|err| BridgeError::Booting(err.to_string()))
                    {
                        boot_tx.send(Err(err)).expect("send to succeed");
                        return;
                    }
                }
            });
        })
        .expect("thread to spawn");

    boot_rx.recv().expect("recv to succeed")?;
    thread::sleep(Duration::from_secs(3000000));
    todo!()
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
}

mod coordinator;
pub mod peer;
