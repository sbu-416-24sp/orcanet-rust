mod consumer;
mod grpc;
mod producer;
mod store;

use anyhow::{anyhow, Result};
use clap::{arg, Arg, Command, Parser};
// use config::{builder, Config, File, FileFormat};


fn cli() -> Command {
    Command::new("peernode")
        .about("Orcanet Peernode CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("producer")
                .about("Producer node commands")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("register")
                        .about("Registers with all known market servers")
                        .arg(arg!(<SERVER> "The market to target"))
                        // TODO: ADD FILTER MECHANISM
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("add")
                        .about("Registers a file with the market server")
                        .arg(arg!(<FILE_NAME> "The file name to register").required(true))
                        .arg(
                            Arg::new("all")
                                .short('a')
                                .long("all")
                                .help("Register all files in the specified directory")
                                .required(false),
                        ),
                ),
        )
        .subcommand(
            Command::new("consumer")
                .about("Consumer node commands")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("upload")
                        .about("Uploads a file to a producer")
                        .arg(arg!(<FILE_NAME> "The file to upload").required(true))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("get")
                        .about("Downloads a file from a producer")
                        .arg(arg!(<FILE_HASH> "The hash of the file to download").required(true))
                        .arg_required_else_help(true),
                ),
        )
        .subcommand(
          Command::new("market")
              .about("market commands")
              .arg_required_else_help(true)
              .subcommand(
                  Command::new("add")
                      .about("Adds a new market server")
                      .arg(arg!(<MARKET_URL> "The new market server to add").required(true))
                      .arg_required_else_help(true),
              )
              .subcommand(
                Command::new("rm")
                    .about("Removes a market server")
                    .arg(arg!(<MARKET_URL> "The market server to remove").required(true))
                    .arg_required_else_help(true),
            )
              .subcommand(
                  Command::new("ls")
                      .about("Lists all market servers")
              ),
      )
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = store::Configurations::new();

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("producer", producer_matches)) => {
            match producer_matches.subcommand() {
                Some(("register", register_matches)) => {
                    println!("Register command: {:?}", register_matches);
                    // producer::run().await?;
                }
                Some(("add", add_matches)) => {
                    println!("Add command: {:?}", add_matches);
                    // Add your implementation for the add subcommand here
                }
                _ => unreachable!(), // If arg_required_else_help is set to true, this should never happen
            }
        }
        Some(("consumer", consumer_matches)) => {
            match consumer_matches.subcommand() {
                Some(("upload", upload_matches)) => {
                    println!("Upload command: {:?}", upload_matches);
                    // Add your implementation for the upload subcommand here
                }
                Some(("get", get_matches)) => {
                    println!("Get command: {:?}", get_matches);
                    // Add your implementation for the get subcommand here
                }
                _ => unreachable!(), // If arg_required_else_help is set to true, this should never happen
            }
        }
        Some(("market", consumer_matches)) => {
          match consumer_matches.subcommand() {
              Some(("add", add_matches)) => {
                  let market_url = match add_matches.get_one::<String>("MARKET_URL").map(|s| s.as_str()) {
                      Some(url) => url,
                      _ => unreachable!(),
                  };
                  config.add_market(market_url.to_string());
              }
              Some(("rm", add_matches)) => {
                let market_url = match add_matches.get_one::<String>("MARKET_URL").map(|s| s.as_str()) {
                    Some(url) => url,
                    _ => unreachable!(),
                };
                config.remove_market(market_url.to_string());
            }
              Some(("ls", _)) => {
                  // Add your implementation for the ls subcommand here
                  config.get_market();
                  for market in config.get_market() {
                      println!("{}", market);
                  }
              }
              _ => unreachable!(), // If arg_required_else_help is set to true, this should never happen
          }
      }
      _ => {
          eprintln!("Error: Unrecognized subcommand or missing required arguments.");
          std::process::exit(1); // Exit with non-zero status to indicate error
      }

    }
    Ok(())
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     let args: Args = Args::parse();

//     match args.producer {
//         true => producer::run(args.market).await?,
//         false => match args.file_hash {
//             Some(file_hash) => consumer::run(args.market, file_hash).await?,
//             None => return Err(anyhow!("No file hash provided")),
//         },
//     }

//     Ok(())
// }
