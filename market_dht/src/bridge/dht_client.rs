//! DHT client for interacting with the DHT server

use std::{net::Ipv4Addr, sync::Arc};

use anyhow::Result;
use cid::Cid;
use futures::{
    channel::{mpsc::Sender, oneshot},
    lock::Mutex,
    SinkExt,
};
use libp2p::{Multiaddr, PeerId};

use crate::{
    command::{Command, CommandCallback},
    CommandOk, CommandResult,
};

/// Client that is used to interact with the DHT server
#[derive(Debug)]
pub struct DhtClient {
    sender: Arc<Mutex<Sender<CommandCallback>>>,
}

impl DhtClient {
    pub(crate) fn new(sender: Sender<CommandCallback>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }

    /// Sends a listening request to listen on some [Multiaddr]
    ///
    /// # Errors
    /// Can fail if the [Multiaddr] provided by the user is already in use
    pub async fn listen_on(&self, addr: impl Into<Multiaddr>) -> CommandResult {
        let (callback_sender, receiver) = oneshot::channel();
        let addr = addr.into();
        let mut sender_lock = self.sender.lock().await;
        sender_lock
            .send((Command::Listen { addr }, callback_sender))
            .await?;
        drop(sender_lock);
        receiver.await?
    }

    /// Sends a bootstrap request to start connecting to the user provided bootnodes
    ///
    /// # Errors
    /// This can ultimately fail if you provided no bootnodes.
    pub async fn bootstrap(
        &self,
        boot_nodes: impl IntoIterator<Item = (PeerId, Multiaddr)>,
    ) -> Result<CommandOk> {
        let (callback_sender, receiver) = oneshot::channel();
        let mut sender_lock = self.sender.lock().await;
        sender_lock
            .send((
                Command::Bootstrap {
                    boot_nodes: boot_nodes.into_iter().collect(),
                },
                callback_sender,
            ))
            .await?;
        drop(sender_lock);
        receiver.await?
    }

    /// Dials the requested peer knowing the peer_id and the addr
    ///
    /// # Errors
    /// Can fail if the peer_id or addr is invalid, or if we just can't connect due to not sharing
    /// the same protocols
    pub async fn dial(&self, peer_id: PeerId, addr: Multiaddr) -> CommandResult {
        // TODO: maybe change in the future to support it more genericly with dialopts
        let (callback_sender, receiver) = oneshot::channel();
        let mut sender_lock = self.sender.lock().await;
        sender_lock
            .send((Command::Dial { peer_id, addr }, callback_sender))
            .await?;
        drop(sender_lock);
        receiver.await?
    }

    /// Registers the file based on the CID to the DHT server
    ///
    /// We use the ip, port, and price_per_mb to register the as reference metadata to the DHT
    /// server. That way, users that use the [libp2p::kad::Behaviour::get_record] method to yield this
    /// reference metadata so that they can yield this specific file or chunk.
    ///
    /// # Errors
    /// Can fail if the file_cid is invalid
    pub async fn register(
        &self,
        file_cid: &[u8],
        ip: impl Into<Ipv4Addr>,
        port: u16,
        price_per_mb: u64,
    ) -> CommandResult {
        let (callback_sender, receiver) = oneshot::channel();
        let file_cid = Cid::try_from(file_cid)?;
        let mut sender_lock = self.sender.lock().await;
        sender_lock
            .send((
                Command::Register {
                    file_cid,
                    ip: ip.into(),
                    port,
                    price_per_mb,
                },
                callback_sender,
            ))
            .await?;
        drop(sender_lock);
        receiver.await?
    }

    /// Yields the file metadata based on the CID
    ///
    /// # Errors
    /// Fails if the file_cid is invalid
    pub async fn get_file(&self, file_cid: &[u8]) -> CommandResult {
        let (callback_sender, receiver) = oneshot::channel();
        let file_cid = Cid::try_from(file_cid)?;
        let mut sender_lock = self.sender.lock().await;
        sender_lock
            .send((Command::GetFile { file_cid }, callback_sender))
            .await?;
        drop(sender_lock);
        receiver.await?
    }

    /// Yields the closest peers based on the file CID
    ///
    /// # Errors
    /// Fails if the file_cid is invalid
    pub async fn get_closest_peers(&self, file_cid: &[u8]) -> CommandResult {
        let file_cid = Cid::try_from(file_cid)?;
        let (callback_sender, receiver) = oneshot::channel();
        let mut sender_lock = self.sender.lock().await;
        sender_lock
            .send((Command::GetClosestPeers { file_cid }, callback_sender))
            .await?;
        drop(sender_lock);
        receiver.await?
    }

