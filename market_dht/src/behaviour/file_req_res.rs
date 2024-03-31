use std::net::Ipv4Addr;

use libp2p::{request_response::cbor, swarm::NetworkBehaviour};
use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
pub(crate) struct FileReqResBehaviour {
    req_res: cbor::Behaviour,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FileHash(pub(crate) Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FileMetadata {
    pub(crate) file_hash: FileHash,
    pub(crate) supplier_info: SupplierInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct SupplierInfo {
    pub(crate) ip: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) price: i32,
    pub(crate) username: String,
}
