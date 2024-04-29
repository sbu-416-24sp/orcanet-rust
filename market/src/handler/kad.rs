use libp2p::{
    kad::{
        AddProviderError, AddProviderOk, BootstrapError, Event, GetClosestPeersError,
        GetProvidersError, GetProvidersOk, InboundRequest, ProgressStep, QueryId, QueryResult,
    },
    Swarm,
};
use log::{error, info, warn};
use tokio::sync::oneshot;

use crate::{
    behaviour::Behaviour,
    command::{
        request::{KadRequest, Query},
        QueryHandler,
    },
    handler::send_err,
    lmm::{LocalMarketMap, SupplierInfo},
    FailureResponse, KadFailureResponse, KadSuccessfulResponse, Response, SuccessfulResponse,
};

use super::{CommandRequestHandler, EventHandler};

pub(crate) struct KadHandler<'a> {
    swarm: &'a mut Swarm<Behaviour>,
    lmm: &'a mut LocalMarketMap,
    query_handler: &'a mut QueryHandler,
}

impl<'a> KadHandler<'a> {
    pub(crate) fn new(
        swarm: &'a mut Swarm<Behaviour>,
        lmm: &'a mut LocalMarketMap,
        query_handler: &'a mut QueryHandler,
    ) -> Self {
        KadHandler {
            swarm,
            lmm,
            query_handler,
        }
    }

    fn handle_inbound_request(&self, request: InboundRequest) {
        match request {
            InboundRequest::FindNode { num_closer_peers } => {
                warn!("[Kademlia] - FindNode request received and handled");
                info!("[Kademlia] - The number of closest peers found {num_closer_peers}");
            }
            InboundRequest::GetProvider {
                num_closer_peers,
                num_provider_peers,
            } => {
                warn!("[Kademlia] - GetProvider request received and handled");
                info!("[Kademlia] - The number of closest peers found {num_closer_peers}");
                info!("[Kademlia] - The number of provider peers found {num_provider_peers} for this particular key");
            }
            InboundRequest::AddProvider { .. } => {
                warn!("[Kademlia] - AddProvider request received and handled");
            }
            _ => {}
        }
    }

    fn handle_outbound_event(&mut self, qid: QueryId, result: QueryResult, step: ProgressStep) {
        match result {
            QueryResult::Bootstrap(result) => match result {
                Ok(ok) => {
                    info!("[Kademlia] - Bootstrap query successful");
                    info!("[Kademlia] - Successfully bootstrapped to {}", ok.peer)
                }
                Err(BootstrapError::Timeout { peer, .. }) => {
                    error!("[Kademlia] - Bootstrap query failed due to timeout. Could not bootstrap to peer {peer} in time.");
                }
            },
            QueryResult::GetClosestPeers(result) => {
                if step.last {
                    match result {
                        Ok(ok) => {
                            info!("[Kademlia] - GetClosestPeers query successful");
                            for peer in &ok.peers {
                                info!("[Kademlia] - Peer {peer} is one of the closest peers found");
                            }
                            self.query_handler.respond(
                                Query::Kad(qid),
                                Ok(SuccessfulResponse::KadResponse(
                                    KadSuccessfulResponse::GetClosestPeers { peers: ok.peers },
                                )),
                            )
                        }
                        Err(GetClosestPeersError::Timeout { key, .. }) => {
                            error!("[Kademlia] - GetClosestPeers query failed due to timeout.");
                            self.query_handler.respond(
                                Query::Kad(qid),
                                Err(FailureResponse::KadError(
                                    KadFailureResponse::GetClosestPeers {
                                        key,
                                        error: "timeout".to_owned(),
                                    },
                                )),
                            )
                        }
                    }
                }
            }
            QueryResult::GetProviders(result) => match result {
                Ok(maybe_ok) => {
                    if step.last {
                        match maybe_ok {
                            GetProvidersOk::FoundProviders { providers, .. } => {
                                info!("[Kademlia] - GetProviders query successful");
                                self.query_handler.respond(
                                    Query::Kad(qid),
                                    Ok(SuccessfulResponse::KadResponse(
                                        KadSuccessfulResponse::GetProviders {
                                            providers: providers.into_iter().collect(),
                                        },
                                    )),
                                );
                            }
                            GetProvidersOk::FinishedWithNoAdditionalRecord { .. } => {
                                warn!("[Kademlia] - GetProviders query didn't necessarily fail, but no additional records were found.");
                                self.query_handler.respond(
                                    Query::Kad(qid),
                                    Ok(SuccessfulResponse::KadResponse(
                                        KadSuccessfulResponse::GetProviders {
                                            providers: Default::default(),
                                        },
                                    )),
                                );
                            }
                        }
                    };
                }
                Err(GetProvidersError::Timeout { .. }) => {
                    error!("[Kademlia] - GetProviders query failed due to timeout.");
                    self.query_handler.respond(
                        Query::Kad(qid),
                        Err(FailureResponse::KadError(
                            KadFailureResponse::GetProviders {
                                error: "timeout".to_owned(),
                            },
                        )),
                    );
                }
            },
            QueryResult::StartProviding(result) => match result {
                Ok(AddProviderOk { .. }) => {
                    info!("[Kademlia] - StartProviding query successful");
                    self.query_handler.respond(
                        Query::Kad(qid),
                        Ok(SuccessfulResponse::KadResponse(
                            KadSuccessfulResponse::RegisterFile,
                        )),
                    )
                }
                Err(AddProviderError::Timeout { .. }) => {
                    error!("[Kademlia] - StartProviding query failed due to timeout.");
                    self.query_handler.respond(
                        Query::Kad(qid),
                        Err(FailureResponse::KadError(
                            KadFailureResponse::RegisterFile {
                                error: "timeout".to_owned(),
                            },
                        )),
                    )
                }
            },
            _ => {}
        }
    }
}

