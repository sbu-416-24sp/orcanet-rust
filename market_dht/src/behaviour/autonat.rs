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
                } => {
                    info!("[{probe_id:?}] - Inbound probe request to {peer} with {addresses:?}",);
                }
                InboundProbeEvent::Response {
                    probe_id,
                    peer,
                    address,
                } => {
                    info!("[{probe_id:?}] - Inbound probe response to {peer} with {address:?}",);
                }
                InboundProbeEvent::Error {
                    probe_id,
                    peer,
                    error,
                } => {
                    error!("[{probe_id:?}] - Error with inbound probe request from {peer:?}: {error:?}");
                }
            },
            Event::OutboundProbe(event) => match event {
                OutboundProbeEvent::Request { probe_id, peer } => {
                    info!("[{probe_id:?}] - Sent outbound probe request from {}", peer);
                }
                OutboundProbeEvent::Response {
                    probe_id,
                    peer,
                    address,
                } => {
                    info!(
                        "[{probe_id:?}] - Sent outbound probe response to {peer} with {address:?}",
                    );
                }
                OutboundProbeEvent::Error {
                    probe_id,
                    peer: Some(peer),
                    error,
                } => {
                    error!("[{probe_id:?}] - Error sending outbound probe to {peer:?}: {error:?}");
                }
                _ => {}
            },
            Event::StatusChanged { old, new } => {
                info!("Status changed from {:?} to {:?}", old, new);
            }
        }
    }
}
