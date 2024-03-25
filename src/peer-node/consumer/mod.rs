pub mod http;

use crate::grpc::MarketClient;
use std::time::{Duration, Instant};

use anyhow::Result;

pub async fn run(market: String, file_hash: String) -> Result<()> {
    let mut client = MarketClient::new(market).await?;

    // Check the producers for the file
    println!("Consumer: Checking producers for file hash {}", file_hash);
    let producers = client.check_holders(file_hash.clone()).await?;

    // For now, use the first producer
    // TODO: Allow user to choose a producer, give them a list of options with IP and port
    let producer = producers
        .holders
        .get(0)
        .ok_or(anyhow::anyhow!("No producers found"))?;
    println!(
        "Consumer: Found producer at {}:{}",
        producer.ip, producer.port
    );

    let mut chunk = 0;
    let mut token = String::from("token");
    // TODO: allow looping through chunks, but client should be allowed to cancel at any time
    // when the client cancels, the chunk num they stopped at should be returned to them so they
    // can query another producer for the next chunk
    loop {
        let start = Instant::now();
        match http::get_file(producer.clone(), file_hash.clone(), token, chunk).await {
            Ok(auth_token) => {
                token = auth_token;
                println!("HTTP: Chunk {} downloaded successfully", chunk);
                chunk += 1;
            }
            Err(e) => {
                if e.to_string() == "Request failed with status code: 404 Not Found" {
                    println!("HTTP: File downloaded successfully");
                    break;
                }
                eprintln!("Failed to download chunk {}: {}", chunk, e);
                break;
            }
        }
        let duration = start.elapsed();
        println!("HTTP: Chunk {} took {:?} s to download", chunk, duration.as_secs());
    }

    // // Fetch the file from the producer
    // match http::get_file(producer.clone(), file_hash, chunk).await {
    //     Ok(_) => println!("File downloaded successfully"),
    //     Err(e) => eprintln!("Error downloading file: {}", e),
    // }

    Ok(())
}
