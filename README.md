# Orcanet Market Rust

A library that provides the implementation of the Orcanet Market using [libp2p](https://github.com/libp2p/rust-libp2p) as the framework for building the network.

The library uses [tokio](https://tokio.rs/) as the asynchronous executor for handling the requests sent by the peer and within the libp2p swarm.

## Quickstart
Creating a peer and using the provided methods within it.

```Rust
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
```

## Project Structure
Here is an explanation of the project's directory structure
- `examples/` - this is where you can find the `market_server` example that uses this library as its backend to perform the p2p communications and provides a gRPC communication with the `market_test_client`. You may find the other examples useful as well.
- `src/` - this is the source code for the implementation of the Orcanet Market
- `tests/` - this is where you can find the tests for the Orcanet Market in the library
- `CHANGELOG.md` - provides a changelog for the project

## License
This is under the MIT License.

