use anyhow::Result;
use tokio::sync::mpsc;

use crate::request::{RequestData, RequestHandler, ResponseData};
use crate::PeerId;

#[derive(Debug)]
pub struct Peer {
    id: PeerId,
    sender: mpsc::UnboundedSender<RequestHandler>,
}

impl Peer {
    pub(crate) const fn new(sender: mpsc::UnboundedSender<RequestHandler>, id: PeerId) -> Self {
        Peer { sender, id }
    }

    pub const fn id(&self) -> &PeerId {
        &self.id
    }

    pub async fn is_connected_to(&self, peer_id: PeerId) -> Result<ResponseData> {
        self.send_request(RequestData::IsConnectedTo(peer_id)).await
    }

    pub async fn get_all_listeners(&self) -> Result<ResponseData> {
        self.send_request(RequestData::GetAllListeners).await
    }

    pub async fn get_connected_peers(&self) -> Result<ResponseData> {
        self.send_request(RequestData::GetConnectedPeers).await
    }

    async fn send_request(&self, request_data: RequestData) -> Result<ResponseData> {
        let (request_handler, response_handler) = RequestHandler::new(request_data);
        self.sender.send(request_handler)?;
        Ok(response_handler.get_response_data().await?)
    }
}
