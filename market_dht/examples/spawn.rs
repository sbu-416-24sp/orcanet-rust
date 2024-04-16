use std::env::args;

use log::Level;
use market_dht::{bridge::spawn, BootNodes, Config};
use tracing_log::LogTracer;

fn main() {
    let sub = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(sub).unwrap();
    LogTracer::init().unwrap();
    let args = args().collect::<Vec<_>>();
    let config;
    if let Some(boot_node) = args.get(1) {
        config = Config::builder()
            .set_boot_nodes(BootNodes::try_with_nodes(vec![boot_node.as_str()]).unwrap())
            .set_peer_tcp_port(19000)
            .build();
    } else {
        config = Config::builder().set_peer_tcp_port(16899).build();
    }
    println!("{:?}", config);
    spawn(config).unwrap();
}
