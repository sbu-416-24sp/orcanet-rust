use anyhow::Result;
use orcanet_market_rust::{bridge::spawn, Config, SuccessfulResponse};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::default();
    let peer = spawn(config).unwrap();
    println!("{}", peer.peer_id());
    let listeners = peer.listeners().await?;
    if let SuccessfulResponse::Listeners { listeners } = listeners {
        for listener in listeners {
            println!("Listener: {}", listener);
        }
    }
    if let SuccessfulResponse::ConnectedPeers { peers } = peer.connected_peers().await? {
        for peer in peers {
            println!("Peer: {}", peer);
        }
    }
    Ok(())
}
