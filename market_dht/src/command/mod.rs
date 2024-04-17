use self::request::Request;
use crate::{
    handler::{send_err, send_ok},
    Response,
};
use log::error;
use std::collections::HashMap;
use tokio::sync::oneshot;

pub(crate) type Message = (Request, oneshot::Sender<Response>);

#[derive(Debug, Default)]
pub(crate) struct QueryHandler {
    inner: HashMap<Query, oneshot::Sender<Response>>,
}

impl QueryHandler {
    pub(crate) fn add_query(&mut self, query: Query, responder: oneshot::Sender<Response>) {
        self.inner.insert(query, responder);
    }

    pub(crate) fn respond(&mut self, query: Query, response: Response) {
        let responder = self.inner.remove(&query);
        if let Some(responder) = responder {
            match response {
                Ok(success) => {
                    send_ok!(responder, success);
                }
                Err(failure) => {
                    send_err!(responder, failure);
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum Query {}

pub(crate) mod request;
pub(crate) mod response;
