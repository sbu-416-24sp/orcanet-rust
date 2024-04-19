# Orcanet Market Rust

A library that provides the implementation of the Orcanet Market using [libp2p](https://github.com/libp2p/rust-libp2p) as the framework for building the network.

The library uses [tokio](https://tokio.rs/) as the asynchronous executor for handling the requests sent by the peer and within the libp2p swarm.

## Quickstart
Creating a peer and using the provided methods within it.

```Rust
use anyhow::Result;
use orcanet_market::{bridge::spawn, Config, SuccessfulResponse};

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
```

## MSRV
The minimum supported Rust version (MSRV) is 1.73.0.

## License
This is under the MIT License.

