use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use gen::market_proto_rpc::User;

pub use self::gen::*;

mod gen {
    pub mod market_proto_rpc {
        tonic::include_proto!("market");
    }
}

impl User {
    // NOTE: why is port even 32 bits?
    pub fn new(id: String, name: String, ip: String, port: i32, price: i64) -> Self {
        User {
            id,
            name,
            ip,
            port,
            price,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Peer ID: {}, Username: {}, Consumer IP Addr: {}:{}, Price Per MB: ${}",
            self.id, self.name, self.ip, self.port, self.price
        )
    }
}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for User {}
