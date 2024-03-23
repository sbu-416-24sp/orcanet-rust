//! DHT server that listens for client commands and responds to them by communicating with the
//! libp2p DHT swarm

use std::{collections::HashMap, fmt::Debug};

use anyhow::{anyhow, Context, Result};
use futures::{
    channel::{mpsc, oneshot},
    select, StreamExt,
};
use libp2p::{
    kad::{
        self, store::MemoryStore, Behaviour, BootstrapOk, Event, QueryId, QueryResult, QueryStats,
    },
    swarm::SwarmEvent,
    PeerId, Swarm,
};

use crate::{
    command::{Command, CommandCallback},
    CommandOk, CommandResult,
};

pub struct DhtServer {
    swarm: Swarm<Behaviour<MemoryStore>>,
    cmd_receiver: mpsc::Receiver<CommandCallback>,
    pending_queries: HashMap<QueryId, oneshot::Sender<CommandResult>>,
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
                if let Err(err) = sender.send(Ok(CommandOk::Bootstrap {
                    peer,
                    num_remaining,
                })) {
                    return Err(anyhow!("Failed to send oneshot response: {:?}", err));
                }
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                id,
                result: QueryResult::Bootstrap(Err(kad::BootstrapError::Timeout { .. })),
                ..
            }) => {
                let sender = self
                    .pending_queries
                    .remove(&id)
                    .with_context(|| anyhow!("Query ID not found"))?;
                if let Err(err) = sender.send(Err(anyhow!("Bootstrap timeout"))) {
                    return Err(anyhow!("Failed to send oneshot response: {:?}", err));
                }
            }
            SwarmEvent::OutgoingConnectionError {
                peer_id: Some(peer_id),
                ..
            } => {
                self.swarm.behaviour_mut().remove_peer(&peer_id);
            }
            ev => {}
        };
        Ok(())
    }

    async fn handle_cmd(&mut self, cmd: CommandCallback) -> Result<()> {
        let (cmd, sender) = cmd;
        match cmd {
            Command::Listen { addr } => {
                let oneshot_res = match self.swarm.listen_on(addr) {
                    Ok(listener_id) => Ok(CommandOk::Listen { listener_id }),
                    Err(err) => Err(err.into()),
                };
                if let Err(err) = sender.send(oneshot_res) {
                    return Err(anyhow!("Failed to send oneshot response: {:?}", err));
                }
            }
            Command::Bootstrap { boot_nodes } => {
                // TODO: bootstrap can fail if boot_nodes is empty
                for node in boot_nodes {
                    self.swarm.behaviour_mut().add_address(&node.0, node.1);
                }
                // TODO: note that addresses we add here can be pending so the bootstrap command we
                // run after shouldn't actually fail.
                match self.swarm.behaviour_mut().bootstrap() {
                    Ok(qid) => {
                        self.pending_queries.insert(qid, sender);
                    }
                    Err(err) => {
                        if let Err(err) = sender.send(Err(err.into())) {
                            return Err(anyhow!("Failed to send oneshot response: {:?}", err));
                        }
                    }
                }
            }
            Command::Dial { opts } => todo!(),
            Command::Register {
                file_cid,
                ip,
                port,
                price_per_mb,
            } => todo!(),
            Command::FindHolders { file_cid } => todo!(),
            Command::GetClosestPeers { file_cid } => todo!(),
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
