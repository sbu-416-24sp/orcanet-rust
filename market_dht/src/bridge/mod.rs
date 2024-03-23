//! A bridge between a dht_client API with the actual dht_server

pub mod dht_client;
pub mod dht_server;

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
