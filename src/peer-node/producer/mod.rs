mod db;
mod files;
mod http;

use std::sync::Arc;

use crate::grpc::MarketClient;

use anyhow::Result;

pub async fn run(market: String) -> Result<()> {
    let mut client = MarketClient::new(market).await?;

    // Load the files
    let file_map = Arc::new(files::FileMap::new());
    file_map.add_all("files/**/*").await?;

    // Launch the HTTP server in the background
    let http_file_map = file_map.clone();
    tokio::spawn(async move {
        if let Err(e) = http::run(http_file_map).await {
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

