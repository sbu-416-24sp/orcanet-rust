//! File related utilities
use anyhow::Result;
use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

/// Creates a new CIDv0 from the given data that is to be inserted into the DHT as the key
///
/// # Errors
/// Fails based on the [Cid::new_v0] function result provided by the [Cid] crate
pub fn new_cidv0(data: &[u8]) -> Result<Cid> {
    let h = Code::Sha2_256.digest(data);
    let cid = Cid::new_v0(h);
    Ok(cid?)
}

/// Metadata for the file that is stored in the DHT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    ip: Ipv4Addr,
    port: u16,
    price_per_mb: u64,
}

impl FileMetadata {
    pub(crate) const fn new(ip: Ipv4Addr, port: u16, price_per_mb: u64) -> Self {
        Self {
            ip,
            port,
            price_per_mb,
        }
    }

    /// Returns the IP address of the peer that is storing the file
    pub const fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    /// Returns the port of the peer that is storing the file
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns the price per MB of the file
    pub const fn price_per_mb(&self) -> u64 {
        self.price_per_mb
    }
}
