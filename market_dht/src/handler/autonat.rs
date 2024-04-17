use libp2p::autonat::Event;

use crate::BootNodes;

use super::EventHandler;

pub(crate) struct AutoNatHandler<'a> {
    boot_nodes: Option<&'a BootNodes>,
}

impl<'a> AutoNatHandler<'a> {
    pub(crate) const fn new(boot_nodes: Option<&'a BootNodes>) -> Self {
        AutoNatHandler { boot_nodes }
    }
}

impl<'a> EventHandler for AutoNatHandler<'a> {
    type Event = Event;

    fn handle_event(&mut self, event: Self::Event) {
        match event {
            Event::InboundProbe(_) => todo!(),
            Event::OutboundProbe(_) => todo!(),
            Event::StatusChanged { old, new } => todo!(),
        }
    }
}
