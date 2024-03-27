//! # A library for providing an interface to the Orcanet Kademlia DHT
#![warn(
    missing_debug_implementations,
    // missing_docs,
    // clippy::missing_errors_doc,
    // clippy::missing_panics_doc,
    // clippy::missing_const_for_fn
)]
#![deny(unsafe_code, unreachable_pub)]

pub use bridge::*;
pub use command::{CommandOk, CommandResult};

mod bridge;
pub(crate) mod command;
pub mod file;
