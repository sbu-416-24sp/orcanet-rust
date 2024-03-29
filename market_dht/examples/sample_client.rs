use market_dht::{config::Config, net::NetworkBridge};
fn main() {
    let config = Config::builder("/ip4/127.0.0.1/tcp/0".parse().unwrap()).build();
    let bridge = NetworkBridge::new(config).unwrap();
}
