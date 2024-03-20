use std::hash::{Hash, Hasher};

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

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for User {}
