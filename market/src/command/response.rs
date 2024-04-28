use libp2p::{Multiaddr, PeerId};
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

pub type Response = Result<SuccessfulResponse, FailureResponse>;

#[derive(Debug)]
pub enum SuccessfulResponse {
    Listeners { listeners: Vec<Multiaddr> },
    ConnectedPeers { peers: Vec<PeerId> },
    ConnectedTo { connected: bool },
    KadResponse(KadSuccessfulResponse),
}

#[derive(Debug, Error)]
pub enum FailureResponse {
    #[error("Failed to send request: {0}")]
    SendError(String),
    #[error("Failed to receive response: {0}")]
    RecvError(#[from] RecvError),
    #[error("[Kademlia Error] - {0}")]
    KadError(KadFailureResponse),
}

#[derive(Debug)]
pub enum KadSuccessfulResponse {
    GetClosestPeers { peers: Vec<PeerId> },
}

#[derive(Debug, Error)]
pub enum KadFailureResponse {
    #[error("Failed to get closest peers: {error}")]
    GetClosestPeers { key: Vec<u8>, error: String },
}
