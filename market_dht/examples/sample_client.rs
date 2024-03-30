use std::{borrow::Cow, thread, time::Duration};

use market_dht::{config::Config, multiaddr, net::spawn_bridge};
use tokio::runtime::Runtime;
use tracing_log::LogTracer;

fn main() {
    LogTracer::init().unwrap();
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_thread_names(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let peer1 = spawn_bridge(
        Config::builder()
            .with_thread_name("peer1".to_owned())
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(5555u16)))
            .build(),
    )
    .unwrap();
    let peer1_id = peer1.id();
    let peer2 = spawn_bridge(
        Config::builder()
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(1234u16)))
            .with_thread_name("peer2".to_owned())
            .with_boot_nodes(
                vec![("/ip4/127.0.0.1/tcp/5555".to_owned(), peer1_id.to_string())]
                    .try_into()
                    .unwrap(),
            )
            .build(),
    )
    .unwrap();

    let peer3 = spawn_bridge(
        Config::builder()
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(3333u16)))
            .with_thread_name("peer3".to_owned())
            .with_boot_nodes(
                vec![("/ip4/127.0.0.1/tcp/5555".to_owned(), peer1_id.to_string())]
                    .try_into()
                    .unwrap(),
            )
            .build(),
    )
    .unwrap();

    thread::sleep(Duration::from_secs(2));
    let peer3_id = peer3.id();
    let peer4 = spawn_bridge(
        Config::builder()
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(22222u16)))
            .with_thread_name("peer4".to_owned())
            .with_boot_nodes(
                vec![("/ip4/127.0.0.1/tcp/3333".to_owned(), peer3_id.to_string())]
                    .try_into()
                    .unwrap(),
            )
            .build(),
    )
    .unwrap();
    thread::sleep(Duration::from_secs(2));
    Runtime::new().unwrap().block_on(async {
        let peers = peer4
            .get_closest_local_peers(Cow::Owned(peer3_id.to_bytes()))
            .await
            .unwrap();
        println!("{:?}", peers);
        let peers = peer4
            .get_closest_peers(Cow::Owned(peer3_id.to_bytes()))
            .await
            .unwrap();
        println!("{:?}", peers);
    });
    thread::sleep(Duration::from_secs(7777777));
}
