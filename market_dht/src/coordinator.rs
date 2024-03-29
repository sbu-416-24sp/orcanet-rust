use std::time::Duration;

use futures::StreamExt;
use libp2p::{core::ConnectedPoint, kad::store::MemoryStore, swarm::SwarmEvent, Swarm};
use log::{debug, error, info, warn};
use tokio::{
    sync::{mpsc, oneshot::Sender},
    time,
};

use crate::{
    behaviour::{MarketBehaviour, MarketBehaviourEvent},
    config::Config,
    peer::Peer,
    request::Request,
};

const BOOTSTRAP_REFRESH_INTERVAL: Duration = Duration::from_secs(10);

pub(crate) struct Coordinator {
    swarm: Swarm<MarketBehaviour<MemoryStore>>,
}

impl Coordinator {
    pub(crate) fn new(mut swarm: Swarm<MarketBehaviour<MemoryStore>>, config: Config) -> Self {
        swarm.listen_on(config.listener).expect("listen_on to work");
        if let Some(boot_nodes) = config.boot_nodes {
            swarm
                .behaviour_mut()
                .bootstrap_with_nodes(boot_nodes)
                .expect("bootstrap to work");
        }
        Self { swarm }
    }

    fn handle_bootstrap_refresh(&mut self) {
        if let Err(err) = self.swarm.behaviour_mut().bootstrap_peer() {
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
                        debug!("request receiver channel closed, shutting down coordinator");
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
                endpoint,
                num_established,
                established_in,
                ..
            } => {
                info!("[ConnId {connection_id}] - Connection established with peer: {peer_id}");
                info!("Number of established connections: {num_established}");
                info!("Established in: {established_in:?}");
                match endpoint {
                    ConnectedPoint::Dialer { address, .. } => {
                        info!("[ConnId {connection_id}] - Dialer connection to: {address}");
                    }
                    ConnectedPoint::Listener { send_back_addr, .. } => {
                        info!(
                            "[ConnId {connection_id}] - Listener connection from: {send_back_addr}"
                        );
                    }
                }
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => {
                warn!("[ConnId {connection_id}] - Connection closed with peer: {peer_id}");
                warn!("Number of established connections: {num_established}");
                warn!("Established in: {num_established:?}");
                match endpoint {
                    ConnectedPoint::Dialer { address, .. } => {
                        warn!("[ConnId {connection_id}] - Dialer connection to: {address} ended");
                    }
                    ConnectedPoint::Listener { send_back_addr, .. } => {
                        warn!(
                            "[ConnId {connection_id}] - Listener connection from: {send_back_addr} ended"
                        );
                    }
                }
                if let Some(cause) = cause {
                    warn!("Connection Closed: {cause}");
                }
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
            } => todo!(),
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                info!("[{listener_id}] - Listening on {:?}", address);
            }
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => todo!(),
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => todo!(),
            SwarmEvent::ListenerError { listener_id, error } => todo!(),
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => {
                warn!("[ConnId {connection_id}] - Dialing peer: {:?}", peer_id);
            }
            ev => {
                error!("Unsupported Swarm Event: {ev:?}");
            }
        }
    }
}
