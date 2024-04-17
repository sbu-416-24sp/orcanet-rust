use libp2p::relay::Event;

use crate::handler::EventHandler;

pub(crate) struct RelayServerHandler;

impl EventHandler for RelayServerHandler {
    type Event = Event;
    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::ReservationReqAccepted {
                src_peer_id,
                renewed,
            } => todo!(),
            Event::ReservationReqDenied { src_peer_id } => todo!(),
            Event::ReservationTimedOut { src_peer_id } => todo!(),
            Event::CircuitReqDenied {
                src_peer_id,
                dst_peer_id,
            } => todo!(),
            Event::CircuitReqAccepted {
                src_peer_id,
                dst_peer_id,
            } => todo!(),
            Event::CircuitClosed {
                src_peer_id,
                dst_peer_id,
                error,
            } => todo!(),
            _ => {}
        }
    }
}
