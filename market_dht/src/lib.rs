#![warn(
    missing_debug_implementations,
    // missing_docs,
    // clippy::missing_docs_in_private_items,
    // clippy::missing_errors_doc,
    // clippy::missing_panics_doc,
    clippy::missing_const_for_fn
)]
#![deny(unsafe_code, unreachable_pub)]

pub use libp2p::multiaddr::{multiaddr, Protocol};
pub use libp2p::Multiaddr;
pub use libp2p::PeerId;
pub use req_res::{FileReqResResponseData, KadResponseData, ResponseData};

pub mod boot_nodes;
pub mod config;
pub mod net;
pub mod peer;

mod behaviour;
mod coordinator;
mod req_res;
