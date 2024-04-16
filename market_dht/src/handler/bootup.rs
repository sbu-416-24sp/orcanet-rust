use std::cell::Cell;

use libp2p::{swarm::SwarmEvent, Swarm};

use crate::behaviour::{Behaviour, BehaviourEvent};

use super::EventHandler;

pub(crate) struct BootupHandler<'a, 'b> {
    swarm: &'a mut Swarm<Behaviour>,
    booting: &'b Cell<bool>,
}

impl<'a, 'b> BootupHandler<'a, 'b> {
    pub(crate) fn new(swarm: &'a mut Swarm<Behaviour>, booting: &'b Cell<bool>) -> Self {
        BootupHandler { swarm, booting }
    }
}

impl<'a, 'b> EventHandler for BootupHandler<'a, 'b> {
    type Event = SwarmEvent<BehaviourEvent>;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            SwarmEvent::Behaviour(_) => todo!(),
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
