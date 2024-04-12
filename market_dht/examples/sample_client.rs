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
    thread::sleep(Duration::from_secs(1));
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

    thread::sleep(Duration::from_secs(1));
    let peer3 = spawn_bridge(
        Config::builder()
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(1233u16)))
            .with_thread_name("peer3".to_owned())
            .with_boot_nodes(
                vec![("/ip4/127.0.0.1/tcp/5555".to_owned(), peer1_id.to_string())]
                    .try_into()
                    .unwrap(),
            )
            .build(),
    )
    .unwrap();

    thread::sleep(Duration::from_secs(1));
    let peer3_id = peer3.id();
    let peer4 = spawn_bridge(
        Config::builder()
            .with_listener(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(22222u16)))
            .with_thread_name("peer4".to_owned())
            .with_boot_nodes(
                vec![("/ip4/127.0.0.1/tcp/1233".to_owned(), peer3_id.to_string())]
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
        let peer4_id = peer4.id();
        let peers = peer4
            .get_closest_peers(Cow::Owned(peer4_id.to_bytes()))
            .await
            .unwrap();
        println!("{:?}", peers);
        println!("{peer4_id}");
        let sha_hash = [33u8; 32];
        println!(
            "{:?}",
            peer3
                .register_file(
                    Cow::Owned(sha_hash.to_vec()),
                    [190, 32, 11, 23],
                    9003,
                    300,
                    "obama".to_string()
                )
                .await
        );
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!(
            "{:?}",
            peer4
                .register_file(
                    Cow::Owned(sha_hash.to_vec()),
                    [190, 32, 11, 23],
                    9003,
                    300,
                    "obama".to_string()
                )
                .await
        );
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!(
            "{:?}",
            peer1.check_holders(Cow::Owned(sha_hash.to_vec())).await
        );
        tokio::time::sleep(Duration::from_secs(20)).await;
        println!(
            "{:?}",
            peer1.check_holders(Cow::Owned(sha_hash.to_vec())).await
        );
    });
    thread::sleep(Duration::from_secs(7777777));
}
