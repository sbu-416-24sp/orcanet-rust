use tokio::sync::mpsc;

use crate::request::Request;

#[derive(Debug)]
pub struct Peer {
    sender: mpsc::UnboundedSender<Request>,
}

impl Peer {
    pub(crate) fn new(sender: mpsc::UnboundedSender<Request>) -> Self {
        Peer { sender }
    }
}
