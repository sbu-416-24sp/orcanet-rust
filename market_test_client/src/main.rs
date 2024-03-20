use std::thread;

use anyhow::Result;
use clap::Parser;
use tokio::{runtime::Runtime, sync::mpsc};

use market_proto::market_proto_rpc::User;

use crate::{
    actor::Actor,
    cli::{start_main_loop, Cli, LOOPBACK_ADDR},
};

fn main() -> Result<()> {
    // Market server is typically just a local server process that represents the DHT for the peer
    // node. Peer nodes then communicate through TCP sockets to the market server with the gRPC
    // method abstractions.
    let cli = Cli::parse();
    let user = User::new(
        cli.id.unwrap_or("test_id".to_owned()),
        cli.username,
        cli.client_ip.to_string(),
        cli.client_port as i32,
        i64::try_from(cli.price)?,
    );
    let market_port = cli.market_port;
    let (tx, rx) = mpsc::unbounded_channel();
    thread::scope(|s| {
        s.spawn(move || -> Result<()> {
            let actor = Actor::new(user, rx);
            Runtime::new()?.block_on(actor.run(market_port));
            Ok(())
        });
        s.spawn(move || start_main_loop(tx).unwrap());
    });
    Ok(())
}

mod actor;
mod cli;
