use anyhow::Result;
use clap::Parser;
use tonic::transport::Uri;

use market_proto::market_proto_rpc::{market_client::MarketClient, RegisterFileRequest, User};

use crate::cli::{Cli, LOOPBACK_ADDR};

#[tokio::main]
async fn main() -> Result<()> {
    // Market server is typically just a local server process that represents the DHT for the peer
    // node. Peer nodes then communicate through TCP sockets to the market server with the gRPC
    // method abstractions.
    let cli = Cli::parse();
    let user = Box::leak(Box::new(User::new(
        cli.id,
        cli.username,
        cli.client_ip.to_string(),
        cli.client_port as i32,
        i64::try_from(cli.price)?,
    )));
    let mut client = MarketClient::connect(
        Uri::builder()
            .scheme("http")
            .authority(format!("{}:{}", LOOPBACK_ADDR, cli.market_port).as_str())
            .path_and_query("/")
            .build()?,
    )
    .await?;
    RegisterFileRequest {
        user: Some(user.to_owned()),
        file_hash: "asd".to_owned(),
    };
    todo!()
}

mod cli;
