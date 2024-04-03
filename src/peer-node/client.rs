mod consumer;
mod grpc;
mod producer;
mod store;

use std::io::{self, Write};

use anyhow::{anyhow, Result};
use clap::{arg, Command};
use store::Configurations;

fn cli() -> Command {
    Command::new("peernode")
        .about("Orcanet Peernode CLI")
        .no_binary_name(true)
        .ignore_errors(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("producer")
                .about("Producer node commands")
                .subcommand_required(true)
                .ignore_errors(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("register")
                        .about("Registers with all known market servers")
                        .arg(arg!(<PORT> "The port to run the HTTP server on").required(false)),
                )
                .subcommand(
                    Command::new("add")
                        .about("Registers a dir/file with the market server")
                        .arg(
                            arg!(<FILE_NAME> "The file or directory name to register")
                                .required(true),
                        )
                        .arg(arg!(<PRICE> "The price of the file").required(true))
                        .arg_required_else_help(true),
                ),
        )
        .subcommand(
            Command::new("consumer")
                .about("Consumer node commands")
                .subcommand_required(true)
                .ignore_errors(true)
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
                .subcommand_required(true)
                .ignore_errors(true)
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
                .subcommand(Command::new("ls").about("Lists all market servers")),
        )
}

#[tokio::main]
async fn main() {
    println!("Orcanet Peernode CLI: Type 'help' for a list of commands");
    let mut cli = cli();
    let help = cli.render_help();
    loop {
        // Print command prompt and get command    
        print!("> ");
        io::stdout().flush().expect("Couldn't flush stdout");
        let mut config = store::Configurations::new();
        let market = "localhost:50051".to_string();
        // take in user input, process it with cli, and then execute the command
        // if the user wants to exit, break out of the loop

        // take in user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        let matches = cli
            .clone()
            .get_matches_from(input.split_whitespace().collect::<Vec<&str>>());
        match handle_arg_matches(matches, &mut config, market).await {
            Ok(_) => {}
            Err(e) => eprintln!("\x1b[93mError:\x1b[0m {}\n{}", e, help),
        };
    }
}

async fn handle_arg_matches(
    matches: clap::ArgMatches,
    config: &mut Configurations,
    market: String,
) -> Result<()> {
    match matches.subcommand() {
        Some(("producer", producer_matches)) => {
            match producer_matches.subcommand() {
                Some(("register", register_matches)) => {
                    // register files with the market service
                    let port = match register_matches.get_one::<u16>("PORT") {
                        Some(port) => *port,
                        None => 8080,
                    };

                    producer::register_files(market, config.get_files(), port).await?;
                    Ok(())
                }
                Some(("add", add_matches)) => {
                    let file_name = match add_matches
                        .get_one::<String>("FILE_NAME")
                        .map(|s| s.as_str())
                    {
                        Some(file_name) => file_name,
                        _ => unreachable!(),
                    };
                    let price = match add_matches.get_one::<i64>("PRICE") {
                        Some(price) => *price,
                        _ => unreachable!(),
                    };
                    config.add_file(file_name.to_string(), price);
                    Ok(())
                }
                //  handle invalid subcommand
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        Some(("consumer", consumer_matches)) => {
            match consumer_matches.subcommand() {
                Some(("upload", upload_matches)) => {
                    println!("Upload command: {:?}", upload_matches);
                    // Add your implementation for the upload subcommand here
                    Ok(())
                }
                Some(("get", get_matches)) => {
                    println!("Get command: {:?}", get_matches);
                    // Add your implementation for the get subcommand here
                    Ok(())
                }
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        Some(("market", consumer_matches)) => {
            match consumer_matches.subcommand() {
                Some(("add", add_matches)) => {
                    let market_url = match add_matches
                        .get_one::<String>("MARKET_URL")
                        .map(|s| s.as_str())
                    {
                        Some(url) => url,
                        _ => unreachable!(),
                    };
                    config.add_market(market_url.to_string());
                    Ok(())
                }
                Some(("rm", add_matches)) => {
                    let market_url = match add_matches
                        .get_one::<String>("MARKET_URL")
                        .map(|s| s.as_str())
                    {
                        Some(url) => url,
                        _ => unreachable!(),
                    };
                    config.remove_market(market_url.to_string());
                    Ok(())
                }
                Some(("ls", _)) => {
                    // Add your implementation for the ls subcommand here
                    config.get_market();
                    for market in config.get_market() {
                        println!("{}", market);
                    }
                    Ok(())
                }
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        _ => Err(anyhow!("Invalid subcommand")),
    }
    // Ok(())/
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
