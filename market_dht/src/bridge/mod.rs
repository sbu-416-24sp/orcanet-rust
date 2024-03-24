//! A bridge between a dht_client API with the dht_server

use std::time::Duration;

use anyhow::Result;
use futures::channel::mpsc;
use libp2p::{
    kad::{self, store::MemoryStore},
    noise, yamux, SwarmBuilder,
};

use self::{dht_client::DhtClient, dht_server::DhtServer};

pub mod dht_client;
pub mod dht_server;

/// Creates a dht client and server bridge
///
/// # Warning
/// [DhtClient] and a [DhtServer] object are returned. Note that this does not start the command
/// server. You will need to start the command server with the [DhtServer::run] method. Only then
/// will the client be able to send commands to the server. If the client sends a command before
/// the server is started, the client will block until the server is started so that it can listen
/// to client commands.
///
/// # Errors
/// Returns a generic error based on [anyhow::Error] but we will probably introduce a more concrete
/// type for the [Result]
///
/// # Example
/// Here's an example with [tokio]
///
/// ```rust
/// use market_dht::bridge;
/// use tokio::spawn;
/// use tokio::sync::oneshot;
/// use libp2p::Multiaddr;
///
/// # tokio_test::block_on(async {
///     let (mut client, mut server) = bridge(16).unwrap();
///     spawn(async move {
///         server.run().await;
///     });
///     client.listen_on("/ip4/127.0.0.1/tcp/6669".parse::<Multiaddr>().unwrap()).await.unwrap();
/// # })
///
/// ```
pub fn bridge(cmd_buffer: usize) -> Result<(DhtClient, DhtServer)> {
    let swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            Default::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|key| {
            // TODO: maybe mdns for bootstrap nodes that are close?
            let peer_id = key.public().to_peer_id();
            let config = kad::Config::default();
            kad::Behaviour::new(peer_id, MemoryStore::new(peer_id))
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60 * 60)))
        .build();
    let (command_sender, command_receiver) = mpsc::channel(cmd_buffer);
    // TODO: maybe bridge should also start up the command receiver server?
    Ok((
        DhtClient::new(command_sender),
        DhtServer::new(command_receiver, swarm),
    ))
}

#[cfg(test)]
mod tests {
    use libp2p::{Multiaddr, PeerId};
    use pretty_assertions::assert_eq;

    use crate::{file::new_cidv0, CommandOk};

    #[tokio::test]
    async fn test_get_closest_peers() {
        let client = setup();
        let file_cid = new_cidv0(b"this is some content!").unwrap();
        if let CommandOk::GetClosestPeers {
            file_cid: fcid,
            peers,
        } = client
            .get_closest_peers(&file_cid.to_bytes())
            .await
            .unwrap()
        {
            assert!(peers.is_empty());
            assert_eq!(fcid, file_cid.to_bytes());
        } else {
            panic!("unexpected command")
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn test_find_holder_should_fail() {
        let client = setup();
        let file_cid = new_cidv0(b"this is some content!").unwrap();
        client.get_file(&file_cid.to_bytes()).await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "the quorum failed")]
    async fn test_register_file_should_fail() {
        let client = setup();
        let file_cid = new_cidv0(b"this is some content!").unwrap();
        let ip = [127, 0, 0, 1];
        let port = 6969;
        let price_per_mb = 2;
        client
            .register(&file_cid.to_bytes(), ip, port, price_per_mb)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn test_dial_should_fail() {
        // NOTE: we'll have an actual test that can succeed in the integration tests when we can
        // create two peers with this library and have them dial each other
        let client = setup();
        let peer_id = PeerId::random();
        let addr = "/ip4/127.0.0.1/tcp/6969".parse().unwrap();
        let msg = client.dial(peer_id, addr).await.unwrap();
        assert!(matches!(msg, CommandOk::Dial { .. }))
    }

    #[tokio::test]
    async fn test_listen_on() {
        let client = setup();
        // NOTE: this blocks until the server on the other end sends that oneshot back or if the
        // oneshot is dropped without sending (in which case it would crash)
        let msg = client
            .listen_on("/ip4/127.0.0.1/tcp/6969".parse::<Multiaddr>().unwrap())
            .await
            .unwrap();
        assert!(matches!(msg, CommandOk::Listen { .. }))
    }

    #[tokio::test]
    async fn test_bootstrap_basic_should_pass() {
        let client = setup();
        // NOTE: this blocks until the server on the other end sends that oneshot back or if the
        // oneshot is dropped without sending (in which case it would crash)
        let mock_peer_id = PeerId::random();
        let mock_addr = "/ip4/127.0.0.1/tcp/6696".parse::<Multiaddr>().unwrap();
        // NOTE: this still passes since bootstrap still passes even if the dialing fails...
        client
            .bootstrap(vec![(mock_peer_id, mock_addr)])
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "No known peers")]
    async fn test_bootstrap_basic_should_fail() {
        let client = setup();
        client.bootstrap(vec![]).await.unwrap();
    }

    fn setup() -> super::DhtClient {
        let (client, mut server) = super::bridge(16).unwrap();
        tokio::spawn(async move {
            let _ = server.run().await;
        });
        client
    }
}
