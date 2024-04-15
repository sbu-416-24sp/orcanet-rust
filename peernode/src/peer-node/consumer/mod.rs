pub mod http;

use crate::grpc::MarketClient;

use anyhow::Result;
use sha2::Digest;
use sha2::Sha256;
use std::fs::File;
use std::io;

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
    let mut file_path = String::new();
    // TODO: allow looping through chunks, but client should be allowed to cancel at any time
    // when the client cancels, the chunk num they stopped at should be returned to them so they
    // can query another producer for the next chunk
    loop {
        match http::get_file_chunk(producer.clone(), file_hash.clone(), token, chunk).await {
            Ok(response) => {
                match response {
                    http::GetFileResponse::Token(new_token, file_name) => {
                        token = new_token;
                        if file_path.is_empty() {
                            file_path = format!("download/{}", file_name);
                        }
                    }
                    http::GetFileResponse::Done => {
                        println!("Consumer: File downloaded successfully");
                        break;
                    }
                }
                chunk += 1;
            }
            Err(e) => {
                eprintln!("Failed to download chunk {}: {}", chunk, e);
                break;
            }
        }
    }

    // Verify the file hash
    let mut file = File::open(&file_path)?;
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256)?;
    let hash = sha256.finalize();
    let hash = format!("{:x}", hash);
    
    if hash == file_hash {
        println!("Consumer: File hash verified");
    } else {
        eprintln!("Consumer: File hash does not match");
        println!("Expected: {}", file_hash);
        println!("Actual: {}", hash);
    }

    Ok(())
}
