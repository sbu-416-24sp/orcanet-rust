use std::borrow::Cow;
use std::net::Ipv4Addr;

use tokio::sync::mpsc;

use crate::behaviour::file_req_res::{FileMetadata, SupplierInfo};
use crate::req_res::{KadRequestData, Request, RequestData, RequestHandler, Response};
use crate::PeerId;

use self::macros::send;

#[derive(Debug)]
pub struct Peer {
    id: PeerId,
    sender: mpsc::UnboundedSender<Request>,
}

impl Peer {
    #[inline(always)]
    pub(crate) const fn new(sender: mpsc::UnboundedSender<Request>, id: PeerId) -> Self {
        Peer { sender, id }
    }

    #[inline(always)]
    pub const fn id(&self) -> &PeerId {
        &self.id
    }

    #[inline(always)]
    pub async fn is_connected_to(&self, peer_id: PeerId) -> Response {
        send!(self, RequestData::IsConnectedTo(peer_id))
    }

    #[inline(always)]
    pub async fn get_all_listeners(&self) -> Response {
        send!(self, RequestData::GetAllListeners)
    }

    #[inline(always)]
    pub async fn get_connected_peers(&self) -> Response {
        send!(self, RequestData::GetConnectedPeers)
    }

    #[inline(always)]
    pub async fn get_closest_local_peers(&self, key: Cow<'_, Vec<u8>>) -> Response {
        let key = get_owned_key(key);
        send!(
            self,
            RequestData::KadRequest(KadRequestData::ClosestLocalPeers { key })
        )
    }

    #[inline(always)]
    pub async fn get_closest_peers(&self, key: Cow<'_, Vec<u8>>) -> Response {
        let key = get_owned_key(key);
        send!(
            self,
            RequestData::KadRequest(KadRequestData::ClosestPeers { key })
        )
    }

    #[inline(always)]
    pub async fn register_file(
        &self,
        file_hash: Cow<'_, Vec<u8>>,
        ip: impl Into<Ipv4Addr>,
        port: u16,
        price: i32,
        username: String,
    ) -> Response {
        // NOTE: the price is i32 because the protobuf file specified i32 for some reason
        let file_hash = get_owned_key(file_hash);
        let supplier_info = SupplierInfo {
            ip: ip.into(),
            port,
            price,
            username,
        };
        let file_metadata = FileMetadata {
            file_hash,
            supplier_info,
        };
        send!(
            self,
            RequestData::KadRequest(KadRequestData::RegisterFile { file_metadata })
        )
    }

    #[inline(always)]
    pub async fn check_holders(&self, file_hash: Cow<'_, Vec<u8>>) -> Response {
        let file_hash = get_owned_key(file_hash);
        send!(
            self,
            RequestData::KadRequest(KadRequestData::GetProviders { key: file_hash })
        )
    }

    #[inline(always)]
    async fn send_request(&self, request_data: RequestData) -> Response {
        let (request_handler, response_handler) = RequestHandler::new();
        self.sender.send((request_data, request_handler))?;
        response_handler.get_response_data().await
    }
}

#[inline(always)]
fn get_owned_key(key: Cow<'_, Vec<u8>>) -> Vec<u8> {
    match key {
        Cow::Borrowed(bo) => bo.to_owned(),
        Cow::Owned(owned) => owned,
    }
}

mod macros {
    macro_rules! send {
        ($self: ident, $request:expr) => {
            $self.send_request($request).await
        };
    }
    pub(super) use send;
}
