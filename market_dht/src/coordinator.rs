use std::time::Duration;

use futures::StreamExt;
use libp2p::{kad::store::MemoryStore, swarm::SwarmEvent, Swarm};
use log::{error, info, warn};
use tokio::{
    sync::{mpsc, oneshot::Sender},
    time,
};

use crate::{
    behaviour::{
        ident::IdentifyHandler,
        kademlia::{BootstrapMode, KadHandler},
        MarketBehaviour, MarketBehaviourEvent,
    },
    config::Config,
    peer::Peer,
    request::{RequestData, RequestHandler, ResponseData},
};

const BOOTSTRAP_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 30);

pub(crate) struct Coordinator {
    swarm: Swarm<MarketBehaviour<MemoryStore>>,
    kad_handler: KadHandler,
    identify_handler: IdentifyHandler,
}

impl Coordinator {
    pub(crate) fn new(mut swarm: Swarm<MarketBehaviour<MemoryStore>>, config: Config) -> Self {
        swarm.listen_on(config.listener).expect("listen_on to work");
        if let Some(boot_nodes) = config.boot_nodes {
            swarm
                .behaviour_mut()
                .kademlia_mut()
                .bootstrap(BootstrapMode::WithNodes(boot_nodes))
                .expect("initial bootstrap to work");
        }
        Self {
            swarm,
            kad_handler: Default::default(),
            identify_handler: Default::default(),
        }
    }

    fn handle_event(&mut self, event: MarketBehaviourEvent<MemoryStore>) {
        match event {
            MarketBehaviourEvent::Kademlia(event) => {
                self.kad_handler
                    .handle_kad_event(self.swarm.behaviour_mut().kademlia_mut(), event);
            }
            MarketBehaviourEvent::Identify(event) => self
                .identify_handler
                .handle_identify_event(event, self.swarm.behaviour_mut().kademlia_mut()),
        }
    }

    fn handle_bootstrap_refresh(&mut self) {
        if let Err(err) = self
            .swarm
            .behaviour_mut()
            .kademlia_mut()
            .bootstrap(BootstrapMode::WithoutNodes)
        {
            error!("Failed to bootstrap peer: {}", err);
        }
    }

    pub(crate) async fn run(mut self, ready_tx: Sender<Peer>) {
        let (request_tx, mut request_rx) = mpsc::unbounded_channel();
        let peer = Peer::new(request_tx, *self.swarm.local_peer_id());
        let mut bootstrap_refresh_interval = time::interval(BOOTSTRAP_REFRESH_INTERVAL);
        ready_tx.send(peer).unwrap();

        loop {
            tokio::select! {
                _ = bootstrap_refresh_interval.tick() => {
                    self.handle_bootstrap_refresh();
                }
                request = request_rx.recv() => {
                    if let Some(request) = request {
                        self.handle_request(request).await;
                    } else {
                        error!("request receiver channel closed, shutting down coordinator");
                        break;
                    }
                }
                swarm_event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(swarm_event).await;
                }
            }
        }
    }

    async fn handle_request(&mut self, request: RequestHandler) {
        match request.request_data {
            RequestData::GetAllListeners => {
                let listeners = self.swarm.listeners().cloned().collect::<Vec<_>>();
                request.respond(ResponseData::GetAllListeners { listeners });
            }
            RequestData::GetConnectedPeers => todo!(),
            RequestData::IsConnectedTo(peer_id) => {
                let is_connected = self.swarm.is_connected(&peer_id);
                request.respond(ResponseData::IsConnectedTo { is_connected });
            }
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<MarketBehaviourEvent<MemoryStore>>) {
        match event {
            SwarmEvent::Behaviour(event) => {
                self.handle_event(event);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                num_established,
                established_in,
                ..
            } => {
                info!("[ConnId {connection_id}] - Connection established with peer: {peer_id}. Number of established connections: {num_established}. Established in: {established_in:?}");
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                num_established,
                cause,
                ..
            } => {
                let cause = {
                    if let Some(cause) = cause {
                        format!("{}", cause)
                    } else {
                        "unknown".to_string()
                    }
                };
                warn!("[ConnId {connection_id}] - Connection closed with peer: {peer_id}. Number of established connections: {num_established}. Cause: {cause}");
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                info!(
                    "[ConnId {connection_id}: {local_addr}] - Incoming connection from: {:?}",
                    send_back_addr
                );
            }
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => {
                error!(
                    "[ConnId {connection_id}: {local_addr}] - Incoming connection from: {:?} failed with {error}",
                    send_back_addr
                );
            }
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id: Some(peer_id),
                error,
            } => {
                error!(
                    "[ConnId {connection_id}] - Outgoing connection to {peer_id} failed with {error}"
                );
            }
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                info!("[{listener_id}] - Listening on {:?}", address);
            }
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => {
                // TODO: do something about expired listen addresses since there's only one listen
                // addr in a session
                error!("[{listener_id}] - Expired listening on {}", address);
            }
            SwarmEvent::ListenerClosed { listener_id, .. } => {
                error!("[{listener_id}] - Listener closed");
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                error!("[{listener_id}] - Listener error: {error}");
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => {
                warn!("[ConnId {connection_id}] - Dialing peer: {:?}", peer_id);
            }
            SwarmEvent::NewExternalAddrCandidate { address } => {
                // TODO: this will be useful when we deal with NAT remotely since upnp emits a
                // SwarmEvent::ExternalAddressConfirmed event in which we will use to actually add
                // the address in I think
                self.swarm.add_external_address(address);
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                self.swarm.remove_external_address(&address);
            }
            _ => {
                error!("Unhandled swarm event: {:?}", event);
            }
        }
    }
}
