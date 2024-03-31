use std::{thread, time::Duration};

use libp2p::{
    identify::{Behaviour as IdentifyBehaviour, Config as IdentifyConfig},
    kad::{store::MemoryStore, Behaviour as KadBehaviour, Config as KadConfig},
    noise, yamux,
};
use thiserror::Error;
use tokio::{runtime::Runtime, sync::oneshot};

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
const ONE_HOUR: Duration = Duration::from_secs(60 * 60);

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
            config.set_provider_publication_interval(None);
            config.set_provider_record_ttl(Some(ONE_HOUR));
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
    let (ready_tx, ready_rx) = oneshot::channel();
    // NOTE: this thread places the coordinator in a static context assuming the
    // thread lives for program life
    let Config {
        boot_nodes,
        listener,
        thread_name,
    } = config;
    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            Runtime::new().unwrap().block_on(async move {
                let coordinator = Coordinator::new(swarm, listener, boot_nodes);
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
