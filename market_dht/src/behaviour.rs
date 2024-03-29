use crate::{boot_nodes::BootNodes, Multiaddr, PeerId};
use libp2p::{
    identify::Behaviour as IdentifyBehaviour,
    kad::{
        store::{MemoryStore, RecordStore},
        Behaviour as KadBehaviour,
    },
    swarm::NetworkBehaviour,
};
use thiserror::Error;

pub(crate) const IDENTIFY_PROTOCOL_NAME: &str = "/ipfs/id/1.0.0";

pub trait KadStore: RecordStore + Send + Sync + 'static {}
impl KadStore for MemoryStore {}

#[derive(NetworkBehaviour)]
#[allow(missing_debug_implementations)]
#[non_exhaustive] // NOTE: maybe more protocols?
pub(crate) struct MarketBehaviour<TKadStore: KadStore> {
    pub(crate) kademlia: Kad<TKadStore>,
    pub(crate) identify: IdentifyBehaviour,
}

impl<TKadStore: KadStore> MarketBehaviour<TKadStore> {
    pub(crate) const fn new(
        kademlia: KadBehaviour<TKadStore>,
        identify: IdentifyBehaviour,
    ) -> Self {
        Self {
            kademlia: Kad::new(kademlia),
            identify,
        }
    }
}

#[derive(NetworkBehaviour)]
pub(crate) struct Kad<TStore> {
    kad: KadBehaviour<TStore>,
}

impl<TStore: KadStore> Kad<TStore> {
    pub(crate) const fn new(kad: KadBehaviour<TStore>) -> Self {
        Self { kad }
    }

    pub(crate) fn bootstrap(&mut self, boot_nodes: BootNodes) -> Result<(), KadError> {
        for node in boot_nodes {
            self.kad.add_address(&node.peer_id, node.addr);
        }
        let qid = self
            .kad
            .bootstrap()
            .map_err(|err| KadError::Bootstrap(err.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub(crate) enum KadError {
    #[error("Failed to bootstrap Kademlia: {0}")]
    Bootstrap(String),
}
