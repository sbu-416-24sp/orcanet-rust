use libp2p::Swarm;

use crate::{behaviour::Behaviour, lmm::LocalMarketMap};

use self::{
    autonat::AutoNatHandler, dcutr::DcutrHandler, identify::IdentifyHandler, kad::KadHandler,
    ping::PingHandler, relay_client::RelayClientHandler, relay_server::RelayServerHandler,
};

use super::behaviour::BehaviourEvent;

pub(crate) trait EventHandler {
    type Event;
    fn handle_event(&mut self, event: Self::Event);
}

pub(crate) trait CommandRequestHandler {}

pub(crate) struct Handler<'a, 'b> {
    swarm: &'a mut Swarm<Behaviour>,
    lmm: &'b mut LocalMarketMap,
}

impl<'a, 'b> Handler<'a, 'b> {
    pub(crate) fn new(swarm: &'a mut Swarm<Behaviour>, lmm: &'b mut LocalMarketMap) -> Self {
        Handler { swarm, lmm }
    }
}

impl<'a, 'b> EventHandler for Handler<'a, 'b> {
    type Event = BehaviourEvent;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            BehaviourEvent::Kad(event) => {
                let mut kad_handler = KadHandler::new(self.swarm, self.lmm);
                kad_handler.handle_event(event);
            }
            BehaviourEvent::Identify(event) => {
                let mut identify_handler = IdentifyHandler::new(self.swarm);
                identify_handler.handle_event(event);
            }
            BehaviourEvent::Ping(event) => {
                let mut ping_handler = PingHandler {};
                ping_handler.handle_event(event);
            }
            BehaviourEvent::Autonat(event) => {
                let mut autonat_handler = AutoNatHandler {};
                autonat_handler.handle_event(event);
            }
            BehaviourEvent::RelayServer(event) => {
                let mut relay_server_handler = RelayServerHandler {};
                relay_server_handler.handle_event(event);
            }
            BehaviourEvent::Dcutr(event) => {
                let mut dcutr_handler = DcutrHandler {};
                dcutr_handler.handle_event(event);
            }
            BehaviourEvent::RelayClient(event) => {
                let mut relay_client = RelayClientHandler {};
                relay_client.handle_event(event);
            }
        }
    }
}

impl<'a, 'b> CommandRequestHandler for Handler<'a, 'b> {}

mod autonat;
mod dcutr;
mod identify;
mod kad;
mod ping;
mod relay_client;
mod relay_server;
