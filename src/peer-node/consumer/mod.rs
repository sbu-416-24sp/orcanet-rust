pub mod http;
pub mod encode;

use std::io;

use crate::grpc::{MarketClient};
use anyhow::Result;


pub async fn list_producers(file_hash: String, market: String) -> Result<()> {
    let mut client = MarketClient::new(market).await?;
    let producers = client.check_holders(file_hash).await?;
    for producer in producers.holders {
        // serialize the producer struct to a string
        // let producer_str = serde_json::to_string(&producer)?;
        // let encoded_producer = general_purpose::STANDARD.encode(producer_str.as_bytes());
        let encoded_producer = encode::encode_user(&producer);

        // let mut encoder = EncoderWriter::new(&mut encoded_producer, &STANDARD);
        // io::copy(&mut producer_str, &mut encoder)?;
        // let encoded_producer = match encoding::all::ISO_8859_1.encode(producer_str.as_str(), encoding::EncoderTrap::Strict){
        //     Ok(encoded_producer) => encoded_producer,
        //     Err(_) => {
        //         eprintln!("Failed to encode producer");
        //         continue;
        //     }
        // };
        // for every field in the producer struct, convert it to a string
        // and append it to the string
        println!("Producer:\n  id: {}\n  Price: {}", encoded_producer, producer.price);
    }
    Ok(())
}

// pub async fn get_file_chunk(producer: User, file_hash: String, token: String, chunk: u64) -> Result<String> {
//     let mut Token = token;

//     match http::get_file_chunk(producer.clone(), file_hash.clone(), token, chunk).await {
//         Ok(response) => {
//             match response {
//                 http::GetFileResponse::Token(new_token) => {
//                     Token = new_token;
//                 }
//                 http::GetFileResponse::Done => {
//                     println!("Consumer: File downloaded successfully");
//                 }
//             }
//         }
//         Err(e) => {
//             eprintln!("Failed to download chunk {}: {}", chunk, e);
//         }
//     }

//     Ok("urmom".to_string())
// }

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
