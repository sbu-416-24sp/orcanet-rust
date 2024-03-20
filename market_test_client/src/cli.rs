use clap::Parser;
pub const LOOPBACK_ADDR: &str = "127.0.0.1";
pub const DEFAULT_PORT: &str = "8080";

pub type MarketServerPort = u16;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The port where the market server is listening on and the client should connect to.
    #[arg(short, long, default_value = DEFAULT_PORT)]
    pub port: MarketServerPort,
}
