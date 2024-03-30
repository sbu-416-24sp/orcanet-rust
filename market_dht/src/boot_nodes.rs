use std::{
    error::Error as StdError,
    fmt::{Display, Formatter},
    ops::Deref,
};

use thiserror::Error;

use crate::{Multiaddr, PeerId};

// FIXIT: fix the generic types here and can prob be more modular

#[derive(Debug, Clone)]
pub struct BootNodes(pub(crate) Vec<BootNode>);

impl BootNodes {
    pub fn new<TNode, TIter>(iter: TIter) -> Result<Self, NodesError>
    where
        TNode: TryInto<BootNode>,
        TNode::Error: StdError + Send + Sync + 'static,
        TIter: IntoIterator<Item = TNode>,
    {
        let boot_nodes = iter
            .into_iter()
            .filter_map(|elem| elem.try_into().ok())
            .collect::<Vec<BootNode>>();
        if boot_nodes.is_empty() {
            Err(NodesError::Failed {
                reason: "no nodes converted successfully".to_string(),
            })
        } else {
            Ok(Self(boot_nodes))
        }
    }

    pub fn iter(&self) -> BootNodesIter {
        BootNodesIter {
            inner: self.0.iter(),
        }
    }
}

impl From<BootNodes> for Vec<BootNode> {
    fn from(value: BootNodes) -> Self {
        value.0
    }
}

impl Deref for BootNodes {
    type Target = [BootNode];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct BootNodesIter<'a> {
    inner: std::slice::Iter<'a, BootNode>,
}

impl<'a> Iterator for BootNodesIter<'a> {
    type Item = &'a BootNode;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl IntoIterator for BootNodes {
    type Item = BootNode;
    type IntoIter = std::vec::IntoIter<BootNode>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> TryFrom<Vec<T>> for BootNodes
where
    T: TryInto<BootNode>,
    T::Error: StdError + Send + Sync + 'static,
{
    type Error = NodesError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BootNode {
    pub(crate) addr: Multiaddr,
    pub(crate) peer_id: PeerId,
}

impl Display for BootNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Addr: {}, PeerId: {}", self.addr, self.peer_id)
    }
}

impl BootNode {
    pub const fn new(addr: Multiaddr, peer_id: PeerId) -> Self {
        BootNode { addr, peer_id }
    }
}

impl From<(Multiaddr, PeerId)> for BootNode {
    fn from(value: (Multiaddr, PeerId)) -> Self {
        BootNode::new(value.0, value.1)
    }
}

impl<'a, 'b> TryFrom<(&'a str, &'b str)> for BootNode {
    type Error = BootNodeError<&'a str, &'b str>;
    fn try_from(value: (&'a str, &'b str)) -> Result<Self, Self::Error> {
        Ok(BootNode::new(
            value
                .0
                .parse::<Multiaddr>()
                .map_err(|err| BootNodeError::Invalid {
                    reason: err.to_string(),
                    multiaddr: value.0,
                    peer_id: value.1,
                })?,
            value
                .1
                .parse::<PeerId>()
                .map_err(|err| BootNodeError::Invalid {
                    reason: err.to_string(),
                    multiaddr: value.0,
                    peer_id: value.1,
                })?,
        ))
    }
}

impl TryFrom<(String, String)> for BootNode {
    type Error = BootNodeError<String, String>;
    fn try_from(value: (String, String)) -> Result<Self, Self::Error> {
        let (addr, peer_id) = (value.0.as_str(), value.1.as_str());
        match BootNode::try_from((addr, peer_id)) {
            Ok(bn) => Ok(bn),
            Err(err) => Err(BootNodeError::Invalid {
                reason: err.to_string(),
                multiaddr: value.0,
                peer_id: value.1,
            }),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum BootNodeError<TMultiaddr, TPeerId> {
    #[error("Failed to parse: {reason}")]
    Invalid {
        reason: String,
        multiaddr: TMultiaddr,
        peer_id: TPeerId,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum NodesError {
    #[error("Failed to parse: {reason}")]
    Failed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::BootNodes;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_should_have_node() {
        let vc = vec![
            (
                "/ip4/127.0.0.1/tcp/1234",
                "QmX3YpKj7vB6qQbWxWb5q7K1E2bZqY7Z3g3t7Q8b1y6u6u",
            ),
            (
                "/ip4/127.0.0.1/tcp/1234",
                "QmX3YpKj7vB6qQbWxWb5q7K1E2bZqY7Z3g3t7Q8b1y6u6u",
            ),
        ];
        let bn = BootNodes::new(vc.clone()).unwrap();
        let expected_addr = vc[0].0.parse::<libp2p::Multiaddr>().unwrap();
        let id = vc[0].1.parse::<libp2p::PeerId>().unwrap();
        assert_eq!(bn.0[0].addr, expected_addr);
        assert_eq!(bn.0[0].peer_id, id);
        assert_eq!(bn.0.len(), 2);
    }

    #[test]
    #[should_panic]
    fn test_should_panic_since_empty() {
        let vc: Vec<(&str, &str)> = vec![];
        BootNodes::new(vc).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_bad_addr() {
        let vc = vec![("bad", "QmX3YpKj7vB6qQbWxWb5q7K1E2bZqY7Z3g3t7Q8b1y6u6u")];
        BootNodes::new(vc).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_bad_id() {
        let vc = vec![("/ip4/127.0.0.1/tcp/1234", "bad")];
        BootNodes::new(vc).unwrap();
    }

    #[test]
    fn test_vec_into_bootnodes() {
        let vc = vec![
            (
                "/ip4/127.0.0.1/tcp/1234",
                "QmX3YpKj7vB6qQbWxWb5q7K1E2bZqY7Z3g3t7Q8b1y6u6u",
            ),
            (
                "/ip4/127.0.0.1/tcp/1234",
                "QmX3YpKj7vB6qQbWxWb5q7K1E2bZqY7Z3g3t7Q8b1y6u6u",
            ),
        ];
        let _: BootNodes = vc.try_into().unwrap();
    }

    #[test]
    fn test_iter() {
        let vc = vec![
            (
                "/ip4/127.0.0.1/tcp/1234",
                "QmX3YpKj7vB6qQbWxWb5q7K1E2bZqY7Z3g3t7Q8b1y6u6u",
            ),
            (
                "/ip4/127.0.0.1/tcp/1234",
                "QmX3YpKj7vB6qQbWxWb5q7K1E2bZqY7Z3g3t7Q8b1y6u6u",
            ),
        ];
        let len = vc.len();
        let bn = BootNodes::new(vc.clone()).unwrap();
        let iter = bn.iter().collect::<Vec<_>>();
        assert_eq!(len, iter.len());
    }
}
