mod db;
mod files;
mod http;

use std::sync::Arc;

use crate::grpc::MarketClient;

use anyhow::{anyhow, Result};

pub async fn run(market: String, ip: Option<String>) -> Result<()> {
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

    // Get the public IP address
    let ip = match ip {
        Some(ip) => ip,
        // Use the AWS checkip service to get the public IP address
        None => match reqwest::get("http://checkip.amazonaws.com").await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text.trim().to_string(),
                Err(e) => {
                    return Err(anyhow!("Failed to get public IP: {}", e));
                }
            },
            Err(e) => {
                return Err(anyhow!("Failed to get public IP: {}", e));
            }
        },
    };
    println!("Producer: IP address is {}", ip);

    // Register the files with the market service
    let hash = file_map.get_hashes().await;
    for hash in hash {
        // TODO: Find a way to get the public IP address
        println!("Producer: Registering file with hash {}", hash);
        client
            .register_file(
                "id".to_string(),
                "name".to_string(),
                ip.clone(),
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
