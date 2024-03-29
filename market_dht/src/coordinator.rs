use std::time::Duration;

use futures::StreamExt;
use libp2p::{kad::store::MemoryStore, swarm::SwarmEvent, Swarm};
use log::{error, info, warn};
use tokio::{
    sync::{mpsc, oneshot::Sender},
    time,
};

use crate::{
    behaviour::{BootstrapMode, MarketBehaviour, MarketBehaviourEvent},
    config::Config,
    peer::Peer,
    request::Request,
};

const BOOTSTRAP_REFRESH_INTERVAL: Duration = Duration::from_secs(5);

pub(crate) struct Coordinator {
    swarm: Swarm<MarketBehaviour<MemoryStore>>,
}

impl Coordinator {
    pub(crate) fn new(mut swarm: Swarm<MarketBehaviour<MemoryStore>>, config: Config) -> Self {
        swarm.listen_on(config.listener).expect("listen_on to work");
        if let Some(boot_nodes) = config.boot_nodes {
            swarm
                .behaviour_mut()
                .bootstrap(BootstrapMode::WithNodes(boot_nodes))
                .expect("initial bootstrap to work");
        }
        Self { swarm }
    }

    fn handle_bootstrap_refresh(&mut self) {
        if let Err(err) = self
            .swarm
            .behaviour_mut()
            .bootstrap(BootstrapMode::WithoutNodes)
        {
            error!("Failed to bootstrap peer: {}", err);
        }
    }

    pub(crate) async fn run(mut self, ready_tx: Sender<Peer>) {
        let (request_tx, mut request_rx) = mpsc::unbounded_channel();
        let peer = Peer::new(request_tx, *self.swarm.local_peer_id());
        ready_tx.send(peer).unwrap();
        let mut bootstrap_refresh_interval = time::interval(BOOTSTRAP_REFRESH_INTERVAL);

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

    async fn handle_request(&mut self, request: Request) {
        match request {}
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<MarketBehaviourEvent<MemoryStore>>) {
        match event {
            SwarmEvent::Behaviour(event) => {
                self.swarm.behaviour_mut().handle_event(event);
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
                peer_id,
                error,
            } => {
                let peer_id = {
                    if let Some(peer_id) = peer_id {
                        format!(" to {peer_id} ")
                    } else {
                        " ".to_owned()
                    }
                };
                error!(
                    "[ConnId {connection_id}] - Outgoing connection{peer_id}failed with {error}"
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
                self.swarm.add_external_address(address);
            }
            _ => {
                error!("Unhandled swarm event: {:?}", event);
            }
        }
    }
}
