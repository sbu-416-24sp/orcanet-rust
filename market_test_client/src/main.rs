use std::thread;

use anyhow::Result;
use clap::Parser;
use cli::{Port, LOOPBACK_ADDR};
use tokio::{
    runtime::Runtime,
    sync::{mpsc, oneshot},
};

use market_proto::market_proto_rpc::{market_client::MarketClient, User};
use tonic::transport::{Channel, Uri};

use crate::{
    actor::Actor,
    cli::{start_main_loop, Cli},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorMarketState {
    NotConnected,
    Connected,
}

fn main() -> Result<()> {
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
    let (m_state_tx, m_state_rx) = oneshot::channel::<ActorMarketState>();

    thread::scope(|s| {
        s.spawn(move || -> Result<()> {
            let actor = Actor::new(user, rx);
            Runtime::new()?.block_on(async {
                if let Ok(client) = initialize_client(market_port).await {
                    if let Ok(()) = m_state_tx.send(ActorMarketState::Connected) {
                        actor.run(client).await;
                    }
                } else {
                    m_state_tx.send(ActorMarketState::NotConnected).unwrap();
                }
            });
            Ok(())
        });
        s.spawn(move || {
            println!("Waiting for actor client to connect to market server...");
            if let ActorMarketState::Connected = m_state_rx.blocking_recv().expect(
                "We should be expecting that it does send something so this shouldn't panic, but panics if sender doesn't send something",
            ) {
                println!("Market client connected!");
                start_main_loop(tx).unwrap();
            } else {
                eprintln!("Failed to connect to market server!");
            }

        });
    });
    Ok(())
}

async fn initialize_client(market_port: Port) -> Result<MarketClient<Channel>> {
    // Market server is typically just a local server process that represents the DHT for the peer
    // node. Peer nodes then communicate through TCP sockets to the market server with the gRPC
    // method abstractions. So we can just use the loopback address; eventually I believe we
    // combine it with the peer node team and get rid of the gRPC socket overhead
    let uri = Uri::builder()
        .scheme("http")
        .authority(format!("{}:{}", LOOPBACK_ADDR, market_port).as_str())
        .path_and_query("/")
        .build()?;
    MarketClient::connect(uri)
        .await
        .map_err(|err| anyhow::anyhow!(err))
}

mod actor;
mod cli;
