use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use thiserror::Error;

use futures::StreamExt;
use libp2p::{kad::store::MemoryStore, swarm::SwarmEvent, Multiaddr, Swarm};
use log::{error, info, warn};
use tokio::{sync::mpsc, time};

use crate::{
    behaviour::{
        file_req_res::{FileHash, FileReqResHandler, SupplierInfo},
        ident::IdentifyHandler,
        kademlia::{BootstrapMode, KadHandler},
        MarketBehaviour, MarketBehaviourEvent,
    },
    boot_nodes::BootNodes,
    net::PROVIDER_RECORD_TTL,
    req_res::{Request, RequestData, RequestHandler, ResponseData},
};

const BOOTSTRAP_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 10);

pub(crate) struct Coordinator {
    swarm: Swarm<MarketBehaviour<MemoryStore>>,
    kad_handler: KadHandler,
    identify_handler: IdentifyHandler,
    file_req_res_handler: FileReqResHandler,
    market_map: LocalMarketMap,
    request_receiver: mpsc::UnboundedReceiver<Request>,
}

impl Coordinator {
    pub(crate) fn new(
        mut swarm: Swarm<MarketBehaviour<MemoryStore>>,
        listen_addr: Multiaddr,
        boot_nodes: Option<BootNodes>,
        request_receiver: mpsc::UnboundedReceiver<Request>,
    ) -> Result<Self, CoordinatorError> {
        swarm
            .listen_on(listen_addr)
            .map_err(|err| CoordinatorError::SpawnError(err.to_string()))?;
        if let Some(boot_nodes) = boot_nodes {
            swarm
                .behaviour_mut()
                .kademlia_mut()
                .bootstrap(BootstrapMode::WithNodes(boot_nodes))
                .map_err(|err| CoordinatorError::SpawnError(err.to_string()))?;
        }
        Ok(Self {
            swarm,
            kad_handler: Default::default(),
            identify_handler: Default::default(),
            file_req_res_handler: Default::default(),
            market_map: Default::default(),
            request_receiver,
        })
    }

    pub(crate) async fn run(mut self) {
        let mut bootstrap_refresh_interval = time::interval(BOOTSTRAP_REFRESH_INTERVAL);

        loop {
            tokio::select! {
                _ = bootstrap_refresh_interval.tick() => {
                    self.handle_bootstrap_refresh();
                }
                request = self.request_receiver.recv() => {
                    if let Some(request) = request {
                        self.handle_request(request.0, request.1);
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

    fn handle_event(&mut self, event: MarketBehaviourEvent<MemoryStore>) {
        match event {
            MarketBehaviourEvent::Kademlia(event) => self
                .kad_handler
                .handle_kad_event(event, &mut self.market_map),
            MarketBehaviourEvent::Identify(event) => self
                .identify_handler
                .handle_identify_event(event, self.swarm.behaviour_mut().kademlia_mut()),
            MarketBehaviourEvent::FileReqRes(event) => {
                self.file_req_res_handler.handle_event(
                    event,
                    &mut self.market_map,
                    self.swarm.behaviour_mut().file_req_res_mut(),
                );
            }
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

    fn handle_request(&mut self, request_data: RequestData, request_handler: RequestHandler) {
        match request_data {
            RequestData::GetAllListeners => {
                let listeners = self.swarm.listeners().cloned().collect::<Vec<_>>();
                request_handler.respond(Ok(ResponseData::AllListeners { listeners }));
            }
            RequestData::GetConnectedPeers => {
                let connected_peers = self.swarm.connected_peers().cloned().collect::<Vec<_>>();
                request_handler.respond(Ok(ResponseData::ConnectedPeers { connected_peers }));
            }
            RequestData::IsConnectedTo(peer_id) => {
                let is_connected = self.swarm.is_connected(&peer_id);
                request_handler.respond(Ok(ResponseData::IsConnectedTo { is_connected }));
            }
            RequestData::KadRequest(request) => self.kad_handler.handle_kad_request(
                self.swarm.behaviour_mut().kademlia_mut(),
                request_handler,
                request,
                &mut self.market_map,
            ),
            RequestData::ReqResRequest(request) => {
                self.file_req_res_handler.handle_request(
                    request,
                    request_handler,
                    self.swarm.behaviour_mut().file_req_res_mut(),
                );
            }
            RequestData::GetLocalSupplierInfo { file_hash } => {
                request_handler.respond(Ok(ResponseData::GetLocalSupplierInfo {
                    supplier_info: self.market_map.get_if_not_expired(&file_hash),
                }));
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
                // TODO: something we need to focus on when we allow user to use more listening
                // addresses maybe?
                if num_established == 0 {
                    self.swarm
                        .behaviour_mut()
                        .kademlia_mut()
                        .kad_mut()
                        .remove_peer(&peer_id);
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

#[derive(Debug, Default)]
pub(crate) struct LocalMarketMap {
    inner: HashMap<FileHash, (SupplierInfo, CreationTime)>,
}

impl LocalMarketMap {
    pub(crate) fn remove(&mut self, file_hash: &FileHash) {
        self.inner.remove(file_hash);
    }

    pub(crate) fn insert(&mut self, file_hash: FileHash, supplier_info: SupplierInfo) {
        self.inner
            .insert(file_hash, (supplier_info, CreationTime::now()));
    }

    pub(crate) fn get_if_not_expired(&mut self, file_hash: &FileHash) -> Option<SupplierInfo> {
        if let Some((supplier_info, creation_time)) = self.inner.get(file_hash) {
            let elapsed_time = Instant::now().duration_since(*creation_time);
            if elapsed_time >= PROVIDER_RECORD_TTL {
                self.inner.remove(file_hash);
                None
            } else {
                // NOTE: okay to clone here since we just clone all the time
                // but will prob refactor to not clone later
                Some(supplier_info.clone())
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum CoordinatorError {
    #[error("Failed to spawn coordinator {0}")]
    SpawnError(String),
}

pub(crate) type CreationTime = Instant;
