use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use market_proto::market_proto_rpc::market_server::MarketServer;
use market_service::MarketService;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let service = MarketService::default();
    println!("Starting market server on port {}", cli.port);
    Server::builder()
        .add_service(MarketServer::new(service))
        .serve(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            cli.port,
        ))
        .await?;
    Ok(())
}

mod cli;
mod market_service;
