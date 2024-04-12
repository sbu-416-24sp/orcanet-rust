use std::{thread, time::Duration};

use libp2p::{
    identify::{Behaviour as IdentifyBehaviour, Config as IdentifyConfig},
    kad::{store::MemoryStore, Behaviour as KadBehaviour, Config as KadConfig},
    noise, yamux,
};
use thiserror::Error;
use tokio::{runtime::Runtime, sync::mpsc};

use crate::{
    behaviour::{
        file_req_res::{FileReqResBehaviour, FILE_REQ_RES_PROTOCOL},
        ident::IDENTIFY_PROTOCOL_NAME,
        kademlia::KAD_PROTOCOL_NAME,
        MarketBehaviour,
    },
    config::Config,
    coordinator::Coordinator,
    peer::Peer,
};

const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(60 * 60);
pub(crate) const PROVIDER_RECORD_TTL: Duration = Duration::from_secs(60 * 60);
const PROVIDER_REPUBLICATION: Duration = Duration::from_secs(60 * 5);

pub fn spawn_bridge(config: Config) -> Result<Peer, NetworkBridgeError> {
    let swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            Default::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|err| NetworkBridgeError::Init(err.to_string()))?
        .with_dns()
        .map_err(|err| NetworkBridgeError::Init(err.to_string()))?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();
            // TODO: maybe configure something?
            let mut config = KadConfig::default();
            config.set_protocol_names(vec![KAD_PROTOCOL_NAME]);
            // NOTE: skeptical here?
            config.set_provider_publication_interval(Some(PROVIDER_REPUBLICATION));
            config.set_provider_record_ttl(Some(PROVIDER_RECORD_TTL));
            let kad_behaviour =
                KadBehaviour::with_config(peer_id, MemoryStore::new(peer_id), config);
            let config = IdentifyConfig::new(IDENTIFY_PROTOCOL_NAME.to_string(), key.public());
            let identify_behaviour = IdentifyBehaviour::new(config);
            let file_req_res = FileReqResBehaviour::new(FILE_REQ_RES_PROTOCOL, Default::default());
            MarketBehaviour::new(kad_behaviour, identify_behaviour, file_req_res)
        })
        .map_err(|err| NetworkBridgeError::Init(err.to_string()))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(KEEP_ALIVE_TIMEOUT))
        .build();

    // NOTE: this thread places the coordinator in a static context assuming the
    // thread lives for program life
    let Config {
        boot_nodes,
        listener,
        thread_name,
    } = config;
    let peer_id = *swarm.local_peer_id();

    let (receiver_tx, receiver_rx) = mpsc::unbounded_channel();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();

    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            Runtime::new().unwrap().block_on(async move {
                match Coordinator::new(swarm, listener, boot_nodes, receiver_rx) {
                    Ok(coordinator) => {
                        ready_tx
                            .send(Ok(()))
                            .expect("the receiver to still be alive");
                        drop(ready_tx);
                        coordinator.run().await;
                    }
                    Err(err) => {
                        ready_tx
                            .send(Err(err))
                            .expect("the receiver to still be alive");
                    }
                }
            });
        })
        .expect("it to spawn the network bridge thread");
    let res = ready_rx
        .recv()
        .map_err(|err| NetworkBridgeError::Init(err.to_string()))?;
    if let Err(err) = res {
        Err(NetworkBridgeError::Init(err.to_string()))
    } else {
        let peer = Peer::new(receiver_tx, peer_id);
        Ok(peer)
    }
}

#[derive(Debug, Error)]
pub enum NetworkBridgeError {
    #[error("Failed to initialize network bridge: {0}")]
    Init(String),
}
