use crate::boot_nodes::BootNodes;
use crate::multiaddr;
use crate::Multiaddr;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    pub(crate) boot_nodes: Option<BootNodes>,
    pub(crate) listener: Multiaddr,
}

impl Config {
    pub const fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    pub const fn boot_nodes(&self) -> Option<&BootNodes> {
        self.boot_nodes.as_ref()
    }

    pub const fn listener(&self) -> &Multiaddr {
        &self.listener
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConfigBuilder {
    boot_nodes: Option<BootNodes>,
    listener: Option<Multiaddr>,
}

impl ConfigBuilder {
    const fn new() -> Self {
        Self {
            boot_nodes: None,
            listener: None,
        }
    }

    pub fn with_boot_nodes(mut self, boot_nodes: BootNodes) -> Self {
        self.boot_nodes = Some(boot_nodes);
        self
    }

    pub fn with_listener(mut self, listener: Multiaddr) -> Self {
        self.listener = Some(listener);
        self
    }

    pub fn build(self) -> Config {
        Config {
            boot_nodes: self.boot_nodes,
            listener: self
                .listener
                .unwrap_or(multiaddr!(Ip4([0, 0, 0, 0]), Tcp(0u16))),
        }
    }
}
