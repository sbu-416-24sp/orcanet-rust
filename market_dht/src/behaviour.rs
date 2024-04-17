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

use crate::{lmm::FileHash, SupplierInfo};

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
    pub(crate) req_res: CborReqResBehaviour<FileHash, SupplierInfo>,
}

// impl Behaviour {
//     pub(crate) fn builder<T>(
//         kad: KadBehaviour<MemoryStore>,
//         identify: IdentifyBehaviour,
//         ping: PingBehaviour,
//         autonat: AutoNatBehaviour,
//     ) -> BehaviourBuilder<T> {
//         BehaviourBuilder::new(kad, identify, ping, autonat)
//     }
// }
//
// pub(crate) struct BehaviourBuilder<T> {
//     kad: KadBehaviour<MemoryStore>,
//     identify: IdentifyBehaviour,
//     ping: PingBehaviour,
//     autonat: AutoNatBehaviour,
//     relay_server: Toggle<RelayServerBehaviour>,
//     dcutr: Toggle<DcutrBehaviour>,
//     relay_client: Toggle<RelayClientBehaviour>,
//     _pd: std::marker::PhantomData<T>,
// }
//
// impl<T> BehaviourBuilder<T> {
//     pub(crate) fn new(
//         kad: KadBehaviour<MemoryStore>,
//         identify: IdentifyBehaviour,
//         ping: PingBehaviour,
//         autonat: AutoNatBehaviour,
//     ) -> Self {
//         BehaviourBuilder {
//             kad,
//             identify,
//             ping,
//             autonat,
//             relay_server: None.into(),
//             dcutr: None.into(),
//             relay_client: None.into(),
//             _pd: std::marker::PhantomData,
//         }
//     }
//
//     pub(crate) fn with_relay_server(
//         mut self,
//         relay_server: RelayServerBehaviour,
//     ) -> BehaviourBuilder<WithPublic> {
//         self.relay_server = Some(relay_server).into();
//         BehaviourBuilder {
//             kad: self.kad,
//             identify: self.identify,
//             ping: self.ping,
//             autonat: self.autonat,
//             relay_server: self.relay_server,
//             dcutr: self.dcutr,
//             relay_client: self.relay_client,
//             _pd: std::marker::PhantomData,
//         }
//     }
//
//     pub(crate) fn with_relay_client(
//         mut self,
//         relay_client: RelayClientBehaviour,
//     ) -> BehaviourBuilder<WithRelayClient> {
//         self.relay_client = Some(relay_client).into();
//         BehaviourBuilder {
//             kad: self.kad,
//             identify: self.identify,
//             ping: self.ping,
//             autonat: self.autonat,
//             relay_server: self.relay_server,
//             dcutr: self.dcutr,
//             relay_client: self.relay_client,
//             _pd: std::marker::PhantomData,
//         }
//     }
// }
//
// impl BehaviourBuilder<WithRelayClient> {
//     pub(crate) fn with_dcutr(mut self, dcutr: DcutrBehaviour) -> BehaviourBuilder<WithPrivate> {
//         self.dcutr = Some(dcutr).into();
//         BehaviourBuilder {
//             kad: self.kad,
//             identify: self.identify,
//             ping: self.ping,
//             autonat: self.autonat,
//             relay_server: self.relay_server,
//             dcutr: self.dcutr,
//             relay_client: self.relay_client,
//             _pd: std::marker::PhantomData,
//         }
//     }
// }
//
// impl BehaviourBuilder<WithPrivate> {
//     pub(crate) fn build(self) -> Behaviour {
//         Behaviour {
//             kad: self.kad,
//             identify: self.identify,
//             ping: self.ping,
//             autonat: self.autonat,
//             relay_server: self.relay_server,
//             dcutr: self.dcutr,
//             relay_client: self.relay_client,
//         }
//     }
// }
//
// impl BehaviourBuilder<WithPublic> {
//     pub(crate) fn build(self) -> Behaviour {
//         Behaviour {
//             kad: self.kad,
//             identify: self.identify,
//             ping: self.ping,
//             autonat: self.autonat,
//             relay_server: self.relay_server,
//             dcutr: self.dcutr,
//             relay_client: self.relay_client,
//         }
//     }
// }
//
// struct WithRelayClient;
// struct WithDcutr;
//
// struct WithPublic;
// struct WithPrivate;
