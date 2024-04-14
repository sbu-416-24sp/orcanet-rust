mod db;
pub mod files;
mod http;

use crate::grpc::MarketClient;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

pub async fn start_server(
    files: HashMap<String, PathBuf>,
    prices: HashMap<String, i64>,
    port: String,
) -> tokio::task::JoinHandle<()> {
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

pub async fn register_files(
    prices: HashMap<String, i64>,
    market: String,
    port: String,
) -> Result<()> {
    let mut client = MarketClient::new(market).await?;

    // get port from string
    let port = match port.parse::<i32>() {
        Ok(port) => port,
        Err(_) => {
            eprintln!("Invalid port number");
            return Ok(());
        }
    };

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
                port,
                price,
                hash,
            )
            .await?;
    }

    Ok(())
}
