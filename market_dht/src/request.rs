use libp2p::{Multiaddr, PeerId};
use tokio::sync::oneshot::{self, error::RecvError};

#[derive(Debug)]
pub(crate) struct ResponseHandler {
    inner: oneshot::Receiver<ResponseData>,
}

impl ResponseHandler {
    pub(crate) async fn get_response_data(self) -> Result<ResponseData, RecvError> {
        self.inner.await
    }
}

#[derive(Debug)]
pub(crate) struct RequestHandler {
    pub(crate) request_data: RequestData,
    inner: oneshot::Sender<ResponseData>,
}

impl RequestHandler {
    pub(crate) fn new(request_data: RequestData) -> (Self, ResponseHandler) {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
        (
            Self {
                request_data,
                inner: response_sender,
            },
            ResponseHandler {
                inner: response_receiver,
            },
        )
    }

    pub(crate) fn respond(self, response: ResponseData) {
        self.inner
            .send(response)
            .expect("it to send since oneshot client should not have dropped")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub(crate) enum RequestData {
    GetAllListeners,
    GetConnectedPeers,
    IsConnectedTo(PeerId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResponseData {
    // NOTE: the vec is useful for now when we add functionality for users being able to add
    // listeners?
    GetAllListeners { listeners: Vec<Multiaddr> },
    GetConnectedPeers,
    IsConnectedTo { is_connected: bool },
}
