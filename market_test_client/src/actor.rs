use anyhow::Result;
use std::fmt::Display;

use market_proto::market_proto_rpc::{
    market_client::MarketClient, CheckHoldersRequest, RegisterFileRequest, User,
};
use std::str::FromStr;
use strum_macros::EnumString;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedReceiver;
use tonic::transport::Uri;

use crate::{
    cli::{Port, LOOPBACK_ADDR},
    ActorMarketState, ActorMarketStateLockCond,
};

#[derive(Debug)]
pub struct Actor {
    user: User,
    receiver: UnboundedReceiver<Command>,
}

impl Actor {
    pub fn new(user: User, receiver: UnboundedReceiver<Command>) -> Self {
        Actor { user, receiver }
    }

    pub async fn run(
        mut self,
        market_port: Port,
        lock_cond: ActorMarketStateLockCond,
    ) -> Result<()> {
        let uri = Uri::builder()
            .scheme("http")
            .authority(format!("{}:{}", LOOPBACK_ADDR, market_port).as_str())
            .path_and_query("/")
            .build()?;
        let (lock, cvar) = &*lock_cond;
        match MarketClient::connect(uri).await {
            Ok(mut client) => {
                {
                    let mut state = lock.lock().unwrap();
                    *state = ActorMarketState::Connected;
                    // I mean one works since there's only one thread for the cvar anyways
                    cvar.notify_all();
                }

                while let Some(cmd) = self.receiver.recv().await {
                    match cmd {
                        Command::Quit => break,
                        Command::Help => {
                            // TODO: maybe make it more modular by enumiter in command
                            println!(
                        "Available commands: quit, register <file_hash>, check <file_hash>, help"
                    );
                        }
                        Command::RegisterFile { file_hash } => {
                            let res = client
                                .register_file(RegisterFileRequest {
                                    user: Some(self.user.clone()),
                                    file_hash,
                                })
                                .await;
                            match res {
                                Ok(_) => {
                                    println!("Successfully registered file!")
                                }
                                Err(err) => {
                                    eprintln!("Failed to register file: {err}");
                                }
                            };
                        }
                        Command::CheckHolders { file_hash } => {
                            let res = client
                                .check_holders(CheckHoldersRequest { file_hash })
                                .await;
                            match res {
                                Ok(res) => {
                                    let res = res.into_inner();
                                    println!("Holders: {:?}", res.holders);
                                }
                                Err(err) => {
                                    eprintln!("Failed to find holders: {err}")
                                }
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to connect to market: {err}");
                let mut state = lock.lock().unwrap();
                *state = ActorMarketState::FailedToConnect;
                // I mean one works since there's only one thread for the cvar anyways
                cvar.notify_all();
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
pub enum Command {
    #[strum(serialize = "quit")]
    Quit,
    #[strum(serialize = "register")]
    RegisterFile { file_hash: String },
    #[strum(serialize = "check")]
    CheckHolders { file_hash: String },
    #[strum(serialize = "help")]
    Help,
}

#[derive(Debug, Clone)]
pub struct Message(String);

impl Message {
    pub fn new(line: String) -> Self {
        Message(line.to_lowercase())
    }

    pub fn into_command(self) -> Result<Command, CommandParseError> {
        self.try_into()
    }
}

impl TryFrom<Message> for Command {
    type Error = CommandParseError;
    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let trimmed = value.0.trim();
        let mut iter = trimmed.split_whitespace();
        let cmd = iter.next().ok_or(CommandParseError::NoCommand)?;
        let cmd = Command::from_str(cmd).map_err(|_| CommandParseError::NotFound {
            cmd: cmd.to_owned(),
        })?;
        match cmd {
            Command::Quit => Ok(cmd),
            Command::Help => Ok(cmd),
            Command::RegisterFile { file_hash: _ } => {
                let file_hash = iter
                    .next()
                    .ok_or(CommandParseError::MissingOrInvalidArgs { cmd })?;
                Ok(Command::RegisterFile {
                    file_hash: file_hash.to_owned(),
                })
            }
            Command::CheckHolders { file_hash: _ } => {
                let file_hash = iter
                    .next()
                    .ok_or(CommandParseError::MissingOrInvalidArgs { cmd })?;
                Ok(Command::CheckHolders {
                    file_hash: file_hash.to_owned(),
                })
            }
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum CommandParseError {
    #[error("No command provided")]
    NoCommand,
    #[error("Command {cmd} not found")]
    NotFound { cmd: String },
    #[error("Missing or invalid arg for command {cmd:?}")]
    MissingOrInvalidArgs { cmd: Command },
}

#[cfg(test)]
mod tests {
    use crate::actor::Command;

    use pretty_assertions::assert_eq;

    use super::Message;
    #[test]
    fn test_message_quit_command_conversion() {
        let cmd = Message::new("quit".to_owned()).into_command().unwrap();
        assert_eq!(cmd, Command::Quit);
    }

    #[test]
    #[should_panic]
    fn test_message_unknown_command_conversion() {
        let _ = Message::new("".to_owned()).into_command().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_message_register_no_args_command_conversion() {
        let _ = Message::new("register".to_owned()).into_command().unwrap();
    }

    #[test]
    fn test_message_register_args_command_conversion() {
        let register = Message::new("register sample_hash".to_owned())
            .into_command()
            .unwrap();
        assert_eq!(
            register,
            Command::RegisterFile {
                file_hash: "sample_hash".to_owned()
            }
        );
    }

    #[test]
    fn test_message_check_holders_args_command_conversion() {
        let request = Message::new("check sample_hash".to_owned())
            .into_command()
            .unwrap();
        assert_eq!(
            request,
            Command::CheckHolders {
                file_hash: "sample_hash".to_owned()
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_message_check_holders_no_args_command_conversion() {
        let _ = Message::new("check".to_owned()).into_command().unwrap();
    }
}
