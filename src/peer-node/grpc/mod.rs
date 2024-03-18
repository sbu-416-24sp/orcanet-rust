use tonic::transport::Channel;

use orcanet::market_client::MarketClient as MarketServiceClient;
use orcanet::{RegisterFileRequest, User, CheckHoldersRequest, HoldersResponse};

use anyhow::Result;

pub mod orcanet {
    tonic::include_proto!("market");
}

pub struct MarketClient {
    client: MarketServiceClient<Channel>,
}

impl MarketClient {
    // Initialize a new MarketClient, connecting to the given market service address
    pub async fn new(market: String) -> Result<Self> {
        println!("gRPC: Connecting to market service at {}...", market);
        let client = MarketServiceClient::connect(format!("http://{}", market)).await?;

        Ok(MarketClient { client })
    }

    // Get a list of producers for a given file hash
    pub async fn check_holders(&mut self, file_hash: String) -> Result<HoldersResponse> {
        println!("gRPC: Checking holders for file hash {}", file_hash);
        let request = CheckHoldersRequest {
            file_hash,
        };

        let response = self.client.check_holders(request).await?.into_inner();

        Ok(response)
    }

    // Register a new producer
    pub async fn register_file(&mut self, id: String, name: String, ip: String, port: i32, price: i64, file_hash: String) -> Result<()> {
        let user = User {
            id,
            name,
            ip,
            port,
            price
        };
        let file = RegisterFileRequest {
            user: Some(user),
            file_hash: file_hash.clone(),
        };

        self.client.register_file(file).await?;
        println!("gRPC: Registered producer for file hash {}", file_hash);

        Ok(())
    }
}