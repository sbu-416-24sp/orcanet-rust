use std::collections::HashMap;

use libp2p::{
    kad::{
        self,
        store::{MemoryStore, RecordStore},
        Behaviour as KadBehaviour, GetClosestPeersError, GetClosestPeersOk, InboundRequest,
        ProgressStep, QueryId, QueryResult, QueryStats,
    },
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use log::{debug, error, info, warn};
use thiserror::Error;

use crate::{
    behaviour::kademlia::macros::get_and_send,
    boot_nodes::BootNodes,
    req_res::{KadRequestData, KadResponseData, RequestHandler, ResponseData},
};

pub(crate) const KAD_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/orcanet/kad/1.0.0");
pub(crate) trait KadStore: RecordStore + Send + Sync + 'static {}

#[derive(Debug, Default)]
pub(crate) struct KadHandler {
    pending_queries: HashMap<QueryId, RequestHandler>,
}

impl KadHandler {
    pub(crate) async fn handle_kad_request<TKadStore: KadStore>(
        &mut self,
        Kad { kad }: &mut Kad<TKadStore>,
        request_handler: RequestHandler,
        request: KadRequestData,
    ) {
        match request {
            KadRequestData::GetClosestLocalPeers { key } => {
                let peers = kad
                    .get_closest_local_peers(&key.into())
                    .map(|key| key.into_preimage())
                    .collect::<Vec<PeerId>>();
                request_handler.respond(Ok(ResponseData::KadResponse(
                    KadResponseData::GetClosestLocalPeers { peers },
                )));
            }
            KadRequestData::GetClosestPeers { key } => {
                let qid = kad.get_closest_peers(key);
                self.pending_queries.insert(qid, request_handler);
            }
        }
    }
    pub(crate) fn handle_kad_event<TKadStore: KadStore>(
        &mut self,
        kad: &mut Kad<TKadStore>,
        KadEvent::Kad(event): KadEvent<TKadStore>,
    ) {
        match event {
            kad::Event::InboundRequest { request } => {
                self.handle_inbound_request(request);
            }
            kad::Event::OutboundQueryProgressed {
                id,
                result,
                stats,
                step,
            } => {
                self.handle_outbound_query(kad, id, result, stats, step);
            }
            kad::Event::RoutingUpdated {
                peer, addresses, ..
            } => {
                warn!(
                    "Routing updated for peer {} with addresses: {addresses:?}",
                    peer
                );
            }
            kad::Event::ModeChanged { new_mode } => {
                info!("Kademlia mode changed to {}", new_mode);
            }
            // kad::Event::UnroutablePeer { peer } => {
            //     error!("Peer {} is unroutable", peer);
            // }
            // kad::Event::RoutablePeer { peer, address } => todo!(),
            // kad::Event::PendingRoutablePeer { peer, address } => todo!(),
            _ => {}
        }
    }

    fn handle_outbound_query<TKadStore: KadStore>(
        &mut self,
        kad: &mut Kad<TKadStore>,
        qid: kad::QueryId,
        result: QueryResult,
        stats: QueryStats,
        step: ProgressStep,
    ) {
        debug!("Query {} progressed with stats: {:?}", qid, stats);
        match result {
            QueryResult::Bootstrap(res) => match res {
                Ok(kad::BootstrapOk {
                    peer,
                    num_remaining,
                }) => {
                    info!(
                            "Bootstrap query {qid} succeeded with peer {peer} and {num_remaining} remaining"
                        );
                }
                Err(kad::BootstrapError::Timeout {
                    peer,
                    num_remaining,
                }) => {
                    if let Some(num_remaining) = num_remaining {
                        error!(
                            "Bootstrap query {qid} timed out with peer {peer} and {num_remaining} remaining"
                        );
                    } else {
                        error!("Bootstrap query {qid} timed out with peer {peer}");
                    }
                }
            },
            QueryResult::GetClosestPeers(result) => {
                // NOTE: maybe only care for the last part of the step?
                if step.last {
                    let response = {
                        match result {
                            Ok(GetClosestPeersOk { key, peers }) => Ok(ResponseData::KadResponse(
                                KadResponseData::GetClosestPeers { key, peers },
                            )),
                            Err(err) => Err(err.into()),
                        }
                    };
                    get_and_send!(self.pending_queries, qid, response);
                }
            }
            QueryResult::GetProviders(_) => todo!(),
            QueryResult::StartProviding(_) => todo!(),
            QueryResult::RepublishProvider(_) => todo!(),
            QueryResult::GetRecord(_) => todo!(),
            QueryResult::PutRecord(_) => todo!(),
            QueryResult::RepublishRecord(_) => todo!(),
        }
    }

    fn handle_inbound_request(&mut self, request: InboundRequest) {
        match request {
            InboundRequest::FindNode { num_closer_peers } => {
                info!(
                    "FindNode request handled. Number of closest peers found: {}",
                    num_closer_peers
                );
            }
            InboundRequest::GetProvider {
                num_closer_peers,
                num_provider_peers,
            } => todo!(),
            InboundRequest::AddProvider { record } => todo!(),
            InboundRequest::GetRecord {
                num_closer_peers,
                present_locally,
            } => todo!(),
            InboundRequest::PutRecord {
                source,
                connection,
                record,
            } => todo!(),
        }
    }
}

#[derive(NetworkBehaviour)]
pub(crate) struct Kad<TKadStore> {
    kad: KadBehaviour<TKadStore>,
}

impl<TKadStore: KadStore> Kad<TKadStore> {
    pub(crate) const fn new(kad: KadBehaviour<TKadStore>) -> Self {
        Self { kad }
    }

    pub(crate) fn bootstrap(&mut self, mode: BootstrapMode) -> Result<(), KadError> {
        if let BootstrapMode::WithNodes(boot_nodes) = mode {
            for node in boot_nodes {
                self.kad.add_address(&node.peer_id, node.addr);
            }
        }
        self.kad
            .bootstrap()
            .map_err(|err| KadError::Bootstrap(err.to_string()))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) const fn kad(&self) -> &KadBehaviour<TKadStore> {
        &self.kad
    }

    pub(crate) fn kad_mut(&mut self) -> &mut KadBehaviour<TKadStore> {
        &mut self.kad
    }
}

#[derive(Debug, Error)]
pub(crate) enum KadError {
    #[error("Failed to bootstrap Kademlia: {0}")]
    Bootstrap(String),
}

#[derive(Debug)]
pub(crate) enum BootstrapMode {
    WithNodes(BootNodes),
    WithoutNodes,
}

impl KadStore for MemoryStore {}

mod macros {
    macro_rules! get_and_send {
        ($map: expr, $qid: expr, $map_type: expr) => {
            if let Some(handler) = $map.remove(&$qid) {
                handler.respond($map_type);
            }
        };
    }
    pub(super) use get_and_send;
}
