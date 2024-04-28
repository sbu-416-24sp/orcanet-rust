use libp2p::{
    kad::{Event, InboundRequest},
    Swarm,
};
use log::{info, warn};

use crate::{behaviour::Behaviour, command::QueryHandler, lmm::LocalMarketMap};

use super::EventHandler;

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

    fn handle_inbound_request(&mut self, request: InboundRequest) {
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
}

impl<'a> EventHandler for KadHandler<'a> {
    type Event = Event;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::InboundRequest { request } => self.handle_inbound_request(request),
            Event::OutboundQueryProgressed {
                id,
                result,
                stats,
                step,
            } => todo!(),
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
                warn!("[Kademlia] - Peer {peer} has the following address: {address}");
            }
            Event::ModeChanged { new_mode } => {
                warn!("[Kademlia] - Mode changed to {new_mode}");
            }
            _ => {}
        }
    }
}
