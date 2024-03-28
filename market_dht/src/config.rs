use crate::boot_nodes::BootNodes;
use crate::Multiaddr;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    boot_nodes: Option<BootNodes>,
    listener: Multiaddr,
}

impl Config {
    pub fn builder(listener: Multiaddr) -> ConfigBuilder {
        ConfigBuilder::new(listener)
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
    listener: Multiaddr,
}

impl ConfigBuilder {
    fn new(listener: Multiaddr) -> Self {
        Self {
            boot_nodes: None,
            listener,
        }
    }
    pub fn boot_nodes(mut self, boot_nodes: BootNodes) -> Self {
        self.boot_nodes = Some(boot_nodes);
        self
    }

    pub fn build(self) -> Config {
        Config {
            boot_nodes: self.boot_nodes,
            listener: self.listener,
        }
    }
}
