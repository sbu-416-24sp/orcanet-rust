use libp2p::identity::Keypair;
use libp2p::PeerId;
use proto::market::FileInfo;
use proto::market::User;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::command::Message;
use crate::FailureResponse;
use crate::FileInfoHash;
use crate::{command::request::Request, Response};

#[derive(Debug)]
pub struct Peer {
    peer_id: PeerId,
    sender: mpsc::UnboundedSender<Message>,
    keypair: Keypair,
}

impl Peer {
    #[inline(always)]
    pub(crate) const fn new(
        peer_id: PeerId,
        sender: mpsc::UnboundedSender<Message>,
        keypair: Keypair,
    ) -> Self {
        Self {
            peer_id,
            sender,
            keypair,
        }
    }

    #[inline(always)]
    pub const fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    #[inline(always)]
    pub const fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    #[inline(always)]
    async fn send(&self, request: Request) -> Response {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send((request, tx))
            .map_err(|err| FailureResponse::SendError(err.to_string()))?;
        rx.await.map_err(FailureResponse::RecvError)?
    }

    #[inline(always)]
    pub async fn listeners(&self) -> Response {
        self.send(Request::Listeners).await
    }

    #[inline(always)]
    pub async fn connected_peers(&self) -> Response {
        self.send(Request::ConnectedPeers).await
    }

    #[inline(always)]
    pub async fn connected_to(&self, peer_id: PeerId) -> Response {
        self.send(Request::ConnectedTo { peer_id }).await
    }

    #[inline(always)]
    pub async fn check_holders(&self, file_info: impl Into<FileInfoHash>) -> Response {
        todo!()
    }

    #[inline(always)]
    pub async fn register_file(
        &self,
        user: impl Into<User>,
        fileinfo: impl Into<FileInfo>,
    ) -> Response {
        todo!()
    }
}
