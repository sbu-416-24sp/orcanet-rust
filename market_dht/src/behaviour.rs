use libp2p::{
    identify::Behaviour as IdentifyBehaviour, kad::Behaviour as KadBehaviour,
    swarm::NetworkBehaviour,
};

use self::{
    file_req_res::FileReqResBehaviour,
    ident::Identify,
    kademlia::{Kad, KadStore},
};

#[derive(NetworkBehaviour)]
#[allow(missing_debug_implementations)]
#[non_exhaustive] // NOTE: maybe more protocols?
pub(crate) struct MarketBehaviour<TKadStore: KadStore> {
    kademlia: Kad<TKadStore>,
    identify: Identify,
    file_req_res: FileReqResBehaviour,
}

impl<TKadStore: KadStore> MarketBehaviour<TKadStore> {
    #[inline(always)]
    pub(crate) const fn new(
        kademlia: KadBehaviour<TKadStore>,
        identify: IdentifyBehaviour,
        file_req_res: FileReqResBehaviour,
    ) -> Self {
        Self {
            kademlia: Kad::new(kademlia),
            identify: Identify::new(identify),
            file_req_res,
        }
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

pub(crate) mod file_req_res;
pub(crate) mod ident;
pub(crate) mod kademlia;
