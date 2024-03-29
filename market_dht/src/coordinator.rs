use futures::StreamExt;
use libp2p::{kad::store::MemoryStore, Swarm};
use log::debug;
use tokio::sync::{mpsc, oneshot::Sender};

use crate::{behaviour::MarketBehaviour, config::Config, peer::Peer};

pub(crate) struct Coordinator {
    swarm: Swarm<MarketBehaviour<MemoryStore>>,
}

impl Coordinator {
    pub(crate) fn new(mut swarm: Swarm<MarketBehaviour<MemoryStore>>, config: Config) -> Self {
        swarm.listen_on(config.listener).expect("listen_on to work");
        if let Some(boot_nodes) = config.boot_nodes {
            swarm
                .behaviour_mut()
                .kademlia
                .bootstrap(boot_nodes)
                .expect("bootstrap to work");
        }
        Self { swarm }
    }

    pub(crate) async fn run(mut self, ready_tx: Sender<Peer>) {
        let (request_tx, mut request_rx) = mpsc::unbounded_channel();
        let peer = Peer::new(request_tx);
        ready_tx.send(peer).unwrap();
        loop {
            tokio::select! {
                request = request_rx.recv() => {
                    if let Some(request) = request {
                    } else {
                        debug!("request_rx closed, shutting down coordinator");
                        break;
                    }
                }
                swarm_event = self.swarm.select_next_some() => {}
            }
        }
    }
}
