use clap::Parser;

pub type MarketServerPort = u16;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = "8080")]
    pub port: MarketServerPort,
}
