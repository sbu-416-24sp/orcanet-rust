use libp2p::autonat::{
    Behaviour as AutoNatBehaviour, Event, InboundProbeEvent, OutboundProbeEvent,
};
use log::{error, info};

#[derive(Debug, Default)]
pub(crate) struct AutoNatHandler;

impl AutoNatHandler {
    pub(crate) fn handle_event(&mut self, event: Event, autonat: &mut AutoNatBehaviour) {
        match event {
            Event::InboundProbe(event) => match event {
                InboundProbeEvent::Request {
                    probe_id,
                    peer,
                    addresses,
                } => todo!(),
                InboundProbeEvent::Response {
                    probe_id,
                    peer,
                    address,
                } => todo!(),
                InboundProbeEvent::Error {
                    probe_id,
                    peer,
                    error,
                } => {
                    error!("[{probe_id:?}] - Error with inbound probe from {peer:?}: {error:?}");
                }
            },
            Event::OutboundProbe(event) => {
                match event {
                    OutboundProbeEvent::Request { probe_id, peer } => {
                        info!(
                            "[{probe_id:?}] - Received outbound probe request from {}",
                            peer
                        );
                    }
                    OutboundProbeEvent::Response {
                        probe_id,
                        peer,
                        address,
                    } => todo!(),
                    OutboundProbeEvent::Error {
                        probe_id,
                        peer,
                        error,
                    } => {
                        if let Some(peer) = peer {
                            error!("[{probe_id:?}] - Error sending outbound probe to {peer:?}: {error:?}");
                        } else {
                            error!("[{probe_id:?}] - Error sending outbound probe: {error:?}");
                        }
                    }
                }
            }
            Event::StatusChanged { old, new } => todo!(),
        }
    }
}
