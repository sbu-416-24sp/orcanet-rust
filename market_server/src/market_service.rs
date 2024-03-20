use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use market_proto::market_proto_rpc::{
    market_server::Market, CheckHoldersRequest, HoldersResponse, RegisterFileRequest, User,
};
use tonic::{Request, Response, Status};

// TODO: replace this with a DHT
type MarketStore = HashMap<String, HashSet<User>>;

#[derive(Debug, Clone, Default)]
pub struct MarketService {
    store: Arc<Mutex<MarketStore>>,
}

#[tonic::async_trait]
impl Market for MarketService {
    async fn register_file(
        &self,
        request: Request<RegisterFileRequest>,
    ) -> Result<Response<()>, Status> {
        let file_req = request.into_inner();
        let mut store = self
            .store
            .lock()
            .map_err(|err| Status::internal(format!("Internal Server Error: {}", err)))?;

        let file_hash = file_req.file_hash;
        let user = file_req.user.ok_or(Status::invalid_argument(
            "The user field is required for this request",
        ))?;
        let entry = store.entry(file_hash).or_default();
        // Only on the assumption that user IDs are unique. We have defined the hasher in the gRPC
        // to have it where the user is uniquely identified by the hash of the ID argument they
        // pass in.
        entry.insert(user);
        Ok(Response::new(()))
    }

    async fn check_holders(
        &self,
        request: Request<CheckHoldersRequest>,
    ) -> Result<Response<HoldersResponse>, Status> {
        let holders_req = request.into_inner();
        let file_hash = holders_req.file_hash;
        let store = self
            .store
            .lock()
            .map_err(|err| Status::internal(format!("Internal Server Error: {}", err)))?;

        let holders = store.get(&file_hash).ok_or(Status::not_found(format!(
            "{file_hash} does not exist in the table!"
        )))?;
        Ok(Response::new(HoldersResponse {
            holders: holders.iter().cloned().collect(),
        }))
    }
}
