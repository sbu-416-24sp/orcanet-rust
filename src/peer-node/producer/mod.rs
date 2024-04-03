mod db;
pub mod files;
mod http;

use crate::grpc::orcanet::market_client;
use crate::grpc::MarketClient;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

pub async fn run(market: String) -> Result<()> {
    let mut client = MarketClient::new(market).await?;

    // Load the files
    let file_map = Arc::new(files::FileMap::default());
    file_map.add_dir("files/**/*", 100).await?;

    // Launch the HTTP server in the background
    let http_file_map = file_map.clone();
    tokio::spawn(async move {
        if let Err(e) = http::run(http_file_map, 8080).await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    // Register the files with the market service
    let hash = file_map.get_hashes().await;
    for hash in hash {
        // TODO: Find a way to get the public IP address
        println!("Producer: Registering file with hash {}", hash);
        client
            .register_file(
                "id".to_string(),
                "name".to_string(),
                "127.0.0.1".to_string(),
                8080,
                100,
                hash,
            )
            .await?;
    }

    // Never return
    tokio::signal::ctrl_c().await?;

    Ok(())
}

// pub async fn get_file_map(files: HashMap<String, i64>) -> Result<Arc<files::FileMap>> {
//     let file_map = Arc::new(files::FileMap::new(files).await);
//     Ok(file_map)
// }

pub async fn start_server(files: HashMap<String, PathBuf>, prices: HashMap<String, i64>, port: u16) -> tokio::task::JoinHandle<()> {
    // Launch the HTTP server in the background
    let http_file_map = Arc::new(files::FileMap::new(files, prices));
    tokio::spawn(async move {
        if let Err(e) = http::run(http_file_map, port).await {
            eprintln!("HTTP server error: {}", e);
        }
    })
}

pub async fn stop_server(join_handle: tokio::task::JoinHandle<()>) -> Result<()> {
    // Stop the HTTP server
    join_handle.abort();
    Ok(())
}

pub async fn create_market_client(market: String) -> Result<MarketClient> {
    let client = MarketClient::new(market).await?;
    Ok(client)
}

pub async fn register_files(
    prices: HashMap<String, i64>,
    market: String,
) -> Result<()> {
    let mut client = MarketClient::new(market).await?;

    for (hash, price) in prices {
        // TODO: Find a way to get the public IP address
        println!(
            "Producer: Registering file with hash {} and price {}",
            hash, price
        );
        client
            .register_file(
                "id".to_string(),
                "name".to_string(),
                "127.0.0.1".to_string(),
                8080,
                price,
                hash,
            )
            .await?;
    }

    Ok(())
}
