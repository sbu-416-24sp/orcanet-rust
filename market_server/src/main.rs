use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Result;
use clap::Parser;
use libp2p::{multiaddr::Protocol, Multiaddr};
use market_dht::{boot_nodes::BootNodes, config::Config, net::spawn_bridge};
use market_proto::market_proto_rpc::market_server::MarketServer;
use market_server::{cli::Cli, market_service::MarketService};
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
    let boot_nodes = {
        if let Some(boot_nodes) = cli.boot_nodes {
            let boot_nodes = boot_nodes
                .into_iter()
                .map(|node| {
                    let node = node.split(',').collect::<Vec<_>>();
                    if node.len() != 2 || node[0].is_empty() || node[1].is_empty() {
                        panic!("Invalid boot node address");
                    } else {
                        (node[0].to_owned(), node[1].to_owned())
                    }
                })
                .collect::<Vec<_>>();
            Some(BootNodes::try_from(boot_nodes)?)
        } else {
            None
        }
    };
    let mut listen_addr = Multiaddr::empty();
    listen_addr.push(Protocol::Ip4(Ipv4Addr::new(127, 0, 0, 1)));
    listen_addr.push(Protocol::Tcp(peer_port));
    let config = {
        if let Some(boot_nodes) = boot_nodes {
            Config::builder()
                .with_listener(listen_addr)
                .with_boot_nodes(boot_nodes)
        } else {
            Config::builder().with_listener(listen_addr)
        }
    }
    .build();
    let peer = spawn_bridge(config)?;
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
