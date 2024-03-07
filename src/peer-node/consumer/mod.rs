pub mod http;

use crate::grpc::MarketClient;

use anyhow::Result;

pub async fn run(market: String) -> Result<()> {
    let mut client = MarketClient::new(market).await?;

    let file_hash = "file_hash".to_string();
    let producers = client.get_producers(file_hash).await?;

    // For now, use the first producer
    let producer = producers.producers.get(0).ok_or(anyhow::anyhow!("No producers found"))?;

    // Fetch the file from the producer
    match http::get_file(producer.clone()).await {
        Ok(_) => println!("File downloaded successfully"),
        Err(e) => eprintln!("Error downloading file: {}", e),
    }

    Ok(())
}