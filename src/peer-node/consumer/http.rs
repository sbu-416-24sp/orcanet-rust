use anyhow::{anyhow, Result};

use crate::grpc::orcanet::User;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub async fn get_file(
    producer: User,
    file_hash: String,
    token: String,
    chunk: u64,
) -> Result<String> {
    // Get the link to the file
    let link = format!(
        "http://{}:{}/file/{}",
        producer.ip, producer.port, file_hash
    );
    println!("HTTP: Fetching file from {}", link);

    // Fetch the file from the producer
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}?chunk={}", link, chunk))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    // Check if the request was successful
    if !res.status().is_success() {
        return Err(anyhow!("Request failed with status code: {}", res.status()));
    }

    // get auth token header from response
    let auth_token = res
        .headers()
        .get("X-Access-Token")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    // Get the file name from the Content-Disposition header
    let headers = res.headers().clone();
    let content_disposition = headers
        .get("Content-Disposition")
        .ok_or(anyhow!("No Content-Disposition header"))?
        .to_str()?;
    let file_name = match content_disposition.split("filename=").last() {
        Some(name) => name,
        None => return Err(anyhow!("No filename in Content-Disposition header")),
    };
    let file_name = file_name.trim_matches(|c| c == '"'); // Remove quotes

    // Save the file to disk
    let file = res.bytes().await?;
    let file_path = format!("download/{}", file_name);
    let mut download = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .await?;

    download.write_all(&file).await?;
    println!("HTTP: File saved as {}", file_name);
    Ok(auth_token)
}
