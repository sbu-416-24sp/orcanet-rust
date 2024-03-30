use tokio::sync::mpsc;

use crate::req_res::{Request, RequestData, RequestHandler, Response};
use crate::PeerId;

#[derive(Debug)]
pub struct Peer {
    id: PeerId,
    sender: mpsc::UnboundedSender<Request>,
}

impl Peer {
    pub(crate) const fn new(sender: mpsc::UnboundedSender<Request>, id: PeerId) -> Self {
        Peer { sender, id }
    }

    pub const fn id(&self) -> &PeerId {
        &self.id
    }

    pub async fn is_connected_to(&self, peer_id: PeerId) -> Response {
        self.send_request(RequestData::IsConnectedTo(peer_id)).await
    }

    pub async fn get_all_listeners(&self) -> Response {
        self.send_request(RequestData::GetAllListeners).await
    }

    pub async fn get_connected_peers(&self) -> Response {
        self.send_request(RequestData::GetConnectedPeers).await
    }

    async fn send_request(&self, request_data: RequestData) -> Response {
        let (request_handler, response_handler) = RequestHandler::new();
        self.sender.send((request_data, request_handler))?;
        response_handler.get_response_data().await
    }
}
