//! # DHT Server
//!
//! This is just the other end of the bridge where it handles the commands from the client by
//! directing it to the actual [libp2p] swarm.

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
};

use anyhow::{anyhow, Context, Result};
use futures::{
    channel::{mpsc, oneshot},
    select, StreamExt,
};
use libp2p::{
    core::transport::ListenerId,
    kad::{
        self,
        store::{MemoryStore, RecordStore},
        Behaviour, BootstrapOk, Event, GetClosestPeersError, GetClosestPeersOk, GetRecordOk,
        PutRecordOk, QueryId, QueryResult, Record, RecordKey,
    },
    swarm::SwarmEvent,
    PeerId, Swarm,
};
use log::{error, info, warn};

use crate::{
    command::{Command, CommandCallback},
    file::FileMetadata,
    CommandOk, CommandResult,
};

type CommandResultSender = oneshot::Sender<CommandResult>;

use self::macros::send_oneshot;

/// DHT server that listens for client commands and responds to them by communicating with the
/// libp2p DHT swarm
pub struct DhtServer {
    swarm: Swarm<Behaviour<MemoryStore>>,
    cmd_receiver: mpsc::Receiver<CommandCallback>,
    pending_queries: HashMap<QueryId, CommandResultSender>,
    pending_dials: HashMap<PeerId, CommandResultSender>,
    pending_listeners: HashMap<ListenerId, CommandResultSender>,
}

