use libp2p::{kad::QueryId, request_response::OutboundRequestId, PeerId};

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
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum KadRequest {
    GetClosestPeers { key: Vec<u8> },
}
