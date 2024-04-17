use libp2p::PeerId;

#[derive(Debug)]
pub(crate) enum Request {
    Listeners,
    ConnectedPeers,
    ConnectedTo { peer_id: PeerId },
}
