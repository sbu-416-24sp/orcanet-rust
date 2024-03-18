pub mod http;

use crate::grpc::MarketClient;

use anyhow::Result;

pub async fn run(market: String, file_hash: String) -> Result<()> {
    let mut client = MarketClient::new(market).await?;

    // Check the producers for the file
    println!("Consumer: Checking producers for file hash {}", file_hash);
    let producers = client.check_holders(file_hash.clone()).await?;

    // For now, use the first producer
    // TODO: Pick the least expensive producer
    let producer = producers
        .holders
        .get(0)
        .ok_or(anyhow::anyhow!("No producers found"))?;
    println!(
        "Consumer: Found producer at {}:{}",
        producer.ip, producer.port
    );

    // Fetch the file from the producer
    match http::get_file(producer.clone(), file_hash).await {
        Ok(_) => println!("File downloaded successfully"),
        Err(e) => eprintln!("Error downloading file: {}", e),
    }

    Ok(())
}
