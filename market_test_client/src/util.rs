use anyhow::Result;
use rustyline::error::ReadlineError;

use rustyline::DefaultEditor;
use tokio::sync::mpsc::UnboundedSender;

use crate::actor::{Command, Message};

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

const PROMPT: &str = ">> ";
