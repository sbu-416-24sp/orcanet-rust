use libp2p::{request_response::Event, Swarm};

use crate::{
    behaviour::Behaviour,
    command::{
        request::{Query, ReqResRequest},
        QueryHandler,
    },
    lmm::{FileInfoHash, FileResponse, LocalMarketMap},
};

use super::{CommandRequestHandler, EventHandler};

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
    type Event = Event<FileInfoHash, FileResponse>;

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

impl<'a> CommandRequestHandler for ReqResHandler<'a> {
    type Request = ReqResRequest;

    fn handle_command(
        &mut self,
        request: Self::Request,
        responder: tokio::sync::oneshot::Sender<crate::Response>,
    ) {
        match request {
            ReqResRequest::GetHolderByPeerId {
                peer_id,
                file_info_hash,
            } => {
                let qid = self
                    .swarm
                    .behaviour_mut()
                    .req_res
                    .send_request(&peer_id, file_info_hash);
                self.query_handler.add_query(Query::ReqRes(qid), responder);
            }
        }
    }
}
