use libp2p::{
    identify::Behaviour as IdentifyBehaviour, kad::Behaviour as KadBehaviour,
    swarm::NetworkBehaviour,
};

use self::{
    ident::Identify,
    kademlia::{BootstrapMode, Kad, KadError, KadStore},
};

#[derive(NetworkBehaviour)]
#[allow(missing_debug_implementations)]
#[non_exhaustive] // NOTE: maybe more protocols?
pub(crate) struct MarketBehaviour<TKadStore: KadStore> {
    kademlia: Kad<TKadStore>,
    identify: Identify,
}

impl<TKadStore: KadStore> MarketBehaviour<TKadStore> {
    pub(crate) const fn new(
        kademlia: KadBehaviour<TKadStore>,
        identify: IdentifyBehaviour,
    ) -> Self {
        Self {
            kademlia: Kad::new(kademlia),
            identify: Identify::new(identify),
        }
    }

    pub(crate) fn handle_event(&mut self, event: MarketBehaviourEvent<TKadStore>) {
        match event {
            MarketBehaviourEvent::Kademlia(event) => {
                self.kademlia.handle_kad_event(event);
            }
            MarketBehaviourEvent::Identify(event) => self
                .identify
                .handle_identify_event(event, &mut self.kademlia),
        }
    }

    pub(crate) fn bootstrap(&mut self, mode: BootstrapMode) -> Result<(), KadError> {
        self.kademlia.bootstrap(mode)
    }
}

pub(crate) mod ident;
pub(crate) mod kademlia;
