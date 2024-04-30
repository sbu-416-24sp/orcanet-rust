use libp2p::{
    autonat::Behaviour as AutoNatBehaviour,
    dcutr::Behaviour as DcutrBehaviour,
    identify::Behaviour as IdentifyBehaviour,
    kad::{store::MemoryStore, Behaviour as KadBehaviour},
    ping::Behaviour as PingBehaviour,
    relay::{client::Behaviour as RelayClientBehaviour, Behaviour as RelayServerBehaviour},
    request_response::cbor::Behaviour as CborReqResBehaviour,
    swarm::{behaviour::toggle::Toggle, NetworkBehaviour},
};

use crate::lmm::{FileInfoHash, SupplierInfo};

// TODO: maybe do somethign with toggle in future?

#[derive(NetworkBehaviour)]
pub(crate) struct Behaviour {
    pub(crate) kad: KadBehaviour<MemoryStore>,
    pub(crate) identify: IdentifyBehaviour,
    pub(crate) ping: PingBehaviour,
    pub(crate) autonat: AutoNatBehaviour,
    pub(crate) relay_server: Toggle<RelayServerBehaviour>,
    pub(crate) dcutr: Toggle<DcutrBehaviour>,
    pub(crate) relay_client: Toggle<RelayClientBehaviour>,
    pub(crate) req_res: CborReqResBehaviour<FileInfoHash, SupplierInfo>,
}
