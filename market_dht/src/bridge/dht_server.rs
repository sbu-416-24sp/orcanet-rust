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
    core::{transport::ListenerId, ConnectedPoint},
    identify::{self, Info},
    kad::{
        self,
        store::{MemoryStore, RecordStore},
        Behaviour, BootstrapOk, GetClosestPeersError, GetClosestPeersOk, GetRecordOk, PutRecordOk,
        QueryId, QueryResult, Record, RecordKey,
    },
    swarm::{dial_opts::DialOpts, SwarmEvent},
    PeerId, Swarm,
};
use log::{error, info, warn};

use crate::{
    command::{Command, CommandCallback},
    file::FileMetadata,
    CommandOk, CommandResult,
};

type CommandResultSender = oneshot::Sender<CommandResult>;

use self::macros::{get_and_send, send_oneshot};
use super::{Behaviour as OrcaBehaviour, BehaviourEvent};

/// DHT server that listens for client commands and responds to them by communicating with the
/// libp2p DHT swarm
pub struct DhtServer {
    swarm: Swarm<OrcaBehaviour>,
    cmd_receiver: mpsc::Receiver<CommandCallback>,
    pending_new_listeners: HashMap<ListenerId, CommandResultSender>,
    pending_delete_listeners: HashMap<ListenerId, CommandResultSender>,
    pending_outgoing_dial: HashMap<PeerId, CommandResultSender>,
}

