use libp2p::{
    autonat::Behaviour as AutoNatBehaviour, identify::Behaviour as IdentifyBehaviour,
    kad::Behaviour as KadBehaviour, ping::Behaviour as PingBehaviour, swarm::NetworkBehaviour,
};

use self::{
    file_req_res::FileReqResBehaviour,
    ident::Identify,
    kademlia::{BootstrapMode, Kad, KadError, KadStore},
};

#[derive(NetworkBehaviour)]
#[allow(missing_debug_implementations)]
#[non_exhaustive] // NOTE: maybe more protocols?
pub(crate) struct MarketBehaviour<TKadStore: KadStore> {
    kademlia: Kad<TKadStore>,
    identify: Identify,
    file_req_res: FileReqResBehaviour,
    ping: PingBehaviour,
    autonat: AutoNatBehaviour,
}

impl<TKadStore: KadStore> MarketBehaviour<TKadStore> {
    #[inline(always)]
    pub(crate) const fn new(
        kademlia: KadBehaviour<TKadStore>,
        identify: IdentifyBehaviour,
        file_req_res: FileReqResBehaviour,
        ping: PingBehaviour,
        autonat: AutoNatBehaviour,
    ) -> Self {
        Self {
            kademlia: Kad::new(kademlia),
            identify: Identify::new(identify),
            file_req_res,
            ping,
            autonat,
        }
    }

    pub(crate) fn bootstrap(&mut self, mode: BootstrapMode) -> Result<(), KadError> {
        if let BootstrapMode::WithNodes(boot_nodes) = mode {
            for node in boot_nodes.iter() {
                self.kademlia
                    .kad_mut()
                    .add_address(&node.peer_id, node.addr.clone());
            }
            self.kademlia_mut()
                .kad_mut()
                .bootstrap()
                .map_err(|err| KadError::Bootstrap(err.to_string()))?;
            for node in boot_nodes {
                self.autonat.add_server(node.peer_id, Some(node.addr));
            }
        } else {
            self.kademlia_mut()
                .kad_mut()
                .bootstrap()
                .map_err(|err| KadError::Bootstrap(err.to_string()))?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) const fn kademlia(&self) -> &Kad<TKadStore> {
        &self.kademlia
    }

    pub(crate) fn kademlia_mut(&mut self) -> &mut Kad<TKadStore> {
        &mut self.kademlia
    }

    #[allow(dead_code)]
    pub(crate) const fn identify(&self) -> &Identify {
        &self.identify
    }

    #[allow(dead_code)]
    pub(crate) fn identify_mut(&mut self) -> &mut Identify {
        &mut self.identify
    }

    #[allow(dead_code)]
    pub(crate) const fn file_req_res(&self) -> &FileReqResBehaviour {
        &self.file_req_res
    }

    pub(crate) fn file_req_res_mut(&mut self) -> &mut FileReqResBehaviour {
        &mut self.file_req_res
    }

    pub(crate) fn autonat_mut(&mut self) -> &mut AutoNatBehaviour {
        &mut self.autonat
    }
}

mod macros {
    macro_rules! send_response {
        ($map: expr, $qid: expr, $msg: expr) => {
            if let Some(handler) = $map.remove(&$qid) {
                handler.respond($msg);
            }
        };
        ($request_handler: expr, $err: expr) => {
            $request_handler.respond(Err($err));
        };
    }
    pub(super) use send_response;
}
use macros::send_response;

pub(crate) mod autonat;
pub(crate) mod file_req_res;
pub(crate) mod ident;
pub(crate) mod kademlia;
