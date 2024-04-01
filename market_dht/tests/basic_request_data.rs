use std::{thread, time::Duration};

use market_dht::{config::Config, multiaddr, net::spawn_bridge, ResponseData};
use pretty_assertions::{self, assert_eq};
use tokio::runtime::Runtime;

#[tokio::test]
async fn test_should_not_panic_in_async_context() {
    let _ = spawn_bridge(
        Config::builder()
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(4444u16)))
            .with_thread_name("peer1".to_owned())
            .build(),
    )
    .unwrap();
}

#[test]
fn test_get_connected_peers() {
    let peer1 = spawn_bridge(
        Config::builder()
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(1233u16)))
            .with_thread_name("peer1".to_owned())
            .build(),
    )
    .unwrap();

    let _peer2 = spawn_bridge(
        Config::builder()
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(1234u16)))
            .with_thread_name("peer2".to_owned())
            .with_boot_nodes(
                vec![("/ip4/127.0.0.1/tcp/1233".to_owned(), peer1.id().to_string())]
                    .try_into()
                    .unwrap(),
            )
            .build(),
    )
    .unwrap();

    thread::sleep(Duration::from_secs(1));
    Runtime::new().unwrap().block_on(async move {
        let response = peer1.get_connected_peers().await.unwrap();
        if let ResponseData::ConnectedPeers { connected_peers } = response {
            assert_eq!(1, connected_peers.len());
        } else {
            panic!("Didn't get the correct response!")
        }
    });
}
