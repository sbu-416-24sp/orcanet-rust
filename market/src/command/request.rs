use libp2p::{kad::QueryId, request_response::OutboundRequestId, PeerId};

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum Query {
    Kad(QueryId),
    ReqRes(OutboundRequestId),
}

#[derive(Debug)]
pub(crate) enum Request {
    Listeners,
    ConnectedPeers,
    ConnectedTo { peer_id: PeerId },
}
