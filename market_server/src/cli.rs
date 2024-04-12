use clap::Parser;

use crate::Port;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = "8080")]
    pub market_port: Port,
    #[arg(short, long, default_value = "16899")]
    pub peer_port: Port,
    #[arg(short, long, value_parser, num_args = 0.., value_delimiter = '|')]
    pub boot_nodes: Option<Vec<String>>,
}
