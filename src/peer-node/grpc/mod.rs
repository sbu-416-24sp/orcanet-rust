use tonic::transport::Channel;

use orcanet::market_service_client::MarketServiceClient;
use orcanet::{FileHash, FileProducerList, FileProducer};

use anyhow::Result;

pub mod orcanet {
    tonic::include_proto!("orcanet");
}

pub struct MarketClient {
    client: MarketServiceClient<Channel>,
}

impl MarketClient {
    // Initialize a new MarketClient, connecting to the given market service address
    pub async fn new(market: String) -> Result<Self> {
        println!("Connecting to market service at {}...", market);
        let client = MarketServiceClient::connect(format!("http://{}", market)).await?;

        Ok(MarketClient { client })
    }

    // Get a list of producers for a given file hash
    pub async fn get_producers(&mut self, file_hash: String) -> Result<FileProducerList> {
        let request = FileHash {
            hash: file_hash,
        };

        let response = self.client.get_producers(request).await?.into_inner();

        Ok(response)
    }

    // Register a new producer
    pub async fn register_producer(&mut self, hash: String, link: String, price: f32, payment_address: String) -> Result<()> {
        let producer = FileProducer {
            hash: hash.clone(),
            link,
            price,
            payment_address,
        };

        self.client.add_producer(producer).await?;
        println!("Registered producer for file hash {}", hash);

        Ok(())
    }
}