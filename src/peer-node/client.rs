mod consumer;
mod producer;
mod grpc;

use clap::Parser;
use anyhow::Result;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();

    match args.producer {
        true => producer::run(args.market).await?,
        false => consumer::run(args.market).await?,
    }

    Ok(())
}