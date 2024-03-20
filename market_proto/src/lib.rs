pub use self::gen::*;

mod gen {
    pub mod market_proto_rpc {
        tonic::include_proto!("market");
    }
}
