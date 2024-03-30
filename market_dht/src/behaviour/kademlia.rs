use libp2p::{
    kad::{
        self,
        store::{MemoryStore, RecordStore},
        Behaviour as KadBehaviour, InboundRequest, QueryResult,
    },
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use log::{error, info, warn};
use thiserror::Error;

use crate::boot_nodes::BootNodes;

pub(crate) const KAD_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/orcanet/kad/1.0.0");

pub(crate) trait KadStore: RecordStore + Send + Sync + 'static {}
impl KadStore for MemoryStore {}

#[derive(NetworkBehaviour)]
pub(crate) struct Kad<TKadStore> {
    pub(crate) kad: KadBehaviour<TKadStore>,
}

impl<TKadStore: KadStore> Kad<TKadStore> {
    pub(crate) const fn new(kad: KadBehaviour<TKadStore>) -> Self {
        Self { kad }
    }

    pub(crate) fn handle_kad_event(&mut self, KadEvent::Kad(event): KadEvent<TKadStore>) {
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
                self.handle_outbound_query(id, result);
            }
            kad::Event::RoutingUpdated {
                peer, addresses, ..
            } => {
                warn!(
                    "Routing updated for peer {} with addresses: {addresses:?}",
                    peer
                );
            }
            // kad::Event::UnroutablePeer { peer } => {
            //     error!("Peer {} is unroutable", peer);
            // }
            // kad::Event::RoutablePeer { peer, address } => todo!(),
            // kad::Event::PendingRoutablePeer { peer, address } => todo!(),
            kad::Event::ModeChanged { new_mode } => {
                info!("Kademlia mode changed to {}", new_mode);
            }
            _ => {}
        }
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

#[derive(Debug)]
pub(crate) enum BootstrapMode {
    WithNodes(BootNodes),
    WithoutNodes,
}
