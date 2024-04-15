use libp2p::autonat::Event;

use super::EventHandler;

pub(crate) struct AutoNatHandler;

impl EventHandler for AutoNatHandler {
    type Event = Event;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::InboundProbe(_) => todo!(),
            Event::OutboundProbe(_) => todo!(),
            Event::StatusChanged { old, new } => todo!(),
        }
    }
}
