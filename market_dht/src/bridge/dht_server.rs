//! DHT server that listens for client commands and responds to them by communicating with the
//! libp2p DHT swarm

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
        self, store::MemoryStore, Behaviour, BootstrapOk, Event, GetClosestPeersError,
        GetClosestPeersOk, GetRecordOk, PutRecordOk, QueryId, QueryResult, Record, RecordKey,
    },
    swarm::SwarmEvent,
    PeerId, Swarm,
};
use log::{error, info, warn};

use crate::{
    command::{Command, CommandCallback},
    CommandOk, CommandResult, FileMetadata,
};

type OneshotCommandResultSender = oneshot::Sender<CommandResult>;

use self::macros::send_oneshot;

pub struct DhtServer {
    swarm: Swarm<Behaviour<MemoryStore>>,
    cmd_receiver: mpsc::Receiver<CommandCallback>,
    pending_queries: HashMap<QueryId, OneshotCommandResultSender>,
    pending_dials: HashMap<PeerId, OneshotCommandResultSender>,
    pending_listeners: HashMap<ListenerId, OneshotCommandResultSender>,
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
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
                info!("Bootstrapped peer {peer}");
                send_oneshot!(
                    sender,
                    Ok(CommandOk::Bootstrap {
                        peer,
                        num_remaining,
                    })
                );
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::Bootstrap(Err(kad::BootstrapError::Timeout { .. })),
                ..
            }) => {
                error!("Bootstrap timed out!");
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
                send_oneshot!(sender, Err(anyhow!("Bootstrap timeout")));
            }
            SwarmEvent::OutgoingConnectionError {
                peer_id: Some(peer_id),
                error,
                connection_id,
            } => {
                error!("[{connection_id}]: Failed to connect to {peer_id}");
                if let Some(sender) = self.pending_dials.remove(&peer_id) {
                    send_oneshot!(sender, Err(error.into()));
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
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
                send_oneshot!(sender, Err(err.into()));
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::PutRecord(Ok(PutRecordOk { key })),
                ..
            }) => {
                info!("Record {key:?} was successfully placed into the DHT");
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
                send_oneshot!(
                    sender,
                    Ok(CommandOk::Register {
                        file_cid: key.to_vec()
                    })
                );
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::GetRecord(Err(err)),
                ..
            }) => {
                error!("GetRecord failed: {err}");
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
                send_oneshot!(sender, Err(err.into()));
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::GetRecord(Ok(get_record_ok)),
                ..
            }) => {
                info!("GetRecord succeeded: {get_record_ok:?}");
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
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
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                error!("[{listener_id}] - Listener error: {error}");
                let sender = self
                    .pending_listeners
                    .remove(&listener_id)
                    .with_context(|| anyhow!("Listener ID not found"))?;
                send_oneshot!(sender, Err(error.into()));
            }
            SwarmEvent::NewListenAddr {
                address,
                listener_id,
            } => {
                info!("[{listener_id}] - Listening on {address}");
                let sender = self
                    .pending_listeners
                    .remove(&listener_id)
                    .with_context(|| anyhow!("Listener ID not found"))?;
                send_oneshot!(sender, Ok(CommandOk::Listen { addr: address }));
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::GetClosestPeers(Ok(GetClosestPeersOk { key, peers })),
                ..
            }) => {
                info!("Got closest peers");
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
                send_oneshot!(
                    sender,
                    Ok(CommandOk::GetClosestPeers {
                        file_cid: key,
                        peers
                    })
                )
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::GetClosestPeers(Err(GetClosestPeersError::Timeout { key, .. })),
                ..
            }) => {
                error!("Timed out on getting closest peers");
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
                send_oneshot!(
                    sender,
                    Err(anyhow!("Timed out on getting closest peers with {key:?}"))
                )
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
                        self.pending_queries.insert(qid, sender);
                    }
                    Err(err) => {
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
