use crate::grpc::orcanet::FileProducer;

use anyhow::Result;

pub async fn get_file(producer: FileProducer) -> Result<()> {
    // Check that a valid link is given
    let link = producer.link;
    if link.is_empty() {
        return Err(anyhow::anyhow!("No link provided"));
    }

    // Check if the link contains a protocol, and if not, add one
    let link = if link.starts_with("http://") || link.starts_with("https://") {
        link
    } else {
        format!("http://{}", link)
    };

    // Fetch the file from the producer
    let client = reqwest::Client::new();
    let res = client.get(format!("{}?chunk=0", link))
        .header("Authorization", format!("Bearer {}", "token"))
        .send()
        .await?;

    // Check if the request was successful
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Request failed with status code: {}", res.status()));
    }
    
    // Save the file to disk
    let file = res.bytes().await?;
    tokio::fs::write("giraffe_download.jpg", file).await?;
    Ok(())
}