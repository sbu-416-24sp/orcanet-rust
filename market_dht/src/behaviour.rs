use crate::{boot_nodes::BootNodes, Multiaddr, PeerId};
use libp2p::{
    identify::{self, Behaviour as IdentifyBehaviour},
    kad::{
        self,
        store::{MemoryStore, RecordStore},
        Behaviour as KadBehaviour, BootstrapOk, InboundRequest, QueryResult, QueryStats,
    },
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use log::{error, info, warn};
use thiserror::Error;

pub(crate) const IDENTIFY_PROTOCOL_NAME: &str = "/orcanet/id/1.0.0";
pub(crate) const KAD_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/orcanet/kad/1.0.0");

pub(crate) trait KadStore: RecordStore + Send + Sync + 'static {}
impl KadStore for MemoryStore {}

#[derive(NetworkBehaviour)]
#[allow(missing_debug_implementations)]
#[non_exhaustive] // NOTE: maybe more protocols?
pub(crate) struct MarketBehaviour<TKadStore: KadStore> {
    kademlia: Kad<TKadStore>,
    identify: Identify,
}

impl<TKadStore: KadStore> MarketBehaviour<TKadStore> {
    pub(crate) const fn new(
        kademlia: KadBehaviour<TKadStore>,
        identify: IdentifyBehaviour,
    ) -> Self {
        Self {
            kademlia: Kad::new(kademlia),
            identify: Identify::new(identify),
        }
    }

    pub(crate) fn handle_event(&mut self, event: MarketBehaviourEvent<TKadStore>) {
        match event {
            MarketBehaviourEvent::Kademlia(event) => {
                self.handle_kad_event(event);
            }
            MarketBehaviourEvent::Identify(event) => self.handle_identify_event(event),
        }
    }

    pub(crate) fn bootstrap_with_nodes(&mut self, boot_nodes: BootNodes) -> Result<(), KadError> {
        self.kademlia.bootstrap_with_nodes(boot_nodes)?;
        Ok(())
    }

    pub(crate) fn bootstrap_peer(&mut self) -> Result<(), KadError> {
        self.kademlia.bootstrap_peer()?;
        Ok(())
    }

    fn handle_kad_event(&mut self, KadEvent::Kad(event): KadEvent<TKadStore>) {
        match event {
            kad::Event::InboundRequest { request } => {
                self.kademlia.handle_inbound_request(request);
            }
            kad::Event::OutboundQueryProgressed {
                id,
                result,
                stats,
                step,
            } => {
                self.kademlia.handle_outbound_query(id, result);
            }
            kad::Event::RoutingUpdated {
                peer, addresses, ..
            } => {
                warn!(
                    "Routing updated for peer {} with addresses: {addresses:?}",
                    peer
                );
            }
            kad::Event::UnroutablePeer { peer } => {
                error!("Peer {} is unroutable", peer);
            }
            kad::Event::RoutablePeer { peer, address } => todo!(),
            kad::Event::PendingRoutablePeer { peer, address } => todo!(),
            kad::Event::ModeChanged { new_mode } => {
                info!("Kademlia mode changed to {}", new_mode);
            }
        }
    }

    fn handle_identify_event(&mut self, IdentifyEvent::Identify(event): IdentifyEvent) {
        match event {
            identify::Event::Received {
                peer_id,
                info:
                    identify::Info {
                        listen_addrs,
                        protocols,
                        ..
                    },
            } => {
                warn!("Peer {peer_id} identified with listen addresses: {listen_addrs:?} and protocols: {protocols:?}");
                if protocols.iter().any(|proto| proto == &KAD_PROTOCOL_NAME) {
                    for addr in listen_addrs {
                        self.kademlia.kad.add_address(&peer_id, addr);
                    }
                }
            }
            identify::Event::Sent { peer_id } => {
                info!("Sent an identify request to peer {}", peer_id)
            }
            identify::Event::Pushed { peer_id, info } => {
                warn!("Pushed identify info to peer {peer_id}: {info:?}")
            }
            identify::Event::Error { peer_id, error } => {
                error!("Error identifying peer {peer_id}: {error}")
            }
        }
    }
}

#[derive(NetworkBehaviour)]
pub(crate) struct Kad<TStore> {
    kad: KadBehaviour<TStore>,
}

#[derive(NetworkBehaviour)]
pub(crate) struct Identify {
    identify: IdentifyBehaviour,
}

impl Identify {
    #[inline(always)]
    const fn new(identify: IdentifyBehaviour) -> Self {
        Self { identify }
    }
}

impl<TStore: KadStore> Kad<TStore> {
    const fn new(kad: KadBehaviour<TStore>) -> Self {
        Self { kad }
    }

    fn bootstrap_with_nodes(&mut self, boot_nodes: BootNodes) -> Result<(), KadError> {
        for node in boot_nodes {
            self.kad.add_address(&node.peer_id, node.addr);
        }
        self.bootstrap_peer()?;
        Ok(())
    }

    fn bootstrap_peer(&mut self) -> Result<(), KadError> {
        self.kad
            .bootstrap()
            .map_err(|err| KadError::Bootstrap(err.to_string()))?;
        Ok(())
    }

    fn handle_outbound_query(&mut self, qid: kad::QueryId, result: QueryResult) {
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
                        error!("Bootstrap query {qid} timed out with peer {peer}",);
                    }
                }
            },
            QueryResult::GetClosestPeers(_) => todo!(),
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

#[derive(Debug, Error)]
pub(crate) enum KadError {
    #[error("Failed to bootstrap Kademlia: {0}")]
    Bootstrap(String),
}
