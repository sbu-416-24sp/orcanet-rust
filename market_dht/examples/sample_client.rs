use std::{
    env,
    thread::{self},
    time::Duration,
};

use anyhow::Result;

use libp2p::{Multiaddr, PeerId};
use log::info;
use market_dht::{bridge, CommandOk};
use tokio::{
    runtime::Runtime,
    sync::{
        mpsc,
        oneshot::{self, Receiver, Sender},
    },
    time::sleep,
};
use tracing_log::LogTracer;

fn spawner(
    sender: Sender<(PeerId, Multiaddr)>,
    receiver: Receiver<(PeerId, Multiaddr)>,
    block: bool,
    listen_addr: Option<&str>,
    thread_name: &str,
) -> Result<()> {
    let (client, mut server) = bridge(16)?;
    let args: Vec<String> = env::args().collect();
    thread::scope(|s| {
        let _ = thread::Builder::new()
            .name(thread_name.to_owned() + "server_runner")
            .spawn_scoped(s, move || {
                Runtime::new()
                    .unwrap()
                    .block_on(async move { server.run().await })
                    .unwrap();
            });
        let _ = thread::Builder::new()
            .name(thread_name.to_owned() + "client_runner")
            .spawn_scoped(s, move || {
                Runtime::new().unwrap().block_on(async move {
                    let res = client
                        .listen_on(
                            listen_addr
                                .unwrap_or("/ip4/0.0.0.0/tcp/0")
                                .parse::<Multiaddr>()
                                .unwrap(),
                        )
                        .await
                        .unwrap();
                    if let CommandOk::AddListener { addr, listener_id } = res {
                        let peer_id = client.get_peer_id();
                        sender.send((peer_id, addr.clone())).unwrap();
                        let (other_peer_id, other_addr) = receiver.await.unwrap();
                        if block {
                            sleep(Duration::from_secs(999999)).await;
                        } else {
                            info!("Peer ID: {:?}", peer_id);
                            info!("Addr: {:?}", addr);
                            info!("Other Peer ID: {:?}", other_peer_id);
                            info!("Other Peer Addr: {:?}", other_addr);
                            client
                                .dial_peer_with_id_addr(other_peer_id, other_addr)
                                .await
                                .unwrap();
                            sleep(Duration::from_secs(9999999)).await;
                        }
                    }
                });
            });
    });
    Ok(())
}

fn main() -> Result<()> {
    LogTracer::init()?;
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_line_number(true)
        .with_thread_names(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let (sender_1, receiver_1) = oneshot::channel();
    let (sender_2, receiver_2) = oneshot::channel();
    thread::scope(|s| {
        let _ = thread::Builder::new()
            .name("thread_actual_caller".to_owned())
            .spawn_scoped(s, move || {
                spawner(
                    sender_1,
                    receiver_2,
                    false,
                    Some("/ip4/127.0.0.1/tcp/6669"),
                    "thread_actual_caller",
                )
                .unwrap();
            })
            .unwrap();
        let _ = thread::Builder::new()
            .name("thread_big_listener".to_owned())
            .spawn_scoped(s, move || {
                spawner(
                    sender_2,
                    receiver_1,
                    true,
                    Some("/ip4/127.0.0.1/tcp/1234"),
                    "thread_big_listener",
                )
                .unwrap();
            })
            .unwrap();
    });
    Ok(())
}
