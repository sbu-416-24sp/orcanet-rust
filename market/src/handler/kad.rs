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

    pub(crate) fn handle_inbound_request(&mut self, request: InboundRequest) {
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
                bucket_range,
                old_peer,
            } => todo!(),
            Event::UnroutablePeer { peer } => todo!(),
            Event::RoutablePeer { peer, address } => todo!(),
            Event::PendingRoutablePeer { peer, address } => todo!(),
            Event::ModeChanged { new_mode } => todo!(),
        }
    }
}
