use tokio::sync::mpsc;

use crate::request::Request;
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
}