    /// Yields the local peer node ID
    ///
    /// # Errors
    /// Fails if the file_cid is invalid
    pub async fn get_local_peer_id(&self) -> CommandResult {
        let (callback_sender, receiver) = oneshot::channel();
        let mut sender_lock = self.sender.lock().await;
        sender_lock
            .send((Command::GetLocalPeerId, callback_sender))
            .await?;
        drop(sender_lock);
        receiver.await?
    }
}

#[cfg(test)]
mod tests {
    use futures::{channel::oneshot::Sender, StreamExt};
    use libp2p::{Multiaddr, PeerId};
    use pretty_assertions::assert_eq;

    use crate::{command::Command, CommandOk, CommandResult};

    use super::DhtClient;
    // TODO: write tests

    #[tokio::test]
    #[should_panic]
    async fn test_bootstrap_should_fail() {
        let (sender, mut mock_receiver) =
            futures::channel::mpsc::channel::<(Command, Sender<CommandResult>)>(16);
        let client = DhtClient::new(sender);
        let mock_id = libp2p::PeerId::random();
        tokio::spawn(async move {
            while let Some(command) = mock_receiver.next().await {
                if let Command::Bootstrap { boot_nodes } = command.0 {
                    if boot_nodes.is_empty() {
                        command
                            .1
                            .send(Err(anyhow::anyhow!("no boot nodes")))
                            .unwrap();
                    }
                } else {
                    panic!("unexpected command: {:?}", command.0);
                }
            }
        });
        if let CommandOk::Bootstrap {
            peer,
            num_remaining,
        } = client
            .bootstrap([(
                PeerId::random(),
                "/ip4/127.0.0.1".parse::<Multiaddr>().unwrap(),
            )])
            .await
            .unwrap()
        {
            assert_eq!(peer, mock_id);
            assert_eq!(num_remaining, 32);
        }
    }

    #[tokio::test]
    async fn test_bootstrap_basic() {
        let (sender, mut mock_receiver) =
            futures::channel::mpsc::channel::<(Command, Sender<CommandResult>)>(16);
        let client = DhtClient::new(sender);
        let mock_id = libp2p::PeerId::random();
        tokio::spawn(async move {
            while let Some(command) = mock_receiver.next().await {
                if let Command::Bootstrap {
                    boot_nodes: _boot_nodes,
                } = command.0
                {
                    command
                        .1
                        .send(Ok(CommandOk::Bootstrap {
                            peer: mock_id,
                            num_remaining: 32,
                        }))
                        .unwrap();
                } else {
                    panic!("unexpected command: {:?}", command.0);
                }
            }
        });
        if let CommandOk::Bootstrap {
            peer,
            num_remaining,
        } = client
            .bootstrap([(
                PeerId::random(),
                "/ip4/127.0.0.1".parse::<Multiaddr>().unwrap(),
            )])
            .await
            .unwrap()
        {
            assert_eq!(peer, mock_id);
            assert_eq!(num_remaining, 32);
        }
    }

    #[tokio::test]
    async fn test_listen_on_basic() {
        let (sender, mut mock_receiver) =
            futures::channel::mpsc::channel::<(Command, Sender<CommandResult>)>(16);
        let expected_addr = "/ip4/127.0.0.1".parse::<Multiaddr>().unwrap();
        tokio::spawn(async move {
            if let Some(command) = mock_receiver.next().await {
                if let Command::Listen { addr: _addr } = command.0 {
                    command
                        .1
                        .send(Ok(CommandOk::Listen {
                            addr: expected_addr,
                        }))
                        .unwrap();
                } else {
                    panic!("unexpected command: {:?}", command.0);
                }
            }
        });
        let client = DhtClient::new(sender);
        // NOTE: this thing blocks until the oneshot is received back
        if let CommandOk::Listen { addr } = client
            .listen_on("/ip4/127.0.0.1".parse::<Multiaddr>().unwrap())
            .await
            .unwrap()
        {
            assert_eq!(addr, "/ip4/127.0.0.1".parse::<Multiaddr>().unwrap());
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn test_listen_on_command_bad_multiaddr() {
        let (sender, mut mock_receiver) =
            futures::channel::mpsc::channel::<(Command, Sender<CommandResult>)>(16);
        tokio::spawn(async move {
            while let Some(command) = mock_receiver.next().await {
                if let Command::Listen { addr: _addr } = command.0 {
                    command
                        .1
                        .send(Ok(CommandOk::Listen {
                            addr: "/ip4/".parse::<Multiaddr>().unwrap(),
                        }))
                        .unwrap();
                } else {
                    panic!("unexpected command: {:?}", command.0);
                }
            }
        });
        let client = DhtClient::new(sender);
        client
            .listen_on("/ip4/1270.0.1".parse::<Multiaddr>().unwrap())
            .await
            .unwrap();
    }
}