impl DhtServer {
    pub(crate) fn new(
        cmd_receiver: mpsc::Receiver<CommandCallback>,
        swarm: Swarm<Behaviour<MemoryStore>>,
    ) -> Self {
        Self {
            cmd_receiver,
            swarm,
            pending_queries: Default::default(),
            pending_dials: Default::default(),
            pending_listeners: Default::default(),
        }
    }
    /// Starts running the DHT server
    ///
    /// # Errors
    /// The error returned can depend on either if it's an error that comes from a command or an
    /// event from the swarm. Later on in the future, there will be something more concrete than
    /// the generic [anyhow] error type.
    pub async fn run(&mut self) -> Result<()> {
        loop {
            select! {
                command = self.cmd_receiver.next() => {
                    if let Some(command) = command {
                        self.handle_cmd(command).await?;
                    } else {
                        break Ok(());
                    }
                }
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await?;
                }
            }
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<Event>) -> Result<()> {
        match event {
            // FIXIT: bootstrap is extremely broken atm
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result:
                    QueryResult::Bootstrap(Ok(BootstrapOk {
                        peer,
                        num_remaining,
                    })),
                ..
            }) => {
                // NOTE: bootstrap still returns BootstrapOk even if dialing the bootnodes
                // fail...
                if let Some(sender) = self.pending_queries.remove(&id) {
                    info!("Bootstrapped peer {peer}");
                    send_oneshot!(
                        sender,
                        Ok(CommandOk::Bootstrap {
                            peer,
                            num_remaining,
                        })
                    );
                } else {
                    error!("Query ID not found");
                }
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::Bootstrap(Err(kad::BootstrapError::Timeout { .. })),
                ..
            }) => {
                error!("Bootstrap timed out!");
                if let Some(sender) = self.pending_queries.remove(&id) {
                    send_oneshot!(sender, Err(anyhow!("Bootstrap timeout")));
                } else {
                    error!("Query ID not found");
                }
            }
            SwarmEvent::OutgoingConnectionError {
                peer_id: Some(peer_id),
                error,
                connection_id,
            } => {
                error!("[{connection_id}]: Failed to connect to {peer_id}");
                if let Some(sender) = self.pending_dials.remove(&peer_id) {
                    send_oneshot!(sender, Err(error.into()));
                } else {
                    error!("Peer ID not found");
                }
            }
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                connection_id,
            } => {
                warn!("[{connection_id}]: Currently dialing {peer_id}");
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::PutRecord(Err(err)),
                ..
            }) => {
                error!("PutRecord failed: {err}");
                if let Some(sender) = self.pending_queries.remove(&id) {
                    send_oneshot!(sender, Err(err.into()));
                } else {
                    error!("Query ID not found")
                }
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::PutRecord(Ok(PutRecordOk { key })),
                ..
            }) => {
                info!("Record {key:?} was successfully placed into the DHT");
                if let Some(sender) = self.pending_queries.remove(&id) {
                    send_oneshot!(
                        sender,
                        Ok(CommandOk::Register {
                            file_cid: key.to_vec()
                        })
                    );
                } else {
                    error!("Query ID not found");
                }
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::GetRecord(Err(err)),
                ..
            }) => {
                error!("GetRecord failed: {err}");
                if let Some(sender) = self.pending_queries.remove(&id) {
                    send_oneshot!(sender, Err(err.into()));
                } else {
                    error!("Query ID not found");
                }
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::GetRecord(Ok(get_record_ok)),
                ..
            }) => {
                info!("GetRecord succeeded: {get_record_ok:?}");
                if let Some(sender) = self.pending_queries.remove(&id) {
                    if let GetRecordOk::FoundRecord(record_ok) = get_record_ok {
                        let record = record_ok.record;
                        match bincode::deserialize::<FileMetadata>(&record.value) {
                            Ok(metadata) => {
                                let peer = record_ok.peer.unwrap_or(*self.swarm.local_peer_id());
                                send_oneshot!(
                                    sender,
                                    Ok(CommandOk::GetFile {
                                        file_cid: record.key.to_vec(),
                                        metadata,
                                        owner_peer: peer
                                    })
                                );
                            }
                            Err(err) => {
                                send_oneshot!(sender, Err(err.into()))
                            }
                        };
                    } else {
                        // NOTE: Finished with no additional record case but they return no record so yeah.
                        send_oneshot!(sender, Err(anyhow!("Record not found")));
                    }
                } else {
                    error!("Query ID not found");
                }
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                error!("[{listener_id}] - Listener error: {error}");
                if let Some(sender) = self.pending_listeners.remove(&listener_id) {
                    send_oneshot!(sender, Err(error.into()));
                } else {
                    error!("Listener ID not found");
                }
            }
            SwarmEvent::NewListenAddr {
                address,
                listener_id,
            } => {
                info!("[{listener_id}] - Listening on {address}");
                if let Some(sender) = self.pending_listeners.remove(&listener_id) {
                    send_oneshot!(sender, Ok(CommandOk::Listen { addr: address }));
                } else {
                    error!("Listener ID not found");
                }
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::GetClosestPeers(Ok(GetClosestPeersOk { key, peers })),
                ..
            }) => {
                info!("Got closest peers");
                if let Some(sender) = self.pending_queries.remove(&id) {
                    send_oneshot!(
                        sender,
                        Ok(CommandOk::GetClosestPeers {
                            file_cid: key,
                            peers
                        })
                    )
                } else {
                    error!("Query ID not found");
                }
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::GetClosestPeers(Err(GetClosestPeersError::Timeout { key, .. })),
                ..
            }) => {
                if let Some(sender) = self.pending_queries.remove(&id) {
                    error!("Timed out on getting closest peers");
                    send_oneshot!(
                        sender,
                        Err(anyhow!("Timed out on getting closest peers with {key:?}"))
                    );
                } else {
                    error!("Query ID not found");
                }
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                info!(
                    "[{connection_id}] - Incoming connection from {send_back_addr} to {local_addr}"
                );
            }
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => {
                error!(
                    "[{connection_id}] - Incoming connection error from {send_back_addr} to {local_addr}: {error}"
                );
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                established_in,
                ..
            } => {
                info!("[{connection_id}] - Connection established with {peer_id} established in {established_in:?}");
                if endpoint.is_dialer() {
                    // NOTE: dialer already adds to address table
                    // taking here from libp2p example code
                    if let Some(sender) = self.pending_dials.remove(&peer_id) {
                        send_oneshot!(sender, Ok(CommandOk::Dial { peer: peer_id }));
                    } else {
                        error!("Peer ID not found");
                    }
                } else {
                    self.swarm
                        .behaviour_mut()
                        .add_address(&peer_id, endpoint.get_remote_address().clone());
                }
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                ..
            } => {
                warn!("[{connection_id}] - Connection closed with {peer_id}. There are {num_established} connections left with this peer.");
                self.swarm
                    .behaviour_mut()
                    .remove_address(&peer_id, endpoint.get_remote_address());
            }
            SwarmEvent::Behaviour(kad::Event::RoutingUpdated {
                peer, addresses, ..
            }) => {
                warn!("Routing table updated with {peer} {addresses:?}");
            }
            // TODO: support provider records?
            ev => {
                error!("Unsupported event handler for {ev:?}");
            }
        };
        Ok(())
    }

    async fn handle_cmd(&mut self, cmd: CommandCallback) -> Result<()> {
        let (cmd, sender) = cmd;
        info!("Received command: {cmd:?}");
        match cmd {
            Command::Listen { addr } => {
                match self.swarm.listen_on(addr) {
                    Ok(listener_id) => {
                        self.pending_listeners.insert(listener_id, sender);
                    }
                    Err(err) => {
                        send_oneshot!(sender, Err(err.into()));
                    }
                };
            }
            Command::Bootstrap { boot_nodes } => {
                // TODO: bootstrap can fail if boot_nodes is empty
                for node in boot_nodes {
                    self.swarm.behaviour_mut().add_address(&node.0, node.1);
                }
                // TODO: note that addresses we add here can be pending so the bootstrap command we
                // run after shouldn't actually fail. We can probably do something with blocking
                // channels here
                match self.swarm.behaviour_mut().bootstrap() {
                    Ok(qid) => {
                        self.pending_queries.insert(qid, sender);
                    }
                    Err(err) => {
                        send_oneshot!(sender, Err(err.into()));
                    }
                }
            }
            Command::Dial { peer_id, addr } => {
                // TODO: maybe use connectionIds in the future?
                // NOTE: taking this from the libp2p example
                if let Entry::Vacant(entry) = self.pending_dials.entry(peer_id) {
                    self.swarm
                        .behaviour_mut()
                        .add_address(&peer_id, addr.clone());
                    match self.swarm.dial(addr.with_p2p(peer_id).unwrap()) {
                        Ok(()) => {
                            entry.insert(sender);
                        }
                        Err(err) => {
                            send_oneshot!(sender, Err(err.into()));
                        }
                    }
                } else {
                    send_oneshot!(sender, Err(anyhow!("Dial already in progress")));
                }
            }
            Command::Register {
                file_cid,
                ip,
                port,
                price_per_mb,
            } => {
                // NOTE: read message in command.rs
                let mut record = Record::new(
                    file_cid.to_bytes(),
                    bincode::serialize(&FileMetadata::new(ip, port, price_per_mb))?,
                );
                // TODO: set expiration
                record.publisher = Some(*self.swarm.local_peer_id());
                match self
                    .swarm
                    .behaviour_mut()
                    .put_record(record, kad::Quorum::One)
                {
                    Ok(qid) => {
                        info!("[{qid}] - PutRecord was stored locally successfully");
                        self.pending_queries.insert(qid, sender);
                    }
                    Err(err) => {
                        error!("PutRecord failed: failed to store key locally");
                        send_oneshot!(sender, Err(err.into()));
                    }
                }
            }
            Command::GetFile { file_cid } => {
                let key = RecordKey::new(&file_cid.to_bytes());
                let qid = self.swarm.behaviour_mut().get_record(key);
                self.pending_queries.insert(qid, sender);
            }
            Command::GetClosestPeers { file_cid } => {
                let key = RecordKey::new(&file_cid.to_bytes());
                let qid = self.swarm.behaviour_mut().get_closest_peers(key.to_vec());
                self.pending_queries.insert(qid, sender);
            }
            Command::GetLocalPeerId => {
                let peer_id = *self.swarm.local_peer_id();
                send_oneshot!(sender, Ok(CommandOk::GetLocalPeerId { peer_id }));
            }
        };
        Ok(())
    }
}

impl Debug for DhtServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DhtServer")
            .field("cmd_receiver", &self.cmd_receiver)
            .field("swarm_local_peer_id", &self.swarm.local_peer_id())
            .finish()
    }
}

mod macros {
    macro_rules! send_oneshot {
        ($sender:expr, $msg:expr) => {
            if let Err(err) = $sender.send($msg) {
                return Err(anyhow!("Failed to send oneshot response: {:?}", err));
            }
        };
    }
    pub(crate) use send_oneshot;
}
