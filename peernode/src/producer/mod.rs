mod db;
mod http;

use crate::peer::MarketClient;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use proto::market::{FileInfoHash, User};

use crate::store::files::{FileMap, LocalFileInfo};

pub async fn start_server(
    files: HashMap<FileInfoHash, LocalFileInfo>,
    port: String,
) -> tokio::task::JoinHandle<()> {
    // Launch the HTTP server in the background
    let http_file_map = Arc::new(FileMap::new(files));
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
    files: HashMap<FileInfoHash, LocalFileInfo>,
    client: &mut MarketClient,
    port: String,
    ip: Option<String>,
) -> Result<()> {
    // get port from string
    let port = match port.parse::<i32>() {
        Ok(port) => port,
        Err(_) => {
            eprintln!("Invalid port number");
            return Ok(());
        }
    };

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
    // for testing
    let ip = "0.0.0.0".to_string();
    println!("Producer: IP address is {ip}");

    // Generate a random Producer ID
    let producer_id = uuid::Uuid::new_v4().to_string();

    for (
        hash,
        LocalFileInfo {
            file_info, price, ..
        },
    ) in files
    {
        println!("Producer {producer_id}: Registering file with hash {hash} and price {price}",);
        client
            .register_file(
                User {
                    id: producer_id.clone(),
                    name: "producer".into(),
                    ip: ip.clone(),
                    port,
                    price,
                },
                hash,
                file_info,
            )
            .await?;
    }

    Ok(())
}
