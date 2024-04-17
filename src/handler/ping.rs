use libp2p::ping::{Event, Failure};
use log::{error, info};

use super::EventHandler;

#[derive(Debug)]
pub(crate) struct PingHandler;

impl EventHandler for PingHandler {
    type Event = Event;

    fn handle_event(
        &mut self,
        Event {
            result,
            peer,
            connection,
        }: Self::Event,
    ) {
        match result {
            Ok(ms) => {
                info!(
                    "[ConnId: {connection}] Ping to peer {} succeeded in {:?}ms",
                    peer, ms
                );
            }
            Err(err) => match err {
                Failure::Timeout => {
                    error!("[ConnId: {connection}] Ping to peer {peer} timed out!")
                }
                Failure::Unsupported => {
                    error!("[ConnId: {connection}] Peer {peer} does not support the ping protocol!")
                }
                Failure::Other { error } => {
                    error!("[ConnId: {connection}] Ping to peer {peer} failed: {error}")
                }
            },
        }
    }
}
