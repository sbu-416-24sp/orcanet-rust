use std::{borrow::Cow, net::Ipv4Addr};

use market_dht::{peer::Peer, FileReqResResponseData, ResponseData};
use market_proto::market_proto_rpc::{
    market_server::Market, CheckHoldersRequest, HoldersResponse, RegisterFileRequest, User,
};
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct MarketService {
    peer: Peer,
}

impl MarketService {
    pub fn new(peer: Peer) -> Self {
        MarketService { peer }
    }
}

#[tonic::async_trait]
impl Market for MarketService {
    async fn register_file(
        &self,
        request: Request<RegisterFileRequest>,
    ) -> Result<Response<()>, Status> {
        let file_req = request.into_inner();

        let file_hash = file_req.file_hash.as_bytes().to_vec();
        let user = file_req.user.ok_or(Status::invalid_argument(
            "The user field is required for this request",
        ))?;
        let ip = user
            .ip
            .as_str()
            .parse::<Ipv4Addr>()
            .map_err(|err| Status::internal(format!("Internal Server Error: {}", err)))?;
        // NOTE: please make the proto port a u16
        let port: u16 = user
            .port
            .try_into()
            .map_err(|err| Status::internal(format!("Internal Server Error: {}", err)))?;
        let _res = self
            .peer
            .register_file(Cow::Owned(file_hash), ip, port, user.price, user.name)
            .await
            .map_err(|err| Status::internal(format!("Internal Server Error: {}", err)))?;
        Ok(Response::new(()))
    }

    async fn check_holders(
        &self,
        request: Request<CheckHoldersRequest>,
    ) -> Result<Response<HoldersResponse>, Status> {
        // NOTE: Please make the file_hash not a String but a Vec<u8>
        let holders_req = request.into_inner();
        let file_hash = Cow::Owned(holders_req.file_hash.as_bytes().to_vec());
        if let ResponseData::ReqResResponse(FileReqResResponseData::GetSuppliers { suppliers }) =
            self.peer
                .check_holders(file_hash)
                .await
                .map_err(|err| Status::internal(format!("Internal Server Error: {}", err)))?
        {
            let holders = suppliers
                .into_iter()
                .map(|(peer_id, supplier_info)| {
                    User::new(
                        peer_id.to_string(),
                        supplier_info.username,
                        supplier_info.ip.to_string(),
                        supplier_info.port as i32,
                        supplier_info.price,
                    )
                })
                .collect::<Vec<_>>();
            Ok(Response::new(HoldersResponse { holders }))
        } else {
            Err(Status::internal(
                "Did not get the right response for some reason...",
            ))
        }
    }
}
