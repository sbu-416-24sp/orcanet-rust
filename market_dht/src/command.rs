use std::{collections::HashSet, net::Ipv4Addr};

use anyhow::Result;
use futures::channel::oneshot;
use libp2p::{core::transport::ListenerId, swarm::dial_opts::DialOpts, Multiaddr, PeerId};

pub(crate) type CommandCallback = (Command, oneshot::Sender<CommandResult>);
pub type CommandResult = Result<CommandOk>;

#[derive(Debug)]
pub enum CommandOk {
    Listen {
        listener_id: ListenerId,
    },
    Bootstrap {
        peer: PeerId,
        num_remaining: u32,
    },
    Dial {
        opts: DialOpts,
    },
    Register {
        file_cid: String,
    },
    FindHolders {
        file_cid: String,
        peers: HashSet<PeerId>,
    },
    GetClosestPeers {
        file_cid: String,
        peers: Vec<PeerId>,
    },
}

#[derive(Debug)]
pub(crate) enum Command {
    Listen {
        addr: Multiaddr,
    },
    Bootstrap {
        boot_nodes: Vec<(PeerId, Multiaddr)>,
    },
    Dial {
        opts: DialOpts,
    },
    // NOTE: Register should probably just be start_providing; the reason being is that this should
    // be the only file owner until the file is actually purchased. Rather in the future, it
    // shouldn't be file but it should be a chunk. We'll allow peer nodes to register chunks here,
    // but we'll call it file for now.
    //
    // TODO: I think put_record does make sense too since for users that actually want the content the
    // data stored here is still a reference for those peers who want to get that content. Plus,
    // the peer node team is working on having peers that want the data through http.
    // So for those peers using get_record, they may not exactly get the peer node that is
    // providing the content, but they still get the reference of where to get the content actually
    // is. But that quite literally just means that we only need put_record and get_record
    Register {
        file_cid: String,
        ip: Ipv4Addr,
        port: u16,
        // TODO: maybe f64 instead?
        price_per_mb: u64,
    },
    // NOTE: this checks for who is willing to provide the file?
    FindHolders {
        file_cid: String,
    },
    GetClosestPeers {
        file_cid: String,
    },
}

// #[derive(Debug, Error)]
// pub enum CommandError {
//     #[error("failed to listen on {addr}")]
//     Listen { addr: Multiaddr },
//     #[error("failed to bootstrap {peer}, {num_remaining} remaining")]
//     Bootstrap { peer: PeerId, num_remaining: u32 },
//     #[error("failed to dial {err}")]
//     Dial { err: DialError },
//     #[error("failed to register the file with cid {cid}")]
//     Register { cid: String },
//     #[error("failed to find providers for file with cid {cid}")]
//     FindHolders { cid: String },
//     #[error("failed to find closest peers for file with cid {cid}")]
//     GetClosestPeers { cid: String },
// }
