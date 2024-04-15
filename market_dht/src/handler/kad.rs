use libp2p::{kad::Event, Swarm};

use crate::{behaviour::Behaviour, lmm::LocalMarketMap};

use super::EventHandler;

pub(crate) struct KadHandler<'a, 'b> {
    swarm: &'a mut Swarm<Behaviour>,
    lmm: &'b mut LocalMarketMap,
}

impl<'a, 'b> KadHandler<'a, 'b> {
    pub(crate) fn new(swarm: &'a mut Swarm<Behaviour>, lmm: &'b mut LocalMarketMap) -> Self {
        KadHandler { swarm, lmm }
    }
}

impl<'a, 'b> EventHandler for KadHandler<'a, 'b> {
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
