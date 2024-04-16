use libp2p::{swarm::SwarmEvent, Swarm};

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
    type Event = SwarmEvent<BehaviourEvent>;

    fn handle_event(&mut self, event: Self::Event) {
        // NOTE:  maybe use  box,dyn but that would remove zca?
        match event {
            SwarmEvent::Behaviour(event) => match event {
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
            },
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => todo!(),
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => todo!(),
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => todo!(),
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => todo!(),
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => todo!(),
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => todo!(),
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => todo!(),
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => todo!(),
            SwarmEvent::ListenerError { listener_id, error } => todo!(),
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => todo!(),
            SwarmEvent::NewExternalAddrCandidate { address } => todo!(),
            SwarmEvent::ExternalAddrConfirmed { address } => todo!(),
            SwarmEvent::ExternalAddrExpired { address } => todo!(),
            _ => {}
        }
    }
}

impl<'a, 'b> CommandRequestHandler for Handler<'a, 'b> {}

mod autonat;
pub(crate) mod bootup;
mod dcutr;
mod identify;
mod kad;
mod ping;
mod relay_client;
mod relay_server;
