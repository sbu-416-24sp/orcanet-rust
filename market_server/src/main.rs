use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
};

use anyhow::Result;
use clap::Parser;
use libp2p::{Multiaddr, PeerId};
use market_proto::market_proto_rpc::market_server::MarketServer;
use market_server::{cli::Cli, market_service::MarketService};
use tokio::{runtime::Runtime, sync::oneshot};
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
    let boot_nodes = cli
        .boot_nodes
        .into_iter()
        .map(|node| {
            let node = node.split(',').collect::<Vec<_>>();
            if node.len() != 2 || node[0].is_empty() || node[1].is_empty() {
                panic!("Invalid boot node address");
            } else {
                (
                    node[0].parse::<PeerId>().unwrap(),
                    node[1].parse::<Multiaddr>().unwrap(),
                )
            }
        })
        .collect::<Vec<(PeerId, Multiaddr)>>();
    let (dht_client, mut dht_server) = market_dht::bridge(256)?;

    thread::scope(|s| {
        let (server_started_tx, server_started_rx) = oneshot::channel();
        thread::Builder::new()
            .name("dht_server_bridge".to_owned())
            .spawn_scoped(s, move || {
                Runtime::new().unwrap().block_on(async move {
                    server_started_tx.send(true).unwrap();
                    info!("Starting DHT market server on port {}", peer_port);
                    dht_server.run().await.unwrap();
                });
            })
            .unwrap();
        thread::Builder::new()
            .name("market_server_dht_client".to_owned())
            .spawn_scoped(s, move || {
                server_started_rx
                    .blocking_recv()
                    .expect("It should not have dropped the oneshot");
                Runtime::new().unwrap().block_on(async move {
                    let listen = dht_client
                        .listen_on(
                            format!("/ip4/127.0.0.1/tcp/{peer_port}")
                                .parse::<Multiaddr>()
                                .unwrap(),
                        )
                        .await
                        .expect("Failed to listen on address");
                    info!("{listen:?}");
                    let local_peer_node_id = dht_client.get_local_peer_id().await.unwrap();
                    info!("Local peer node ID: {:?}", local_peer_node_id);
                    if !boot_nodes.is_empty() {
                        info!("Bootstrapping with the provided bootnodes: {boot_nodes:?}");
                        // TODO: lazy here I need to fix bootstrapping
                        dht_client
                            .dial(boot_nodes[0].0, boot_nodes[0].1.clone())
                            .await
                            .expect("Failed to bootstrap with the provided bootnodes");
                    }

                    {
                        let service = MarketService::new(dht_client);
                        info!("Starting gRPC market server on port {}", market_port);
                        Server::builder()
                            .add_service(MarketServer::new(service))
                            .serve(SocketAddr::new(
                                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                                market_port,
                            ))
                            .await
                            .unwrap();
                    }
                });
            })
            .unwrap();
    });

    // let service = MarketService::default();
    // info!("Starting market server on port {}", market_port);
    // Server::builder()
    //     .add_service(MarketServer::new(service))
    //     .serve(SocketAddr::new(
    //         IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    //         market_port,
    //     ))
    //     .await?;
    Ok(())
}
