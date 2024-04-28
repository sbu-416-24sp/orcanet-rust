use libp2p::{core::ConnectedPoint, swarm::SwarmEvent, Swarm};
use log::{error, info, warn};
use tokio::sync::oneshot;

use crate::{
    behaviour::Behaviour,
    command::{request::Request, QueryHandler},
    handler::req_res::ReqResHandler,
    lmm::LocalMarketMap,
    BootNodes, Response, SuccessfulResponse,
};

use self::{
    autonat::AutoNatHandler,
    dcutr::DcutrHandler,
    identify::IdentifyHandler,
    kad::KadHandler,
    ping::PingHandler,
    relay::{client::RelayClientHandler, server::RelayServerHandler},
};

use super::behaviour::BehaviourEvent;

pub(crate) trait EventHandler {
    type Event;
    fn handle_event(&mut self, event: Self::Event);
}

pub(crate) trait CommandRequestHandler {
    fn handle_command(&mut self, request: Request, responder: oneshot::Sender<Response>);
}

// NOTE: one lifetime should be covariant enough?
pub(crate) struct Handler<'a> {
    swarm: &'a mut Swarm<Behaviour>,
    lmm: &'a mut LocalMarketMap,
    query_handler: &'a mut QueryHandler,
    boot_nodes: Option<&'a BootNodes>,
}

impl<'a> Handler<'a> {
    pub(crate) fn new(
        swarm: &'a mut Swarm<Behaviour>,
        lmm: &'a mut LocalMarketMap,
        query_handler: &'a mut QueryHandler,
        boot_nodes: Option<&'a BootNodes>,
    ) -> Self {
        Handler {
            swarm,
            lmm,
            query_handler,
            boot_nodes,
        }
    }
}

impl<'a> EventHandler for Handler<'a> {
    type Event = SwarmEvent<BehaviourEvent>;

