pub mod encode;
pub mod http;

use crate::grpc::{orcanet::User, MarketClient};
use anyhow::Result;

use self::http::GetFileResponse;

pub async fn list_producers(file_hash: String, market: String) -> Result<()> {
    let mut client = MarketClient::new(market).await?;
    let producers = client.check_holders(file_hash).await?;
    for producer in producers.holders {
        // serialize the producer struct to a string
        let encoded_producer = encode::encode_user(&producer);
        println!(
            "Producer:\n  id: {}\n  Price: {}",
            encoded_producer, producer.price
        );
    }
    Ok(())
}

pub async fn get_file(
    producer: String,
    file_hash: String,
    token: String,
    chunk: u64,
    continue_download: bool,
) -> Result<String> {
    let producer_user = match encode::decode_user(producer.clone()) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Failed to decode producer: {}", e);
            return Err(anyhow::anyhow!("Failed to decode producer"));
        }
    };
    let mut chunk_num = chunk;
    let mut return_token = String::from(token);
    loop {
        match get_file_chunk(
            producer_user.clone(),
            file_hash.clone(),
            return_token.clone(),
            chunk_num,
        )
        .await
        {
            Ok(response) => {
                match response {
                    GetFileResponse::Token(new_token) => {
                        return_token = new_token;
                    }
                    GetFileResponse::Done => {
                        println!("Consumer: File downloaded successfully");
                        return Ok(return_token);
                    }
                }
                chunk_num += 1;
            }
            Err(e) => {
                eprintln!("Failed to download chunk {}: {}", chunk_num, e);
                return Err(anyhow::anyhow!("Failed to download chunk"));
            }
        }
        if continue_download == false {
            return Ok(return_token);
        }
    }
}

pub async fn get_file_chunk(
    producer: User,
    file_hash: String,
    token: String,
    chunk: u64,
) -> Result<GetFileResponse> {
    return http::get_file_chunk(producer, file_hash.clone(), token, chunk).await;
}

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
        match http::get_file_chunk(producer.clone(), file_hash.clone(), token, chunk).await {
            Ok(response) => {
                match response {
                    http::GetFileResponse::Token(new_token) => {
                        token = new_token;
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

    // // Fetch the file from the producer
    // match http::get_file(producer.clone(), file_hash, chunk).await {
    //     Ok(_) => println!("File downloaded successfully"),
    //     Err(e) => eprintln!("Error downloading file: {}", e),
    // }

    Ok(())
}
