use libp2p::{request_response::Event, Swarm};

use crate::{
    behaviour::Behaviour,
    command::QueryHandler,
    lmm::{FileInfoHash, SupplierInfo, LocalMarketMap},
};

use super::EventHandler;

pub(super) struct ReqResHandler<'a> {
    swarm: &'a mut Swarm<Behaviour>,
    lmm: &'a mut LocalMarketMap,
    query_handler: &'a mut QueryHandler,
}

impl<'a> ReqResHandler<'a> {
    pub(super) fn new(
        swarm: &'a mut Swarm<Behaviour>,
        lmm: &'a mut LocalMarketMap,
        query_handler: &'a mut QueryHandler,
    ) -> Self {
        ReqResHandler {
            swarm,
            lmm,
            query_handler,
        }
    }
}

impl<'a> EventHandler for ReqResHandler<'a> {
    type Event = Event<FileInfoHash, SupplierInfo>;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::Message { peer, message } => todo!(),
            Event::OutboundFailure {
                peer,
                request_id,
                error,
            } => todo!(),
            Event::InboundFailure {
                peer,
                request_id,
                error,
            } => todo!(),
            Event::ResponseSent { peer, request_id } => todo!(),
        }
    }
}
