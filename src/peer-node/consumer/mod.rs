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

  let mut chunk = 0;
  let mut token = String::from("token");
  loop {
    match http::get_file(producer.clone(), file_hash.clone(),token, chunk).await {
      Ok(auth_token) => {
        token = auth_token;
        println!("Chunk {} downloaded successfully", chunk);
        chunk += 1;
      }
      Err(e) => {
        eprintln!("Error downloading chunk {}: {}", chunk, e);
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
