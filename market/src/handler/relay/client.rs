use libp2p::relay::client::Event;

use crate::handler::EventHandler;

pub(crate) struct RelayClientHandler {}

impl EventHandler for RelayClientHandler {
    type Event = Event;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::ReservationReqAccepted {
                relay_peer_id,
                renewal,
                limit,
            } => todo!(),
            Event::OutboundCircuitEstablished {
                relay_peer_id,
                limit,
            } => todo!(),
            Event::InboundCircuitEstablished { src_peer_id, limit } => todo!(),
        }
    }
}
