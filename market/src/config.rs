use std::{fmt::Debug, time::Duration};

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

use crate::lmm::FILE_DEFAULT_TTL;

const DEFAULT_COORDINATOR_THREAD_NAME: &str = "coordinator";
const DEFAULT_PEER_TCP_PORT: u16 = 16899;
const DEFAULT_BOOTSTRAP_TIME: Duration = Duration::from_secs(77);

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) peer_tcp_port: u16,
    pub(crate) boot_nodes: Option<BootNodes>,
    pub(crate) coordinator_thread_name: String,
    pub(crate) file_ttl: Duration,
    // NOTE: this is only useful for if you are a PUBLIC GENESIS NODE.
    // We require this at least for the public genesis node since it can't figure out itself if it
    // can be used as a relay server. By providing this, you are guaranteeing that the node you are
    // running on is reachable by other nodes since your node must be private.
    // However, the behaviour with adding this as an external address doesn't make much of a
    // difference(?). Will PROBABLY DEPRECATE/DISABLE THIS IN THE FUTURE.
    // If you don't provide this and your node is a public node that is opened on the port you
    // provided (or 16899 if you don't provide anything), then your node CANNOT BE USED as a relay
    // server. The only other way is if you can get another (public?) node to try and confirm this
    // public address.
    pub(crate) public_address: Option<Multiaddr>,
    pub(crate) bootstrap_time: Duration,
}

impl Config {
    #[inline(always)]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    #[inline(always)]
    pub const fn peer_tcp_port(&self) -> u16 {
        self.peer_tcp_port
    }

    #[inline(always)]
    pub const fn boot_nodes(&self) -> Option<&BootNodes> {
        self.boot_nodes.as_ref()
    }

    #[inline(always)]
    pub fn coordinator_thread_name(&self) -> &str {
        &self.coordinator_thread_name
    }

    #[inline(always)]
    pub const fn file_ttl(&self) -> Duration {
        self.file_ttl
    }

    #[inline(always)]
    pub const fn public_address(&self) -> Option<&Multiaddr> {
        self.public_address.as_ref()
    }

    #[inline(always)]
    pub const fn bootstrap_time(&self) -> Duration {
        self.bootstrap_time
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            peer_tcp_port: DEFAULT_PEER_TCP_PORT,
            boot_nodes: None,
            coordinator_thread_name: DEFAULT_COORDINATOR_THREAD_NAME.to_owned(),
            file_ttl: FILE_DEFAULT_TTL,
            public_address: None,
            bootstrap_time: DEFAULT_BOOTSTRAP_TIME,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    peer_tcp_port: Option<u16>,
    boot_nodes: Option<BootNodes>,
    coordinator_thread_name: Option<String>,
    file_ttl: Option<Duration>,
    public_address: Option<Multiaddr>,
    bootstrap_time: Option<Duration>,
}

impl ConfigBuilder {
    #[inline(always)]
    pub const fn set_peer_tcp_port(mut self, port: u16) -> Self {
        self.peer_tcp_port = Some(port);
        self
    }

    #[inline(always)]
    pub fn set_boot_nodes(mut self, boot_nodes: BootNodes) -> Self {
        self.boot_nodes = Some(boot_nodes);
        self
    }

    #[inline(always)]
    pub fn set_coordinator_thread_name(mut self, name: impl Into<String>) -> Self {
        self.coordinator_thread_name = Some(name.into());
        self
    }

    #[inline(always)]
    pub const fn set_file_ttl(mut self, ttl: Duration) -> Self {
        self.file_ttl = Some(ttl);
        self
    }

    #[inline(always)]
    pub fn set_public_address(mut self, addr: Multiaddr) -> Self {
        self.public_address = Some(addr);
        self
    }

    #[inline(always)]
    pub const fn set_bootstrap_time(mut self, time: Duration) -> Self {
        self.bootstrap_time = Some(time);
        self
    }

