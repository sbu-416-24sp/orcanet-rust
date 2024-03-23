//! A library for providing an interface to the Orcanet Kademlia DHT
#![warn(missing_debug_implementations)]
#![deny(unsafe_code, unreachable_pub)]
// NOTE: possibly an extension to more protocols later on in libp2p? so may have to also refactor
// the name.

pub mod bridge;
pub(crate) mod command;
pub use command::{CommandOk, CommandResult};
