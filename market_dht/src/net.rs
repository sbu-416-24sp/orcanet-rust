use std::thread;

use libp2p::{
    identify::{Behaviour as IdentifyBehaviour, Config as IdentifyConfig},
    kad::{store::MemoryStore, Behaviour as KadBehaviour, Config as KadConfig},
    noise, yamux,
};
use thiserror::Error;
use tokio::{runtime::Runtime, sync::oneshot};

use crate::{
    behaviour::{MarketBehaviour, IDENTIFY_PROTOCOL_NAME},
    config::Config,
    coordinator::Coordinator,
    peer::Peer,
};

const BRIDGE_THREAD_NAME: &str = "peer_command_coordinator_netbridge_thread";

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
            let config = KadConfig::default();
            let kad_behaviour =
                KadBehaviour::with_config(peer_id, MemoryStore::new(peer_id), config);
            let identify_behaviour = IdentifyBehaviour::new(IdentifyConfig::new(
                IDENTIFY_PROTOCOL_NAME.to_string(),
                key.public(),
            ));
            MarketBehaviour::new(kad_behaviour, identify_behaviour)
        })
        .map_err(|err| NetworkBridgeError::Init(err.to_string()))?
        .build();
    let (ready_tx, ready_rx) = oneshot::channel();
    thread::Builder::new()
        .name(BRIDGE_THREAD_NAME.to_string())
        .spawn(move || {
            Runtime::new().unwrap().block_on(async move {
                let coordinator = Coordinator::new(swarm, config);
                coordinator.run(ready_tx).await;
            });
        })
        .expect("it to spawn the network bridge thread");
    let peer = ready_rx.blocking_recv().unwrap();
    Ok(peer)
}

#[derive(Debug, Error)]
pub enum NetworkBridgeError {
    #[error("Failed to initialize network bridge: {0}")]
    Init(String),
}
