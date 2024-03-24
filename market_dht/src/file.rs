use anyhow::Result;
use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub fn new_cidv0(data: &[u8]) -> Result<Cid> {
    let h = Code::Sha2_256.digest(data);
    let cid = Cid::new_v0(h);
    Ok(cid?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    ip: Ipv4Addr,
    port: u16,
    price_per_mb: u64,
}

impl FileMetadata {
    pub(crate) fn new(ip: Ipv4Addr, port: u16, price_per_mb: u64) -> Self {
        Self {
            ip,
            port,
            price_per_mb,
        }
    }

    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn price_per_mb(&self) -> u64 {
        self.price_per_mb
    }
}