impl DhtServer {
    pub(crate) fn new(
        cmd_receiver: mpsc::Receiver<CommandCallback>,
        swarm: Swarm<OrcaBehaviour>,
    ) -> Self {
        Self {
            cmd_receiver,
            swarm,
            pending_new_listeners: Default::default(),
            pending_delete_listeners: Default::default(),
            pending_outgoing_dial: Default::default(),
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
                        info!("Command channel closed");
                        break Ok(());
                    }
                }
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await?;
                }
            }
        }
    }
    async fn handle_cmd(&mut self, cmd: CommandCallback) -> Result<()> {
        let (cmd, sender) = cmd;
        match cmd {
            Command::AddListener { addr } => match self.swarm.listen_on(addr) {
                Ok(id) => {
                    self.pending_new_listeners.insert(id, sender);
                }
                Err(err) => {
                    send_oneshot!(sender, Err(err.into()));
                }
            },
            Command::RemoveListener { listener_id } => {
                if self.swarm.remove_listener(listener_id) {
                    self.pending_delete_listeners.insert(listener_id, sender);
                } else {
                    send_oneshot!(sender, Err(anyhow!("Listener ID not found")));
                }
            }
            Command::GetAllListeners => {
                send_oneshot!(
                    sender,
                    Ok(CommandOk::GetAllListeners {
                        listeners: self.swarm.listeners().cloned().collect()
                    })
                );
            }
            Command::DialWithIdAddr { peer_id, addr } => {
                let opts = DialOpts::peer_id(peer_id).addresses(vec![addr]).build();
                match self.pending_outgoing_dial.entry(peer_id) {
                    Entry::Occupied(_) => {
                        send_oneshot!(
                            sender,
                            Err(anyhow!("Dial for {peer_id} already in progress"))
                        );
                    }
                    // TODO: add kademlia here?
                    Entry::Vacant(entry) => match self.swarm.dial(opts) {
                        Ok(()) => {
                            entry.insert(sender);
                        }
                        Err(err) => {
                            send_oneshot!(sender, Err(err.into()));
                        }
                    },
                }
            }
            ev => {
                todo!()
            }
        };
        Ok(())
    }

    async fn handle_kademlia_event(&mut self, event: kad::Event) -> Result<()> {
        match event {
            kad::Event::InboundRequest { request } => match request {
                kad::InboundRequest::FindNode { num_closer_peers } => todo!(),
                kad::InboundRequest::GetProvider {
                    num_closer_peers,
                    num_provider_peers,
                } => todo!(),
                kad::InboundRequest::AddProvider { record } => todo!(),
                kad::InboundRequest::GetRecord {
                    num_closer_peers,
                    present_locally,
                } => todo!(),
                kad::InboundRequest::PutRecord {
                    source,
                    connection,
                    record,
                } => todo!(),
            },
            kad::Event::OutboundQueryProgressed {
                id,
                result,
                stats,
                step,
            } => {
                todo!()
            }
            kad::Event::RoutingUpdated {
                peer,
                is_new_peer,
                addresses,
                old_peer,
                ..
            } => {
                if let Some(old_peer) = old_peer {
                    warn!("{old_peer} was evicted from the routing table");
                }
                if is_new_peer {
                    info!("{peer} was added to the routing table");
                } else {
                    info!("{peer} was updated in the routing table");
                }
                info!("{peer} has the following addresses in its DHT: {addresses:?}");
            }
            kad::Event::RoutablePeer { peer, address }
            | kad::Event::PendingRoutablePeer { peer, address } => {
                info!("{peer} is routable at {address}");
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer, address);
            }
            kad::Event::ModeChanged { new_mode } => {
                warn!("Kademlia mode changed to {new_mode}");
            }
            ev => {
                warn!("Unsupported Kademlia event: {ev:?}");
            }
        };
        Ok(())
    }

    async fn handle_identify_event(&mut self, event: identify::Event) -> Result<()> {
        match event {
            identify::Event::Received { peer_id, info } => {
                let Info {
                    protocol_version,
                    listen_addrs,
                    protocols,
                    ..
                } = info;
                info!("Received identify message from {peer_id}");
                info!("Protocol Version: {protocol_version}");
                info!("Listen Addresses: {listen_addrs:?}");
                info!("Supported Protocols: {protocols:?}");

                for addr in listen_addrs {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr);
                }
            }
            identify::Event::Sent { peer_id } => {
                warn!("Sent an identify message to {peer_id}");
            }
            identify::Event::Pushed { peer_id, .. } => {
                warn!("Pushed identify info to {peer_id}");
            }
            identify::Event::Error { peer_id, error } => {
                error!("Error in identifying {peer_id}: {error}");
            }
        };
        Ok(())
    }
    async fn handle_behaviour_event(&mut self, event: BehaviourEvent) -> Result<()> {
        match event {
            BehaviourEvent::Kademlia(event) => self.handle_kademlia_event(event).await,
            BehaviourEvent::Identify(event) => self.handle_identify_event(event).await,
        }
    }
    async fn handle_swarm_event(&mut self, event: SwarmEvent<BehaviourEvent>) -> Result<()> {
        match event {
            SwarmEvent::Behaviour(event) => {
                self.handle_behaviour_event(event).await;
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                established_in,
                ..
            } => {
                warn!("[{connection_id}] Connection established with {peer_id} established in {established_in:?}. Peer connections: {num_established}");
                match endpoint {
                    ConnectedPoint::Dialer { address, .. } => {
                        info!("[{connection_id}] Successfully dialed {address}!");
                        get_and_send!(
                            self.pending_outgoing_dial,
                            peer_id,
                            Ok(CommandOk::DialWithIdAddr {
                                peer_id,
                                addr: address
                            }),
                            "outgoing_dial"
                        );
                    }
                    ConnectedPoint::Listener {
                        local_addr,
                        send_back_addr,
                    } => {
                        info!("[{connection_id}] Successfully connected to {send_back_addr} from {local_addr} as a listener!");
                    }
                };
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => {
                if let Some(err) = cause {
                    warn!("[{connection_id}] Connection closed with {peer_id}. Peer connections: {num_established}. Reason: {err}");
                } else {
                    warn!("[{connection_id}] Connection closed with {peer_id}. Peer connections: {num_established}");
                }
                match endpoint {
                    ConnectedPoint::Dialer { address, .. } => {
                        self.swarm
                            .behaviour_mut()
                            .kademlia
                            .remove_address(&peer_id, &address);
                    }
                    ConnectedPoint::Listener {
                        local_addr,
                        send_back_addr,
                    } => {
                        // TODO: find a better way to do this in the case a user might want
                        // to use multiple listen addresses and not disconnect from all of em?
                        // One approach might be using a custom struct for storing the
                        // connections here?
                        if let Some(_kbucket) =
                            self.swarm.behaviour_mut().kademlia.remove_peer(&peer_id)
                        {
                            let _ = self.swarm.disconnect_peer_id(peer_id);
                        }
                    }
                };
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                warn!(
                    "[ConnId {connection_id}] Incoming connection from {} to {}",
                    send_back_addr, local_addr
                );
            }
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => {
                error!(
                    "[ConnId {connection_id}] Incoming connection failed from {} to {}. Reason: {error}",
                    send_back_addr, local_addr
                );
            }
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => {
                error!("[ConnId {connection_id}] Failed to connect to peer: {error}");
                if let Some(peer_id) = peer_id {
                    get_and_send!(
                        self.pending_outgoing_dial,
                        peer_id,
                        Err(error.into()),
                        "outgoing_dial"
                    );
                }
            }
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                get_and_send!(
                    self.pending_new_listeners,
                    listener_id,
                    Ok(CommandOk::AddListener {
                        addr: address,
                        listener_id
                    }),
                    "new_listeners"
                );
            }
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => {
                warn!("Listener {listener_id} expired: {address}");
            }
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                ..
            } => {
                get_and_send!(
                    self.pending_delete_listeners,
                    listener_id,
                    Ok(CommandOk::RemoveListener {
                        listener_id,
                        addresses
                    }),
                    "delete_listeners"
                );
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                warn!("Listener {listener_id} error: {:?}", error);
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => {
                if let Some(peer_id) = peer_id {
                    warn!("[ConnId {connection_id}] Dialing {peer_id}");
                } else {
                    warn!("[ConnId {connection_id}] Dialing peer");
                }
            }
            SwarmEvent::NewExternalAddrCandidate { address } => {
                warn!("New external address candidate: {}", address);
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                self.swarm.add_external_address(address);
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                self.swarm.remove_external_address(&address);
            }
            _ => todo!(),
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
    macro_rules! get_and_send {
        ($map:expr, $key:expr, $msg:expr, $map_type:expr) => {
            if let Some(sender) = $map.remove(&$key) {
                send_oneshot!(sender, $msg);
            } else {
                error!("Key {} not found in the pending {} map", $key, $map_type);
            }
        };
    }
    pub(crate) use get_and_send;
    pub(crate) use send_oneshot;
}
