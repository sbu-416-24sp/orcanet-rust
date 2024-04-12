use crate::boot_nodes::BootNodes;
use crate::multiaddr;
use crate::Multiaddr;

const BRIDGE_THREAD_NAME: &str = "coordinator_netbridge_thread";

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    pub(crate) boot_nodes: Option<BootNodes>,
    pub(crate) listener: Multiaddr,
    pub(crate) thread_name: String,
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

    pub fn thread_name(&self) -> &str {
        &self.thread_name
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConfigBuilder {
    boot_nodes: Option<BootNodes>,
    listener: Option<Multiaddr>,
    thread_name: Option<String>,
}

impl ConfigBuilder {
    const fn new() -> Self {
        Self {
            boot_nodes: None,
            listener: None,
            thread_name: None,
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

    pub fn with_thread_name(mut self, thread_name: String) -> Self {
        self.thread_name = Some(thread_name);
        self
    }

    pub fn build(self) -> Config {
        Config {
            boot_nodes: self.boot_nodes,
            listener: self
                .listener
                .unwrap_or(multiaddr!(Ip4([0, 0, 0, 0]), Tcp(0u16))),
            thread_name: self
                .thread_name
                .unwrap_or_else(|| BRIDGE_THREAD_NAME.to_owned()),
        }
    }
}
