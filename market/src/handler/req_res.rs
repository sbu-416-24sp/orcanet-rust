use libp2p::{
    request_response::{Event, Message},
    Swarm,
};
use log::{error, info, warn};

use crate::{
    behaviour::Behaviour,
    command::{
        request::{Query, ReqResRequest},
        QueryHandler,
    },
    handler::send_ok,
    lmm::{FileInfoHash, FileResponse, LocalMarketMap},
    FailureResponse, ReqResFailureResponse, ReqResSuccessfulResponse, SuccessfulResponse,
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
            Event::Message { peer, message } => match message {
                Message::Request {
                    request_id,
                    request,
                    channel,
                } => {
                    info!(
                        "[RequestResponse {request_id:?}] - Received request from {}",
                        peer
                    );
                    let response = {
                        if let Some(holder) = self.lmm.get_if_not_expired(&request) {
                            info!("[RequestResponse {request_id:?}] - Found holder for file");
                            FileResponse::HasFile(holder)
                        } else {
                            warn!("[RequestResponse {request_id:?}] - No holder found for file");
                            FileResponse::NoFile
                        }
                    };

                    if self
                        .swarm
                        .behaviour_mut()
                        .req_res
                        .send_response(channel, response)
                        .is_err()
                    {
                        error!(
                                "[RequestResponse {request_id:?}] - Failed to send response to {peer}. Could be timeout or channel closed.",
                            );
                    }
                }
                Message::Response {
                    request_id,
                    response,
                } => {
                    info!(
                        "[RequestResponse {request_id:?}] - Received response from {}",
                        peer
                    );
                    self.query_handler.respond(
                        Query::ReqRes(request_id),
                        Ok(SuccessfulResponse::ReqResResponse(
                            ReqResSuccessfulResponse::GetHolderByPeerId { holder: response },
                        )),
                    );
                }
            },
            Event::OutboundFailure {
                peer,
                request_id,
                error,
            } => {
                error!(
                    "[RequestResponse {request_id:?}] - Outbound request failure to peer: {}",
                    peer
                );
                self.query_handler.respond(
                    Query::ReqRes(request_id),
                    Err(FailureResponse::ReqResError(
                        ReqResFailureResponse::GetHolderByPeerId {
                            error: error.to_string(),
                        },
                    )),
                );
            }
            Event::InboundFailure {
                peer,
                request_id,
                error,
            } => {
                error!(
                    "[RequestResponse {request_id:?}] - Inbound request failure by trying to retrieve from peer: {}",
                    peer
                );
                error!("[RequestResponse {request_id:?}] - Error: {}", error);
            }
            Event::ResponseSent { peer, request_id } => {
                warn!("[RequestResponse {request_id:?}] - Response sent to peer: {peer}");
            }
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
                if &peer_id == self.swarm.local_peer_id() {
                    info!("[RequestResponse] - Requesting file from self");
                    let response = {
                        if let Some(holder) = self.lmm.get_if_not_expired(&file_info_hash) {
                            info!("[RequestResponse] - Found holder for file from self");
                            FileResponse::HasFile(holder)
                        } else {
                            warn!("[RequestResponse] - No holder found for file from self");
                            FileResponse::NoFile
                        }
                    };
                    send_ok!(
                        responder,
                        SuccessfulResponse::ReqResResponse(
                            ReqResSuccessfulResponse::GetHolderByPeerId { holder: response }
                        )
                    );
                } else {
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
}