    fn handle_event(&mut self, event: Self::Event) {
        // NOTE:  maybe use  box,dyn but that would remove zca?
        // or implement a proc macro in the future
        match event {
            SwarmEvent::Behaviour(event) => match event {
                BehaviourEvent::Kad(event) => {
                    let mut kad_handler = KadHandler::new(self.swarm, self.lmm, self.query_handler);
                    kad_handler.handle_event(event);
                }
                BehaviourEvent::Identify(event) => {
                    let mut identify_handler = IdentifyHandler::new(self.swarm);
                    identify_handler.handle_event(event);
                }
                BehaviourEvent::Ping(event) => {
                    let mut ping_handler = PingHandler {};
                    ping_handler.handle_event(event);
                }
                BehaviourEvent::Autonat(event) => {
                    let mut autonat_handler = AutoNatHandler::new(self.boot_nodes);
                    autonat_handler.handle_event(event);
                }
                BehaviourEvent::RelayServer(event) => {
                    let mut relay_server_handler = RelayServerHandler {};
                    relay_server_handler.handle_event(event);
                }
                BehaviourEvent::Dcutr(event) => {
                    let mut dcutr_handler = DcutrHandler {};
                    dcutr_handler.handle_event(event);
                }
                BehaviourEvent::RelayClient(event) => {
                    let mut relay_client = RelayClientHandler {};
                    relay_client.handle_event(event);
                }
                BehaviourEvent::ReqRes(event) => {
                    let mut req_res_handler =
                        ReqResHandler::new(self.swarm, self.lmm, self.query_handler);
                    req_res_handler.handle_event(event);
                }
            },
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                established_in,
                ..
            } => {
                match endpoint {
                    ConnectedPoint::Dialer { address, .. } => {
                        info!("[Swarm ConnectionId {connection_id}] - Connection established by dialing {peer_id} at {address}");
                    }
                    ConnectedPoint::Listener {
                        local_addr,
                        send_back_addr,
                    } => {
                        info!("[Swarm ConnectionId {connection_id}] - Connection established by listening on {local_addr} from {peer_id}'s {send_back_addr}.");
                    }
                };
                info!("[Swarm ConnectionId {connection_id}] - Connections Established with this peer: {num_established}");
                info!("[Swarm ConnectionId {connection_id}] - Established in: {established_in:?}");
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => {
                match endpoint {
                    ConnectedPoint::Dialer { address, .. } => {
                        warn!("[Swarm ConnectionId {connection_id}] - Connection closed with {peer_id} at {address}. Dialing was used to initially establish the connection.");
                    }
                    ConnectedPoint::Listener {
                        local_addr,
                        send_back_addr,
                    } => {
                        warn!("[Swarm ConnectionId {connection_id}] - Connection closed with {peer_id} at {send_back_addr}. Listening on {local_addr} was used to initially establish the connection.");
                    }
                };
                warn!("[Swarm ConnectionId {connection_id}] - Connections Established with this peer: {num_established}");
                if let Some(cause) = cause {
                    error!(
                        "[Swarm ConnectionId {connection_id}] - Connection closed due to: {cause}"
                    );
                }
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                info!(
                    "[Swarm ConnectionId {connection_id}] - Incoming Connection from a peer with {send_back_addr}. We're listening on {local_addr}."
                );
            }
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => {
                error!(
                    "[Swarm ConnectionId {connection_id}] - Incoming Connection Error from a peer with {send_back_addr} to {local_addr}. Reason: {error}"
                );
            }
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id: Some(peer_id),
                error,
            } => {
                error!("[Swarm ConnectionId {connection_id}] - Outgoing Connection Error to {peer_id}. Reason: {error:?}");
            }
            SwarmEvent::NewListenAddr {
                address,
                listener_id,
            } => {
                info!(
                    "[Swarm ListenerId {listener_id}] - New Listen Address: {}",
                    address
                );
            }
            SwarmEvent::ExpiredListenAddr {
                address,
                listener_id,
            } => {
                // NOTE: relay client automatically renews reservations
                warn!(
                    "[Swarm ListenerId {listener_id}] - Expired Listen Address: {}",
                    address
                );
            }
            SwarmEvent::ListenerClosed {
                addresses,
                reason,
                listener_id,
            } => {
                if let Err(err) = reason {
                    warn!(
                        "[Swarm ListenerId {listener_id}] - Listener closed due to error: {}",
                        err
                    );
                } else {
                    warn!("[Swarm ListenerId {listener_id}] - Listener closed");
                }
                warn!("[Swarm ListenerId {listener_id}] - {addresses:?} are now expired");
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                error!("[Swarm ListenerId {listener_id}] - Listener reported an error: {error}")
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => {
                if let Some(peer_id) = peer_id {
                    info!(
                        "[Swarm ConnectionId {connection_id}] - Dialing peer: {}",
                        peer_id
                    );
                } else {
                    warn!(
                        "[Swarm ConnectionId {connection_id}] - Dialing a peer without a peer id"
                    );
                }
            }
            SwarmEvent::NewExternalAddrCandidate { address } => {
                warn!(
                    "[Swarm ExternalAddr] - New External Address Candidate: {}",
                    address
                );
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                info!(
                    "[Swarm ExternalAddr] - External Address Confirmed: {}",
                    address
                );
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                warn!(
                    "[Swarm ExternalAddr] - External Address Expired: {}",
                    address
                );
            }
            _ => {}
        }
    }
}

impl<'a> CommandRequestHandler for Handler<'a> {
    fn handle_command(&mut self, request: Request, responder: oneshot::Sender<Response>) {
        match request {
            Request::Listeners => {
                let listeners = self.swarm.listeners().cloned().collect();
                send_ok!(responder, SuccessfulResponse::Listeners { listeners });
            }
            Request::ConnectedPeers => {
                let peers = self.swarm.connected_peers().cloned().collect();
                send_ok!(responder, SuccessfulResponse::ConnectedPeers { peers });
            }
            Request::ConnectedTo { peer_id } => {
                let connected = self.swarm.is_connected(&peer_id);
                send_ok!(responder, SuccessfulResponse::ConnectedTo { connected });
            }
        };
    }
}

mod macros {
    macro_rules! send_ok {
        ($sender:expr, $response:expr) => {
            if let Err(_) = $sender.send(Ok($response)) {
                error!("Failed to send response back to peer!");
            }
        };
    }
    macro_rules! send_err {
        ($sender:expr, $response:expr) => {
            if let Err(_) = $sender.send(Err($response)) {
                error!("Failed to send response back to peer!");
            }
        };
    }
    pub(crate) use send_err;
    pub(crate) use send_ok;
}
pub(crate) use macros::send_err;
pub(crate) use macros::send_ok;

mod autonat;
mod dcutr;
mod identify;
mod kad;
mod ping;
mod relay;
mod req_res;
