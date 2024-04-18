use libp2p::dcutr::Event;

use super::EventHandler;

pub(crate) struct DcutrHandler;

impl EventHandler for DcutrHandler {
    type Event = Event;
    fn handle_event(
        &mut self,
        Event {
            remote_peer_id,
            result,
        }: Self::Event,
    ) {
        match result {
            Ok(conn_id) => todo!(),
            Err(err) => {
                todo!();
            }
        }
    }
}
