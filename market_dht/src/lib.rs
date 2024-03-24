//! A library for providing an interface to the Orcanet Kademlia DHT
#![warn(missing_debug_implementations)]
#![deny(unsafe_code, unreachable_pub)]
// NOTE: possibly an extension to more protocols later on in libp2p? so may have to also refactor
// the name.

mod bridge;
pub(crate) mod command;
use anyhow::Result;
pub use bridge::*;
use cid::Cid;
pub use command::{CommandOk, CommandResult};
use multihash_codetable::{Code, MultihashDigest};

pub fn new_cidv0(data: &[u8]) -> Result<Cid> {
    let h = Code::Sha2_256.digest(data);
    let cid = Cid::new_v0(h);
    Ok(cid?)
}
