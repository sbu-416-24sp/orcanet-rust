use std::cell::Cell;

use crate::{
    behaviour::Behaviour,
    handler::{self, EventHandler},
    Config,
};
use futures::{executor::block_on, StreamExt};
use libp2p::{
    autonat, dcutr, identify,
    kad::{self, store::MemoryStore},
    noise, ping, relay,
    swarm::behaviour::toggle::Toggle,
    tls, yamux, StreamProtocol, SwarmBuilder,
};
use thiserror::Error;

pub(crate) const IDENTIFY_PROTOCOL_VERSION: &str = "/orcanet/id/1.0.0";
pub(crate) const KAD_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/orcanet/kad/1.0.0");
pub(crate) const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60 * 10);

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
                let config = autonat::Config::default();
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
    block_on(async {
        let booting = Cell::new(true);
        while booting.get() {
            let event = swarm.select_next_some().await;
            let mut handler = handler::BootupHandler::new(&mut swarm, &booting);
            handler.handle_event(event);
        }
    });
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
}

mod coordinator;
pub mod peer;
