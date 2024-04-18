use libp2p::{identify::Event, Swarm};

use crate::behaviour::Behaviour;

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
            Event::Received { peer_id, info } => todo!(),
            Event::Sent { peer_id } => todo!(),
            Event::Error { peer_id, error } => todo!(),
            Event::Pushed { peer_id, info } => todo!(),
        }
    }
}
