mod consumer;
mod grpc;
mod producer;

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Peer node client
struct Args {
    /// Market service address
    #[arg(short, long, default_value = "localhost:50051")]
    market: String,

    /// Whether to run as a producer
    #[arg(short, long, default_value = "false")]
    producer: bool,

    /// File hash
    /// Only used when running as a consumer
    #[arg(short, long)]
    file_hash: Option<String>,

    /// IP address which should be provided to the market service
    /// If not provided, the producer will find its own public IP address
    #[arg(long, requires("producer"))]
    ip: Option<String>,

    /// Port the producer should listen on
    /// If not provided, the producer will listen on 8080
    #[arg(long, requires("producer"))]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();

    match args.producer {
        true => producer::run(args.market, args.ip, args.port).await?,
        false => match args.file_hash {
            Some(file_hash) => consumer::run(args.market, file_hash).await?,
            None => return Err(anyhow!("No file hash provided")),
        },
    }

    Ok(())
}
