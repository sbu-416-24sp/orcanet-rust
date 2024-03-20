use anyhow::Result;
use clap::Parser;
use tonic::transport::Uri;

use market_proto::market_proto_rpc::market_client::MarketClient;

use crate::cli::{Cli, LOOPBACK_ADDR};

#[tokio::main]
async fn main() -> Result<()> {
    // Market server is typically just a local server process that represents the DHT for the peer
    // node. Peer nodes then communicate through TCP sockets to the market server with the gRPC
    // method abstractions.
    let cli = Cli::parse();
    let port = cli.port;
    let mut client = MarketClient::connect(
        Uri::builder()
            .scheme("http")
            .authority(format!("{}:{}", LOOPBACK_ADDR, port).as_str())
            .path_and_query("/")
            .build()?,
    )
    .await?;
    todo!()
}

mod cli;
