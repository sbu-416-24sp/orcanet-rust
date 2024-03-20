use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use market_proto::market_proto_rpc::{
    market_server::Market, CheckHoldersRequest, HoldersResponse, RegisterFileRequest, User,
};
use tonic::{Response, Status};

type MarketStore = HashMap<String, HashSet<User>>;

#[derive(Debug, Clone)]
pub struct MarketServer {
    store: Arc<Mutex<MarketStore>>,
}

#[tonic::async_trait]
impl Market for MarketServer {
    async fn register_file(
        &self,
        request: tonic::Request<RegisterFileRequest>,
    ) -> std::result::Result<Response<()>, Status> {
        let file_req = request.into_inner();
        let mut store = self.store.lock().map_err(|err| {
            Status::new(
                tonic::Code::Internal,
                format!("Internal Server Error: {}", err),
            )
        })?;

        let file_hash = file_req.file_hash;
        let user = file_req.user.ok_or(tonic::Status::invalid_argument(
            "The user field is required for this request",
        ))?;
        let entry = store.entry(file_hash).or_default();
        entry.insert(user);
        Ok(Response::new(()))
    }

    async fn check_holders(
        &self,
        request: tonic::Request<CheckHoldersRequest>,
    ) -> std::result::Result<Response<HoldersResponse>, Status> {
        let holders_req = request.into_inner();
        let file_hash = holders_req.file_hash;
        let store = self.store.lock().map_err(|err| {
            Status::new(
                tonic::Code::Internal,
                format!("Internal Server Error: {}", err),
            )
        })?;

        let holders = store
            .get(&file_hash)
            .ok_or(Status::not_found(format!("{file_hash} not found")))?;
        Ok(Response::new(HoldersResponse {
            holders: holders.iter().cloned().collect(),
        }))
    }
}

fn main() {
    println!("Hello, world!");
}
