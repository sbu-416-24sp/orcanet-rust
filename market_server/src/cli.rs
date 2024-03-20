use clap::Parser;

use crate::Port;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = "8080")]
    pub port: Port,
}
