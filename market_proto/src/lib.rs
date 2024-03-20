use std::hash::{Hash, Hasher};

pub use self::gen::*;

mod gen {
    pub mod market_proto_rpc {
        tonic::include_proto!("market");
    }
}

impl Hash for market_proto_rpc::User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for market_proto_rpc::User {}
