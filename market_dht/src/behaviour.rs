use libp2p::{
    identify::Behaviour as IdentifyBehaviour,
    kad::{store::MemoryStore, Behaviour as KadBehaviour},
    swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[allow(missing_debug_implementations)]
#[non_exhaustive] // NOTE: maybe more protocols?
pub(crate) struct MarketBehaviour<TKadStore> {
    kademlia: KadBehaviour<TKadStore>,
    identify: IdentifyBehaviour,
}

impl<TKadStore> MarketBehaviour<TKadStore> {
    pub(crate) const fn new(
        kademlia: KadBehaviour<TKadStore>,
        identify: IdentifyBehaviour,
    ) -> Self {
        Self { kademlia, identify }
    }
}
