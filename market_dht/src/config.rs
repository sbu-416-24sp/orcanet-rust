use std::{fmt::Debug, net::Ipv4Addr, time::Duration};

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};

use crate::lmm::FILE_DEFAULT_TTL;

const DEFAULT_COORDINATOR_THREAD_NAME: &str = "coordinator";
const DEFAULT_PEER_TCP_PORT: u16 = 0;

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) peer_tcp_port: u16,
    pub(crate) boot_nodes: Option<BootNodes>,
    pub(crate) coordinator_thread_name: String,
    pub(crate) file_ttl: Duration,
}

impl Config {
    #[inline(always)]
    pub const fn builder() -> ConfigBuilder {
        ConfigBuilder {
            peer_tcp_port: None,
            boot_nodes: None,
            coordinator_thread_name: None,
            file_ttl: None,
        }
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            peer_tcp_port: DEFAULT_PEER_TCP_PORT,
            boot_nodes: None,
            coordinator_thread_name: DEFAULT_COORDINATOR_THREAD_NAME.to_owned(),
            file_ttl: FILE_DEFAULT_TTL,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    peer_tcp_port: Option<u16>,
    boot_nodes: Option<BootNodes>,
    coordinator_thread_name: Option<String>,
    file_ttl: Option<Duration>,
}

impl ConfigBuilder {
    pub const fn set_peer_tcp_port(mut self, port: u16) -> Self {
        self.peer_tcp_port = Some(port);
        self
    }

    pub fn set_boot_nodes(mut self, boot_nodes: BootNodes) -> Self {
        self.boot_nodes = Some(boot_nodes);
        self
    }

    pub fn set_coordinator_thread_name(mut self, name: String) -> Self {
        self.coordinator_thread_name = Some(name);
        self
    }

    pub const fn set_file_ttl(mut self, ttl: Duration) -> Self {
        self.file_ttl = Some(ttl);
        self
    }

    pub fn build(self) -> Config {
        Config {
            peer_tcp_port: self.peer_tcp_port.unwrap_or(DEFAULT_PEER_TCP_PORT),
            boot_nodes: self.boot_nodes,
            coordinator_thread_name: self
                .coordinator_thread_name
                .unwrap_or(DEFAULT_COORDINATOR_THREAD_NAME.to_owned()),
            file_ttl: self.file_ttl.unwrap_or(FILE_DEFAULT_TTL),
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn get_kad_addrs(&self) -> Vec<(PeerId, Multiaddr)> {
        // TODO: should optimize this to return lazily?
        self.inner
            .iter()
            .filter_map(|addr| {
                let peer_id = addr
                    .iter()
                    .find(|proto| matches!(proto, Protocol::P2p(_)))
                    .expect("to not fail unless try_with_nodes didn't catch this");
                let ip4 = addr
                    .iter()
                    .find(|proto| matches!(proto, Protocol::Ip4(_)))
                    .expect("to not fail unless try_with_nodes didn't catch this");
                let ip4 = Multiaddr::from(ip4);
                if let Protocol::P2p(peer_id) = peer_id {
                    Some((peer_id, ip4))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn try_with_nodes<TNode: TryInto<Multiaddr>>(
        nodes: impl IntoIterator<Item = TNode>,
    ) -> Result<Self, BootNodesError<TNode>> {
        let maybe_nodes: Result<Vec<Multiaddr>, BootNodesError<TNode>> = nodes
            .into_iter()
            .map(|node: TNode| match node.try_into() {
                Ok(node) => {
                    let node: Multiaddr = node;
                    if node.iter().any(|proto| matches!(proto, Protocol::P2p(_)))
                        && node.iter().any(|proto| matches!(proto, Protocol::Ip4(_)))
                    {
                        Ok(node)
                    } else {
                        Err(BootNodesError::MissingRequiredProtocol)
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
    MissingRequiredProtocol,
    Empty,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    #[should_panic]
    fn test_with_boot_nodes_is_empty() {
        let boot_nodes = BootNodes::with_nodes(Vec::<Multiaddr>::new());
    }

    #[test]
    fn test_boot_nodes_invalid_multiaddr() {
        let res = BootNodes::try_with_nodes(vec!["/ip4/"]);
        assert!(matches!(res, Err(BootNodesError::InvalidMultiaddr(_))));
    }

    #[test]
    fn test_boot_nodes_fail_from_one_bad() {
        let res = BootNodes::try_with_nodes(vec![
            "/ip4/127.0.0.1/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
            "/ip4/127.0.0.1/p2p",
            "/ip4/127.0.0.1/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
        ]);
        assert_eq!(
            matches!(res, Err(BootNodesError::InvalidMultiaddr(_))),
            true
        );
    }

    #[test]
    fn test_boot_nodes_no_peer_id() {
        let res = BootNodes::try_with_nodes(vec!["/ip4/127.0.0.1"]);
        assert!(matches!(res, Err(BootNodesError::MissingRequiredProtocol)));
    }

    #[test]
    fn test_boot_nodes() {
        let res = BootNodes::try_with_nodes(vec![
            "/ip4/127.0.0.1/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
            "/ip4/127.0.0.1/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
            "/ip4/127.0.0.1/p2p/12D3KooWEpLeeMwsMtd6F91z4DEVjt395TvEx3Dv2i833StaFGdQ",
        ])
        .unwrap();
        assert_eq!(res.len(), 3);
    }
}
