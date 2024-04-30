use libp2p::identity::Keypair;
use libp2p::PeerId;
use proto::market::FileInfo;
use proto::market::HoldersResponse;
use proto::market::User;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::command::request::KadRequest;
use crate::command::request::LmmRequest;
use crate::command::request::ReqResRequest;
use crate::command::Message;
use crate::FailureResponse;
use crate::FileInfoHash;
use crate::FileResponse;
use crate::KadSuccessfulResponse;
use crate::LmmSuccessfulResponse;
use crate::ReqResSuccessfulResponse;
use crate::SuccessfulResponse;
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
    pub async fn get_closest_peers(&self, key: impl Into<Vec<u8>>) -> Response {
        self.send(Request::Kad(KadRequest::GetClosestPeers {
            key: key.into(),
        }))
        .await
    }

    #[inline(always)]
    pub async fn is_local_file_owner(&self, file_info_hash: impl Into<FileInfoHash>) -> bool {
        if let Ok(SuccessfulResponse::LmmResponse(LmmSuccessfulResponse::IsLocalFileOwner {
            is_owner,
        })) = self
            .send(Request::LocalMarketMap(LmmRequest::IsLocalFileOwner {
                file_info_hash: file_info_hash.into(),
            }))
            .await
        {
            is_owner
        } else {
            panic!("This should never run since there is no error ever sent back.")
        }
    }

    #[inline(always)]
    pub async fn get_holder_by_peer_id(
        &self,
        peer_id: PeerId,
        file_info_hash: impl Into<FileInfoHash>,
    ) -> Response {
        self.send(Request::ReqRes(ReqResRequest::GetHolderByPeerId {
            peer_id,
            file_info_hash: file_info_hash.into(),
        }))
        .await
    }

    pub async fn get_providers(&self, file_info_hash: impl Into<FileInfoHash>) -> Response {
        let file_info_hash: FileInfoHash = file_info_hash.into();
        let is_local_file_owner = self.is_local_file_owner(file_info_hash.clone()).await;
        let mut res = self
            .send(Request::Kad(KadRequest::GetProviders { file_info_hash }))
            .await;
        if let Ok(SuccessfulResponse::KadResponse(KadSuccessfulResponse::GetProviders {
            ref mut providers,
        })) = res
        {
            if is_local_file_owner && !providers.contains(self.peer_id()) {
                providers.push(*self.peer_id());
            }
        }
        res
    }

    #[inline(always)]
    pub async fn check_holders(&self, file_info_hash: impl Into<FileInfoHash>) -> Response {
        let file_info_hash: FileInfoHash = file_info_hash.into();
        let res = self.get_providers(file_info_hash.clone()).await;
        if let Ok(SuccessfulResponse::KadResponse(KadSuccessfulResponse::GetProviders {
            providers,
        })) = res
        {
            let mut holders = Vec::new();
            let mut file_info = None;
            for provider in providers {
                // TODO: can optimize this but lazy for now
                let maybe_holder = self
                    .get_holder_by_peer_id(provider, file_info_hash.clone())
                    .await;
                if let Ok(SuccessfulResponse::ReqResResponse(
                    ReqResSuccessfulResponse::GetHolderByPeerId {
                        holder: FileResponse::HasFile(holder),
                    },
                )) = maybe_holder
                {
                    if file_info.is_none() {
                        file_info = Some(holder.file_info);
                    }
                    holders.push(holder.user);
                }
            }
            Ok(SuccessfulResponse::CheckHolders(HoldersResponse {
                file_info,
                holders,
            }))
        } else {
            res
        }
    }

    #[inline(always)]
    pub async fn register_file(
        &self,
        user: impl Into<User>,
        file_info_hash: impl Into<FileInfoHash>,
        file_info: impl Into<FileInfo>,
    ) -> Response {
        self.send(Request::Kad(KadRequest::RegisterFile {
            file_info_hash: file_info_hash.into(),
            file_info: file_info.into(),
            user: user.into(),
        }))
        .await
    }
}
