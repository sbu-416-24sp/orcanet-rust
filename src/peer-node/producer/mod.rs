mod db;
mod http;

use crate::grpc::MarketClient;

use anyhow::Result;

pub async fn run(market: String) -> Result<()>  {
    let mut client = MarketClient::new(market).await?;

    // Launch the HTTP server in the background
    tokio::spawn(async move {
        if let Err(e) = http::run().await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    let hash = "file_hash".to_string();
    client.register_producer(
        hash.clone(),
        format!("http://localhost:8080/file/{}", hash),
        0.0,
        "payment_address".to_string(),
    ).await?;

    // Never return
    tokio::signal::ctrl_c().await?;

    Ok(())
}