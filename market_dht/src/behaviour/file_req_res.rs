use std::net::Ipv4Addr;

use libp2p::{
    request_response::{cbor, Config, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use serde::{Deserialize, Serialize};

pub(crate) const FILE_REQ_RES_PROTOCOL: [(StreamProtocol, ProtocolSupport); 1] = [(
    StreamProtocol::new("/file_req_res/1.0.0"),
    ProtocolSupport::Full,
)];

#[derive(NetworkBehaviour)]
pub(crate) struct FileReqResBehaviour {
    req_res: cbor::Behaviour<FileHash, SupplierInfo>,
}

impl FileReqResBehaviour {
    pub(crate) fn new<I: IntoIterator<Item = (StreamProtocol, ProtocolSupport)>>(
        protocols: I,
        config: Config,
    ) -> Self {
        Self {
            req_res: cbor::Behaviour::new(protocols, config),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
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
