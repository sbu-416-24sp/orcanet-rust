use anyhow::Result;
use market_proto::market_proto_rpc::market_client::MarketClient;
use rustyline::error::ReadlineError;

use rustyline::DefaultEditor;
use tokio::sync::mpsc::UnboundedSender;
use tonic::transport::{Channel, Uri};

use crate::{
    actor::{Command, Message},
    cli::Port,
};

pub const LOOPBACK_ADDR: &str = "127.0.0.1";
pub const DEFAULT_MARKET_SERVER_PORT: &str = "8080";

pub fn start_main_loop(tx: UnboundedSender<Command>) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let line = rl.readline(PROMPT);
        match line {
            Ok(line) => {
                let msg = Message::new(line);
                match msg.into_command() {
                    Ok(cmd) => {
                        // bails when the receiver is dropped
                        if let Command::Quit = cmd {
                            tx.send(cmd)?;
                            break;
                        } else {
                            tx.send(cmd)?;
                        }
                    }
                    Err(err) => {
                        eprintln!("Error parsing command: {}", err);
                    }
                }
            }
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                let _ = tx.send(Command::Quit);
                break;
            }
            Err(err) => {
                eprintln!("Error reading line: {}", err);
                let _ = tx.send(Command::Quit);
                break;
            }
        }
    }
    Ok(())
}

pub async fn initialize_client(market_port: Port) -> Result<MarketClient<Channel>> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorMarketState {
    NotConnected,
    Connected,
}

const PROMPT: &str = ">> ";