impl<'a> EventHandler for KadHandler<'a> {
    type Event = Event;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::InboundRequest { request } => self.handle_inbound_request(request),
            Event::OutboundQueryProgressed {
                id, result, step, ..
            } => {
                self.handle_outbound_event(id, result, step);
            }
            Event::RoutingUpdated {
                peer,
                is_new_peer,
                addresses,
                old_peer,
                ..
            } => {
                warn!("[Kademlia] - Routing table updated");
                info!("[Kademlia] - Peer {peer} has been updated in the routing table");
                if is_new_peer {
                    warn!("[Kademlia] - Peer {peer} is a new peer that has been added to the routing table");
                }
                info!("[Kademlia] - Peer {peer} has the following addresses: {addresses:?}");
                if let Some(old_peer) = old_peer {
                    warn!("[Kademlia] - Peer {old_peer} has been replaced by peer {peer}. The old peer has been evicted.");
                }
            }
            Event::UnroutablePeer { peer } => {
                warn!("[Kademlia] - Peer {peer} is unroutable. Peer {peer} has connected, but has no known listening addresses.");
            }
            Event::RoutablePeer { peer, address } => {
                // TODO: contemplating if we still need to actually add it into the routing table?
                // not sure if it does it automatically? Can't find any other documentation on this
                // or examples
                warn!("[Kademlia] - Peer {peer} is routable");
                info!("[Kademlia] - Peer {peer} has the following address: {address}");
            }
            Event::ModeChanged { new_mode } => {
                warn!("[Kademlia] - Mode changed to {new_mode}");
            }
            _ => {}
        }
    }
}

impl<'a> CommandRequestHandler for KadHandler<'a> {
    type Request = KadRequest;
    fn handle_command(&mut self, request: Self::Request, responder: oneshot::Sender<Response>) {
        match request {
            KadRequest::GetClosestPeers { key } => {
                let qid = self.swarm.behaviour_mut().kad.get_closest_peers(key);
                self.query_handler.add_query(Query::Kad(qid), responder);
            }
            KadRequest::RegisterFile {
                file_info_hash,
                file_info,
                user,
            } => {
                let res = self
                    .swarm
                    .behaviour_mut()
                    .kad
                    .start_providing(file_info_hash.clone().into_bytes().into());
                match res {
                    Ok(qid) => {
                        self.lmm
                            .insert(file_info_hash, SupplierInfo { file_info, user });
                        self.query_handler.add_query(Query::Kad(qid), responder);
                    }
                    Err(err) => {
                        send_err!(
                            responder,
                            FailureResponse::KadError(KadFailureResponse::RegisterFile {
                                error: err.to_string(),
                            })
                        );
                    }
                };
            }
            KadRequest::GetProviders { file_info_hash } => {
                let qid = self
                    .swarm
                    .behaviour_mut()
                    .kad
                    .get_providers(file_info_hash.into_bytes().into());
                self.query_handler.add_query(Query::Kad(qid), responder);
            }
        }
    }
}
