use std::net::Ipv4Addr;

use anyhow::Result;
use cid::Cid;
use futures::channel::oneshot;
use libp2p::{Multiaddr, PeerId};

use crate::file::FileMetadata;

pub(crate) type CommandCallback = (Command, oneshot::Sender<CommandResult>);
/// Result for the client's [Command] request
pub type CommandResult = Result<CommandOk>;

#[derive(Debug)]
/// The successful result of the [Command] that was sent
pub enum CommandOk {
    /// Successful result for the [Command::Listen] request sent
    Listen {
        /// Returns the address of where the peer is now listening
        addr: Multiaddr,
    },
    /// Successful result for the [Command::Bootstrap] request sent
    Bootstrap {
        /// The peer sent from the [libp2p::Swarm] bootstrap request
        peer: PeerId,
        /// The num_remaining sent from the [libp2p::Swarm] bootstrap request
        num_remaining: u32,
    },
    /// Successful result for the [Command::Dial] request sent
    Dial {
        /// The peer that we have successfully dialed
        peer: PeerId,
    },
    /// Successful result for the [Command::Register] request sent
    Register {
        /// The CID of the file that the user requested to register to the DHT
        // TODO: maybe change to CID type instead
        file_cid: Vec<u8>,
    },
    /// Successful result for the [Command::GetFile] request sent
    GetFile {
        /// The CID of the file that the user requested
        file_cid: Vec<u8>,
        /// The value/metadata of the file that the user requested
        metadata: FileMetadata,
        /// The peer that we got the record from
        owner_peer: PeerId,
    },
    /// Successful result for the [Command::GetClosestPeers] request sent
    GetClosestPeers {
        /// The CID of the file that the user wants to find the closest peers to
        file_cid: Vec<u8>,
        /// The list of closest peers
        peers: Vec<PeerId>,
    },
    /// Successful result for the [Command::GetLocalPeerId] request sent
    GetLocalPeerId {
        /// The peer id of the local node
        peer_id: PeerId,
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
        peer_id: PeerId,
        addr: Multiaddr,
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
        file_cid: Cid,
        ip: Ipv4Addr,
        port: u16,
        // TODO: maybe f64 instead?
        price_per_mb: u64,
    },
    // NOTE: this checks for who is willing to provide the file?
    GetFile {
        file_cid: Cid,
    },
    GetClosestPeers {
        file_cid: Cid,
    },
    GetLocalPeerId,
}

// TODO: use actual error types, but im lazy atm so l8r
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
