use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
};

use anyhow::Result;
use clap::Parser;
use tokio::{runtime::Runtime, sync::mpsc};

use market_proto::market_proto_rpc::User;

use crate::{
    actor::Actor,
    cli::{start_main_loop, Cli},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorMarketState {
    NotConnected,
    FailedToConnect,
    Connected,
}

pub type ActorMarketStateLockCond = Arc<(Mutex<ActorMarketState>, Condvar)>;

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
    // TODO: can prob maybe initialize market client here but idgaf atm anyways
    let (tx, rx) = mpsc::unbounded_channel();
    let lock_cond = Arc::new((Mutex::new(ActorMarketState::NotConnected), Condvar::new()));
    let actor_lock_cond = Arc::clone(&lock_cond);
    thread::scope(|s| {
        s.spawn(move || -> Result<()> {
            let actor = Actor::new(user, rx);
            // This thread will crash if market server is not running
            // Since main loop depends on this thread,
            Runtime::new()?.block_on(actor.run(market_port, actor_lock_cond))?;
            Ok(())
        });
        s.spawn(move || start_main_loop(tx, lock_cond).unwrap());
    });
    Ok(())
}

mod actor;
mod cli;
