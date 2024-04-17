use std::cell::Cell;

use libp2p::{
    autonat,
    kad::{self, BootstrapError, BootstrapOk, QueryResult},
    swarm::SwarmEvent,
    Swarm,
};
use log::{error, info, warn};

use crate::behaviour::{Behaviour, BehaviourEvent};

use super::EventHandler;

pub(crate) struct BootupHandler<'a, 'b> {
    swarm: &'a mut Swarm<Behaviour>,
    booting: &'b Cell<bool>,
}

impl<'a, 'b> BootupHandler<'a, 'b> {
    pub(crate) fn new(swarm: &'a mut Swarm<Behaviour>, booting: &'b Cell<bool>) -> Self {
        BootupHandler { swarm, booting }
    }
}

impl<'a, 'b> EventHandler for BootupHandler<'a, 'b> {
    type Event = SwarmEvent<BehaviourEvent>;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            SwarmEvent::Behaviour(event) => {
                match event {
                    BehaviourEvent::Kad(event) => match event {
                        kad::Event::RoutingUpdated {
                            peer, addresses, ..
                        } => {
                            warn!("[Kad] Routing updated with {peer} with {addresses:?}");
                        }
                        kad::Event::OutboundQueryProgressed {
                            result: QueryResult::Bootstrap(bootstrap_sts),
                            ..
                        } => match bootstrap_sts {
                            Ok(BootstrapOk {
                                peer,
                                num_remaining,
                            }) => {
                                info!("[Kad] Bootstrap succeeded with {peer} and {num_remaining} remaining");
                            }
                            Err(BootstrapError::Timeout { peer, .. }) => {
                                error!("[Kad] Bootstrap timeout with {peer}");
                            }
                        },
                        kad::Event::ModeChanged { new_mode } => {
                            warn!("[Kad] Mode changed to {new_mode}");
                        }
                        _ => {}
                    },
                    BehaviourEvent::Autonat(event) => {
                        match event {
                            autonat::Event::InboundProbe(event) => match event {
                                autonat::InboundProbeEvent::Request {
                                    probe_id,
                                    peer,
                                    addresses,
                                } => {
                                    info!(
                                "[Autonat ProbeId {probe_id:?}] Dialing {peer} with {addresses:?}"
                            );
                                }
                                autonat::InboundProbeEvent::Response {
                                    probe_id,
                                    peer,
                                    address,
                                } => {
                                    info!(
                                "[Autonat ProbeId {probe_id:?}] Dial back for {address} was sent to {peer}"
                            );
                                }
                                autonat::InboundProbeEvent::Error {
                                    probe_id,
                                    peer,
                                    error,
                                } => {
                                    error!("[Autonat {probe_id:?}] Failed to dial {peer} because of {error:?}");
                                }
                            },
                            autonat::Event::OutboundProbe(event) => match event {
                                autonat::OutboundProbeEvent::Request { probe_id, peer } => {
                                    info!("[Autonat {probe_id:?}] Outbound probe request was sent to {peer}");
                                }
                                autonat::OutboundProbeEvent::Response {
                                    probe_id,
                                    peer,
                                    address,
                                } => {
                                    info!("[Autonat {probe_id:?}] A remote peer {peer} successfully dialed one of our remote addresses {address}");
                                }
                                autonat::OutboundProbeEvent::Error {
                                    probe_id,
                                    peer,
                                    error,
                                } => {
                                    if let Some(peer) = peer {
                                        error!("[Autonat {probe_id:?}] Outbound probe failed for {peer}: {error:?}");
                                    } else {
                                        error!("[Autonat {probe_id:?}] Outbound probe failed: {error:?}");
                                    }
                                }
                            },
                            autonat::Event::StatusChanged { old, new } => {
                                warn!("[Autonat] Status changed from {old:?} to {new:?}");
                                self.booting.replace(false);
                            }
                        }
                    }
                    _ => {}
                }
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                num_established,
                established_in,
                ..
            } => {
                info!(
                    "[ConnId {connection_id}] Connection established with {peer_id} ({num_established} established in {established_in:?})"
                );
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => {
                warn!(
                    "[ConnId {connection_id}] Connection closed with {peer_id} ({num_established} established in {endpoint:?}) because of {cause:?}"
                );
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                warn!("[ConnId {connection_id}] Incoming connection for {local_addr} from {send_back_addr}");
            }
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id: Some(peer_id),
                error,
            } => {
                error!(
                    "[ConnId {connection_id}] Failed to connect to peer: {} {error:?}",
                    peer_id
                );
            }
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                info!("[{listener_id}] Listening on {:?}", address);
            }
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                connection_id,
            } => {
                warn!("[ConnId {connection_id}] Dialing peer: {}", peer_id);
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                self.swarm.add_external_address(address);
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                self.swarm.remove_external_address(&address);
            }
            _ => {}
        }
    }
}