    #[inline(always)]
    pub fn build(self) -> Config {
        Config {
            peer_tcp_port: self.peer_tcp_port.unwrap_or(DEFAULT_PEER_TCP_PORT),
            boot_nodes: self.boot_nodes,
            coordinator_thread_name: self
                .coordinator_thread_name
                .unwrap_or(DEFAULT_COORDINATOR_THREAD_NAME.to_owned()),
            file_ttl: self.file_ttl.unwrap_or(FILE_DEFAULT_TTL),
            public_address: self.public_address,
            bootstrap_time: self.bootstrap_time.unwrap_or(DEFAULT_BOOTSTRAP_TIME),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootNodes {
    pub(crate) inner: Vec<Multiaddr>,
}

impl BootNodes {
    pub fn with_nodes<TNode: Into<Multiaddr> + Debug>(
        nodes: impl IntoIterator<Item = TNode>,
    ) -> BootNodes {
        Self::try_with_nodes(nodes)
            .expect("to fail if user does not provide required things for try_with_nodes")
    }

    pub fn get_kad_addrs(&self) -> impl Iterator<Item = (PeerId, Multiaddr)> + '_ {
        // TODO: should optimize this to return lazily?
        self.inner.iter().filter_map(|addr| {
            let peer_id = addr
                .iter()
                .find(|proto| matches!(proto, Protocol::P2p(_)))
                .expect("to not fail unless try_with_nodes didn't catch this");
            let ip4 = addr
                .iter()
                .find(|proto| matches!(proto, Protocol::Ip4(_)))
                .expect("to not fail unless try_with_nodes didn't catch this");
            let tcp = addr
                .iter()
                .find(|proto| matches!(proto, Protocol::Tcp(_)))
                .expect("to not fail unless try_with_nodes didn't catch this");
            let ip = Multiaddr::from(ip4).with(tcp);
            if let Protocol::P2p(peer_id) = peer_id {
                Some((peer_id, ip))
            } else {
                None
            }
        })
    }

    pub fn try_with_nodes<TNode: TryInto<Multiaddr>>(
        nodes: impl IntoIterator<Item = TNode>,
    ) -> Result<Self, BootNodesError<TNode>> {
        let maybe_nodes: Result<Vec<Multiaddr>, BootNodesError<TNode>> = nodes
            .into_iter()
            .map(|node: TNode| match node.try_into() {
                Ok(node) => {
                    let node: Multiaddr = node;
                    let supports_p2p = node.iter().any(|proto| matches!(proto, Protocol::P2p(_)));

                    let supports_ip4 = node.iter().any(|proto| matches!(proto, Protocol::Ip4(_)));
                    let supports_tcp = node.iter().any(|proto| matches!(proto, Protocol::Tcp(_)));
                    if !supports_p2p {
                        Err(BootNodesError::MissingRequiredProtocol(
                            RequiredProtocol::P2p,
                        ))
                    } else if !supports_ip4 {
                        Err(BootNodesError::MissingRequiredProtocol(
                            RequiredProtocol::Ip4,
                        ))
                    } else if !supports_tcp {
                        Err(BootNodesError::MissingRequiredProtocol(
                            RequiredProtocol::Tcp,
                        ))
                    } else {
                        Ok(node)
                    }
                }
                Err(err) => Err(BootNodesError::InvalidMultiaddr(err)),
            })
            .collect();
        match maybe_nodes {
            Ok(nodes) => {
                if nodes.is_empty() {
                    Err(BootNodesError::Empty)
                } else {
                    Ok(Self { inner: nodes })
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Multiaddr> {
        self.inner.iter()
    }

    pub fn add_node(&mut self, node: Multiaddr) {
        self.inner.push(node);
    }

    pub fn into_inner(self) -> Vec<Multiaddr> {
        self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl IntoIterator for BootNodes {
    type Item = Multiaddr;
    type IntoIter = std::vec::IntoIter<Multiaddr>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[derive(Debug)]
pub enum BootNodesError<TNode: TryInto<Multiaddr>> {
    InvalidMultiaddr(TNode::Error),
    MissingRequiredProtocol(RequiredProtocol),
    Empty,
}

#[derive(Debug)]
pub enum RequiredProtocol {
    Tcp,
    Ip4,
    P2p,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    #[should_panic]
    fn test_with_boot_nodes_is_empty() {
        BootNodes::with_nodes(Vec::<Multiaddr>::new());
    }

    #[test]
    fn test_boot_nodes_invalid_multiaddr() {
        let res = BootNodes::try_with_nodes(vec!["/ip4/"]);
        assert!(matches!(res, Err(BootNodesError::InvalidMultiaddr(_))));
    }

    #[test]
    fn test_boot_nodes_fail_from_one_bad() {
        let res = BootNodes::try_with_nodes(vec![
            "/ip4/127.0.0.1/tcp/4040/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
            "/ip4/127.0.0.1/p2p",
            "/ip4/127.0.0.1/tcp/4040/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
        ]);
        assert!(matches!(res, Err(BootNodesError::InvalidMultiaddr(_))),);
    }

    #[test]
    fn test_boot_nodes_no_peer_id() {
        let res = BootNodes::try_with_nodes(vec!["/ip4/127.0.0.1"]);
        assert!(matches!(
            res,
            Err(BootNodesError::MissingRequiredProtocol(_))
        ));
    }

    #[test]
    fn test_boot_nodes() {
        let res = BootNodes::try_with_nodes(vec![
            "/ip4/127.0.0.1/tcp/4040/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
            "/ip4/127.0.0.1/tcp/4040/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
            "/ip4/127.0.0.1/tcp/4040/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
        ])
        .unwrap();
        assert_eq!(res.len(), 3);
    }
}
