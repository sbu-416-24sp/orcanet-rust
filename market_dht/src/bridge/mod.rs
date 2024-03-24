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
            let peer_id = key.public().to_peer_id();
            kad::Behaviour::new(peer_id, MemoryStore::new(peer_id))
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(8)))
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
        let mut client = setup();
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
        let mut client = setup();
        let file_cid = new_cidv0(b"this is some content!").unwrap();
        client.get_file(&file_cid.to_bytes()).await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "the quorum failed")]
    async fn test_register_file_should_fail() {
        let mut client = setup();
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
        let mut client = setup();
        let peer_id = PeerId::random();
        let addr = "/ip4/127.0.0.1/tcp/6969".parse().unwrap();
        let msg = client.dial(peer_id, addr).await.unwrap();
        assert!(matches!(msg, CommandOk::Dial { .. }))
    }

    #[tokio::test]
    async fn test_listen_on() {
        let mut client = setup();
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
        let mut client = setup();
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
        let mut client = setup();
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

// use std::{fmt::Debug, marker::PhantomData};
//
// use anyhow::Result;
// use libp2p::{
//     futures::StreamExt,
//     kad::{self, store::MemoryStore, Behaviour},
//     noise, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
// };
// use tokio::sync::mpsc::{channel, Receiver, Sender};

// pub type MarketTokioSender = Sender<Command>;
// pub type MarketTokioReceiver = Receiver<Command>;
//
// pub struct Peer {
//     receiver: MarketTokioReceiver,
//     swarm: Swarm<Behaviour<MemoryStore>>,
// }
//
// impl Peer {
//     pub const CHANNEL_BUF_SIZE: usize = 64;
//     pub fn new(
//         boot_nodes: impl IntoIterator<Item = (PeerId, Multiaddr)>,
//         listen_addr: Multiaddr,
//     ) -> Result<(Self, MarketTokioSender)> {
//         // NOTE: libp2p-kad default uses /ipfs/kad/1.0.0 protocol
//         // maybe use a connection_idle_timeout?
//         let mut swarm = SwarmBuilder::with_new_identity()
//             .with_tokio()
//             .with_tcp(
//                 Default::default(),
//                 noise::Config::new,
//                 yamux::Config::default,
//             )?
//             .with_dns()?
//             .with_behaviour(|key| {
//                 let peer_id = key.public().to_peer_id();
//                 kad::Behaviour::new(peer_id, MemoryStore::new(peer_id))
//             })?
//             .build();
//         // NOTE: there's the case of where there's no bootnodes which means that it can be a
//         // genesis/boot node itself
//         for node in boot_nodes {
//             swarm.behaviour_mut().add_address(&node.0, node.1);
//         }
//         // NOTE: we'll just use tokio channels for now and thus, require that tokio is used
//         let (tx, rx) = channel(Self::CHANNEL_BUF_SIZE);
//         Ok((
//             Peer {
//                 receiver: rx,
//                 swarm,
//             },
//             tx,
//         ))
//     }
//
//     pub async fn run(&mut self) -> Result<()> {
//         tokio::select! {
//             command = self.receiver.recv() => {
//
//             }
//             kad_event = self.swarm.select_next_some() => {
//
//             }
//         }
//         Ok(())
//     }
// }

// impl Debug for Peer {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Peer")
//             .field("receiver", &self.receiver)
//             .field("swarm peer id", &self.swarm.local_peer_id())
//             .finish()
//     }
// }

// #[derive(Debug, Clone)]
// pub struct MarketSender<'a, T> {
//     sender: MarketTokioSender,
//     _pd: PhantomData<&'a T>,
// }
//
// impl<'a, T> MarketSender<'a, T> {
//     pub fn new(sender: MarketTokioSender) -> Self {
//         Self {
//             sender,
//             _pd: PhantomData,
//         }
//     }
// }

// impl<TChannel> Peer<TChannel> {
//     fn new(channel: TChannel, boot_nodes: Vec<PeerId>) -> Self {
//         Self {
//             channel,
//             boot_nodes,
//         }
//     }
// }
//
// impl Peer<MarketTokioChannel> {
//     fn init(&self) {}
// }
//
// #[derive(Debug, Clone, Default)]
// pub struct PeerBuilder<TChannel> {
//     boot_nodes: Vec<PeerId>,
//     _pd: PhantomData<TChannel>,
// }
//
// impl<TChannel> PeerBuilder<TChannel> {
//     pub fn new() -> Self {
//         Self {
//             boot_nodes: Vec::default(),
//             _pd: PhantomData,
//         }
//     }
//
//     pub fn with_boot_nodes<T: Into<PeerId>>(
//         mut self,
//         boot_nodes: impl IntoIterator<Item = T>,
//     ) -> Self {
//         self.boot_nodes = boot_nodes.into_iter().map(Into::into).collect();
//         self
//     }
//
//     // TODO:: only allowing tokio channels for now, but will generalize furrther later
//     // and that means you can only build the struct with tokio channels
//     pub fn with_tokio_channel(self) -> PeerBuilder<MarketTokioChannel> {
//         PeerBuilder {
//             boot_nodes: self.boot_nodes,
//             _pd: PhantomData,
//         }
//     }
// }
//
// impl PeerBuilder<MarketTokioChannel> {
//     pub fn build(self, buffer: usize) -> Peer<MarketTokioChannel> {
//         let (sender, receiver) = channel(buffer);
//         Peer::new((sender, receiver), self.boot_nodes)
//     }
// }

// pub trait SenderChannel {
//     fn send(&self, message: impl Into<MarketMessage>);
// }
// pub trait ReceiverChannel {}
// /// NOTE: if ever useful for future purposes (perhaps channels other than tokio ones)
// impl SenderChannel for Sender<MarketMessage> {
//     fn send(&self, message: impl Into<MarketMessage>) {
//     }
// }
// impl ReceiverChannel for Receiver<MarketMessage> {}
