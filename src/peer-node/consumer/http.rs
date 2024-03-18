use anyhow::{Result, anyhow};

use crate::grpc::orcanet::User;

pub async fn get_file(producer: User, file_hash: String) -> Result<()> {
    // Get the link to the file
    let link = format!("http://{}:{}/file/{}", producer.ip, producer.port, file_hash);
    println!("HTTP: Fetching file from {}", link);

    // Fetch the file from the producer
    let client = reqwest::Client::new();
    let res = client.get(format!("{}?chunk=0", link))
        .header("Authorization", format!("Bearer {}", "token"))
        .send()
        .await?;

    // Check if the request was successful
    if !res.status().is_success() {
        return Err(anyhow!("Request failed with status code: {}", res.status()));
    }

    // Get the file name from the Content-Disposition header
    let headers = res.headers().clone();
    let content_disposition = headers.get("Content-Disposition")
        .ok_or(anyhow!("No Content-Disposition header"))?
        .to_str()?;
    let file_name = match content_disposition.split("filename=").last() {
        Some(name) => name,
        None => return Err(anyhow!("No filename in Content-Disposition header")),
    };
    let file_name = file_name.trim_matches(|c| c == '"'); // Remove quotes
    
    // Save the file to disk
    let file = res.bytes().await?;
    tokio::fs::write(format!("download/{}", file_name), file).await?;
    println!("HTTP: File saved as {}", file_name);
    Ok(())
}