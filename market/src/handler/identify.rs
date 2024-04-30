use libp2p::{identify::Event, Swarm};
use log::{error, info, warn};

use crate::{behaviour::Behaviour, bridge::KAD_PROTOCOL_NAME};

use super::EventHandler;

pub(crate) struct IdentifyHandler<'a> {
    swarm: &'a mut Swarm<Behaviour>,
}

impl<'a> IdentifyHandler<'a> {
    pub(crate) fn new(swarm: &'a mut Swarm<Behaviour>) -> Self {
        IdentifyHandler { swarm }
    }
}

impl EventHandler for IdentifyHandler<'_> {
    type Event = Event;
    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::Received { peer_id, info } => {
                if info.protocols.contains(&KAD_PROTOCOL_NAME) {
                    info!(
                        "[Identify] - {peer_id} supports Kademlia. Adding addresses {:?}",
                        info.listen_addrs
                    );

                    for addr in info.listen_addrs {
                        self.swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                    }
                }
            }
            Event::Sent { peer_id } => {
                info!("[Identify] - Identify response sent back to {peer_id}");
            }
            Event::Error { peer_id, error } => {
                error!("[Identify] - Error occurred with {peer_id}: {error}");
            }
            Event::Pushed { peer_id, info } => {
                warn!("[Identify] - Automatically pushed identify information to {peer_id}");
                warn!("[Identify] - Information pushed: {info:?}");
            }
        }
    }
}
