use std::collections::HashMap;

use anyhow::anyhow;
use libp2p::{
    kad::{
        self,
        store::{MemoryStore, RecordStore},
        AddProviderError, AddProviderOk, Behaviour as KadBehaviour, GetClosestPeersOk,
        GetProvidersOk, InboundRequest, ProgressStep, QueryId, QueryResult, QueryStats,
    },
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use log::{debug, error, info, warn};
use thiserror::Error;

use crate::{
    behaviour::send_response,
    boot_nodes::BootNodes,
    coordinator::LocalMarketMap,
    req_res::{KadRequestData, KadResponseData, RequestHandler, ResponseData},
};

use super::file_req_res::FileHash;

pub(crate) const KAD_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/orcanet/kad/1.0.0");
pub(crate) trait KadStore: RecordStore + Send + Sync + 'static {}

#[derive(Debug, Default)]
pub(crate) struct KadHandler {
    pending_queries: HashMap<QueryId, RequestHandler>,
}

impl KadHandler {
    pub(crate) fn handle_kad_request<TKadStore: KadStore>(
        &mut self,
        Kad { kad }: &mut Kad<TKadStore>,
        request_handler: RequestHandler,
        request: KadRequestData,
        market_map: &mut LocalMarketMap,
    ) {
        match request {
            KadRequestData::ClosestLocalPeers { key } => {
                let peers = kad
                    .get_closest_local_peers(&key.into())
                    .map(|key| key.into_preimage())
                    .collect::<Vec<PeerId>>();
                request_handler.respond(Ok(ResponseData::KadResponse(
                    KadResponseData::ClosestLocalPeers { peers },
                )));
            }
            KadRequestData::ClosestPeers { key } => {
                let qid = kad.get_closest_peers(key);
                self.pending_queries.insert(qid, request_handler);
            }
            KadRequestData::RegisterFile { file_metadata } => {
                // TODO: do something about the cloning here
                let key = file_metadata.file_hash;
                match kad.start_providing(key.0.clone().into()) {
                    Ok(qid) => {
                        self.pending_queries.insert(qid, request_handler);
                        market_map.insert(key, file_metadata.supplier_info);
                    }
                    Err(err) => {
                        send_response!(request_handler, err.into());
                    }
                }
            }
            KadRequestData::GetProviders { key } => {
                let qid = kad.get_providers(key.into());
                self.pending_queries.insert(qid, request_handler);
            }
        }
    }
    pub(crate) fn handle_kad_event<TKadStore: KadStore>(
        &mut self,
        KadEvent::Kad(event): KadEvent<TKadStore>,
        market_map: &mut LocalMarketMap,
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
                self.handle_outbound_query(id, result, stats, step, market_map);
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
            // TODO: maybe need the rest of these events later on with upnp and autonat?
            //
            // kad::Event::UnroutablePeer { peer } => {
            //     error!("Peer {} is unroutable", peer);
            // }
            // kad::Event::RoutablePeer { peer, address } => todo!(),
            // kad::Event::PendingRoutablePeer { peer, address } => todo!(),
            _ => {}
        }
    }

    fn handle_outbound_query(
        &mut self,
        qid: kad::QueryId,
        result: QueryResult,
        stats: QueryStats,
        step: ProgressStep,
        market_map: &mut LocalMarketMap,
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
                            Ok(GetClosestPeersOk { key, peers }) => {
                                Ok(ResponseData::KadResponse(KadResponseData::ClosestPeers {
                                    key,
                                    peers,
                                }))
                            }
                            Err(err) => Err(err.into()),
                        }
                    };
                    send_response!(self.pending_queries, qid, response);
                }
            }
            QueryResult::GetProviders(result) => match result {
                Ok(ok_res) => match ok_res {
                    GetProvidersOk::FoundProviders { key, providers } => {
                        info!("GetProviders query succeeded for key {key:?}!");
                        send_response!(
                            self.pending_queries,
                            qid,
                            Ok(ResponseData::KadResponse(KadResponseData::GetProviders {
                                key: key.to_vec(),
                                providers
                            }))
                        );
                    }
                    // TODO: maybe do something with the below event; i don't undderstand if it'll
                    // be useful atm
                    GetProvidersOk::FinishedWithNoAdditionalRecord { .. } => {
                        warn!("GetProviders query finished with no additional record");
                        send_response!(
                            self.pending_queries,
                            qid,
                            Err(anyhow!(
                                "GetProviders query finished with no additional record"
                            ))
                        );
                    }
                },
                Err(err) => {
                    error!("GetProviders query failed with error: {}", err);
                    send_response!(self.pending_queries, qid, Err(err.into()));
                }
            },
            QueryResult::StartProviding(result) => match result {
                Ok(AddProviderOk { key }) => {
                    info!("StartProviding query succeeded for key: {:?}", key);
                    send_response!(
                        self.pending_queries,
                        qid,
                        Ok(ResponseData::KadResponse(KadResponseData::RegisterFile {
                            key: key.to_vec()
                        }))
                    );
                }
                Err(AddProviderError::Timeout { key }) => {
                    error!("StartProviding query timed out for key: {:?}", key);
                    market_map.remove(&FileHash(key.to_vec()));
                    send_response!(
                        self.pending_queries,
                        qid,
                        Err(AddProviderError::Timeout { key }.into())
                    );
                }
            },
            QueryResult::RepublishProvider(result) => match result {
                Ok(AddProviderOk { key }) => {
                    info!("Successfully republished the key {key:?}!");
                }
                Err(AddProviderError::Timeout { key }) => {
                    error!("Timed out! Failed to republish the key {key:?}!");
                }
            },
            _ => {}
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
            } => {
                info!(
                    "GetProvider request handled. Number of closest peers found: {}. Number of provider peers found: {}",
                    num_closer_peers, num_provider_peers
                );
            }
            InboundRequest::AddProvider { .. } => {
                info!("AddProvider request handled");
            }
            // InboundRequest::GetRecord { .. } => {}
            // InboundRequest::PutRecord { .. } => {}
            _ => {}
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
