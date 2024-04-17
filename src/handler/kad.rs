use libp2p::{kad::Event, Swarm};

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
}

impl<'a> EventHandler for KadHandler<'a> {
    type Event = Event;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::InboundRequest { request } => todo!(),
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
