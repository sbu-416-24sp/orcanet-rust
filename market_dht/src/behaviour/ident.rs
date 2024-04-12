use libp2p::{
    identify::{self, Behaviour as IdentifyBehaviour},
    swarm::NetworkBehaviour,
};
use log::{error, info, warn};

use crate::behaviour::kademlia::KAD_PROTOCOL_NAME;

use super::kademlia::{Kad, KadStore};

pub(crate) const IDENTIFY_PROTOCOL_NAME: &str = "/orcanet/id/1.0.0";

#[derive(Debug, Default)]
pub(crate) struct IdentifyHandler {}

impl IdentifyHandler {
    pub(crate) fn handle_identify_event<TKadStore: KadStore>(
        &mut self,
        IdentifyEvent::Identify(event): IdentifyEvent,
        kademlia: &mut Kad<TKadStore>,
    ) {
        match event {
            identify::Event::Received {
                peer_id,
                info:
                    identify::Info {
                        listen_addrs,
                        protocols,
                        ..
                    },
            } => {
                warn!("Peer {peer_id} identified with listen addresses: {listen_addrs:?} and protocols: {protocols:?}");
                if protocols.iter().any(|proto| proto == &KAD_PROTOCOL_NAME) {
                    for addr in listen_addrs {
                        kademlia.kad_mut().add_address(&peer_id, addr);
                    }
                }
            }
            identify::Event::Sent { peer_id } => {
                info!("Sent an identify request to peer {}", peer_id)
            }
            identify::Event::Pushed { peer_id, info } => {
                warn!("Pushed identify info to peer {peer_id}: {info:?}")
            }
            identify::Event::Error { peer_id, error } => {
                error!("Error identifying peer {peer_id}: {error}")
            }
        }
    }
}

#[derive(NetworkBehaviour)]
pub(crate) struct Identify {
    identify: IdentifyBehaviour,
}

impl Identify {
    #[inline(always)]
    pub(crate) const fn new(identify: IdentifyBehaviour) -> Self {
        Self { identify }
    }
}
