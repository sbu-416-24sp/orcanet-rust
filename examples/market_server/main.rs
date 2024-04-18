use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub type Port = u16;

use crate::{cli::Cli, market_service::MarketService};
use anyhow::Result;
use clap::Parser;
use market_p2p::{bridge::spawn, config::BootNodes, config::Config};
use market_proto::market_proto_rpc::market_server::MarketServer;
use tokio::runtime::Runtime;
use tonic::transport::Server;
use tracing::info;

fn main() -> Result<()> {
    tracing_log::LogTracer::init()?;
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_line_number(true)
        .with_file(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();
    let market_port = cli.market_port;
    let peer_port = cli.peer_port;
    let boot_nodes = cli.boot_nodes;
    let public_address = cli.public_address;

    let mut config = Config::builder();
    if let Some(boot_nodes) = boot_nodes {
        config = config.set_boot_nodes(BootNodes::with_nodes(boot_nodes));
    }
    if let Some(public_address) = public_address {
        config = config.set_public_address(public_address);
    }
    let config = config.build();

    let peer = spawn(config)?;
    let market_listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), market_port);
    let market_service = MarketService::new(peer);

    info!("Market is listening on {}", market_listen_addr);
    Runtime::new()
        .unwrap()
        .block_on(
            Server::builder()
                .add_service(MarketServer::new(market_service))
                .serve(market_listen_addr),
        )
        .unwrap();

    Ok(())
}

mod cli;
mod market_service;
