use std::net::Ipv4Addr;

use clap::Parser;

use crate::util::DEFAULT_MARKET_SERVER_PORT;

pub type Port = u16;
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Port where the local market server is listening on and the client should connect to
    // NOTE: Essentially, this is just a port to communicate with the market server over an IPC
    // mechanism like TCP sockets between two processes.
    #[arg(short, long, default_value = DEFAULT_MARKET_SERVER_PORT)]
    pub market_port: Port,
    /// Username of the user registering the file
    // NOTE: probably to be removed since we already have unique SHA256 peerIDs?
    #[arg(short, long)]
    pub username: String,
    /// The price of the file per MB
    // NOTE: protobuf writers set this to be i64
    #[arg(short, long)]
    pub price: u64,
    /// The ID of the peer. If not provided, then it is automatically generated
    #[arg(short, long)]
    pub id: Option<String>,
    /// Port where other consumer peer clients should connect to retrieve files
    #[arg(long)]
    pub client_port: Port,
    /// IP where other consumer peer clients should connect to retrieve files
    #[arg(long)]
    pub client_ip: Ipv4Addr,
}
