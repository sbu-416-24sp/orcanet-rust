use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tonic::{transport::Server, Request, Response, Status};

use orcanet::market_service_server::{MarketService, MarketServiceServer};
use orcanet::{FileProducer, FileHash, FileProducerList};
use clap::Parser;

pub mod orcanet {
    tonic::include_proto!("orcanet");
}

/// Mock version of the market service
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "50051")]
    port: u16,
}

#[derive(Debug, Default)]
struct StoredProducer{
    link: String,
    price: f32,
    payment_address: String,
}

#[derive(Debug, Default)]
pub struct Market {
    // Map of file hashes to producers
    producers_map: Arc<Mutex<HashMap<String, Vec<StoredProducer>>>>,
}

#[tonic::async_trait]
impl MarketService for Market {
    // Implement RPC to get producers for a hashed file
    async fn get_producers(&self, request: Request<FileHash>) -> Result<Response<FileProducerList>, Status> {
        let file_hash = request.into_inner();
        
        // Lock the producers map
        let producers_map = self.producers_map.lock().unwrap();
        
        // Get the producers for the file
        if let Some(producers) = producers_map.get(&file_hash.hash) {
            let file_producers = producers.iter().map(|producer| {
                FileProducer {
                    hash: file_hash.hash.clone(),
                    link: producer.link.clone(),
                    price: producer.price,
                    payment_address: producer.payment_address.clone(),
                }
            }).collect();

            let response = FileProducerList {
                producers: file_producers,
            };

            Ok(Response::new(response))
        } else {
            Err(Status::not_found("File not found"))
        }
    }

    // Implement RPC to add a producer for a file
    async fn add_producer(&self, request: Request<FileProducer>) -> Result<Response<()>, Status> {
        let producer = request.into_inner();
        let file_hash = producer.hash;

        // Lock the producers map
        let mut producers_map = self.producers_map.lock().unwrap();

        // Add the producer to the map
        if let Some(producers) = producers_map.get_mut(&file_hash) {
            producers.push(StoredProducer {
                link: producer.link,
                price: producer.price,
                payment_address: producer.payment_address,
            });
        } else {
            producers_map.insert(file_hash, vec![StoredProducer {
                link: producer.link,
                price: producer.price,
                payment_address: producer.payment_address,
            }]);
        }

        Ok(Response::new(()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command line arguments
    let args = Args::parse();
    
    // Start the server
    let market = Market::default();
    let addr = format!("[::]:{}", args.port).parse()?;
    println!("Market server listening on {}", addr);
    Server::builder()
        .add_service(MarketServiceServer::new(market))
        .serve(addr)
        .await?;

    Ok(())
}
