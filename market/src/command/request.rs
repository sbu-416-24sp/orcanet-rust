use libp2p::{kad::QueryId, request_response::OutboundRequestId, PeerId};
use proto::market::{FileInfo, FileInfoHash, User};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum Query {
    Kad(QueryId),
    ReqRes(OutboundRequestId),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum Request {
    Listeners,
    ConnectedPeers,
    ConnectedTo { peer_id: PeerId },
    Kad(KadRequest),
    LocalMarketMap(LmmRequest),
    ReqRes(ReqResRequest),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum KadRequest {
    GetClosestPeers {
        key: Vec<u8>,
    },
    RegisterFile {
        file_info_hash: FileInfoHash,
        file_info: FileInfo,
        user: User,
    },
    GetProviders {
        file_info_hash: FileInfoHash,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum ReqResRequest {
    GetHolderByPeerId {
        peer_id: PeerId,
        file_info_hash: FileInfoHash,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum LmmRequest {
    IsLocalFileOwner { file_info_hash: FileInfoHash },
}
