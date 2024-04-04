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
                // .arg_required_else_help(true)
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
                )
                .subcommand(
                    Command::new("rm")
                        .about("Removes a file from the market server")
                        .arg(arg!(<FILE_NAME> "The file to remove").required(true))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("restart")
                        .about("Restarts the HTTP server")
                        .arg(arg!(<PORT> "The port to run the HTTP server on").required(false)),
                )
                .subcommand(Command::new("kill").about("Kills the HTTP server"))
                .subcommand(
                    Command::new("port")
                        .about("Sets the port for the HTTP server")
                        .arg(arg!(<PORT> "The port to run the HTTP server on").required(true)),
                )
                .subcommand(
                    Command::new("ls").about("Lists all files registered with the market server"),
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
                    Command::new("ls")
                        .about("Lists all producers with a file")
                        .arg(arg!(<FILE_HASH> "The hash of the file to list").required(true))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("get")
                        .about("Downloads a file from a producer")
                        .arg(arg!(<FILE_HASH> "The hash of the file to download").required(true))
                        .arg_required_else_help(true),
                ),
        )
        .subcommand(Command::new("exit").about("Exits the CLI"))
}

#[tokio::main]
async fn main() {
    println!("Orcanet Peernode CLI: Type 'help' for a list of commands");
    let mut cli = cli();
    let help = cli.render_help();

    // Load the configuration
    let mut config = store::Configurations::new().await;
    loop {
        // Print command prompt and get command
        // print!("> ");
        io::stdout().flush().expect("Couldn't flush stdout");
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
    // file_map: Arc<producer::files::FileMap>,
) -> Result<()> {
    match matches.subcommand() {
        Some(("producer", producer_matches)) => {
            match producer_matches.subcommand() {
                Some(("register", register_matches)) => {
                    // register files with the market service
                    let port = match register_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => String::from("8080"),
                    };
                    producer::register_files(config.get_prices(), market, port.clone()).await?;
                    config.start_http_client(port).await;
                    Ok(())
                }
                Some(("restart", restart_matches)) => {
                    // restart the HTTP server
                    let port = match restart_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => String::from("8080"),
                    };
                    config.start_http_client(port).await;
                    Ok(())
                }
                Some(("kill", _)) => {
                    // kill the HTTP server
                    config.stop_http_client().await;
                    Ok(())
                }
                Some(("add", add_matches)) => {
                    let file_name = match add_matches
                        .get_one::<String>("FILE_NAME")
                        .map(|s| s.as_str())
                    {
                        Some(file_name) => file_name,
                        _ => Err(anyhow!("Invalid file name"))?,
                    };
                    let price = match add_matches.get_one::<String>("PRICE") {
                        Some(price) => price,
                        _ => Err(anyhow!("Invalid price"))?,
                    };
                    // get i64 price
                    let price = match price.parse::<i64>() {
                        Ok(price) => price,
                        Err(_) => {
                            eprintln!("Invalid price");
                            return Ok(());
                        }
                    };
                    config.add_file_path(file_name.to_string(), price);
                    Ok(())
                }
                Some(("rm", rm_matches)) => {
                    let file_name = match rm_matches
                        .get_one::<String>("FILE_NAME")
                        .map(|s| s.as_str())
                    {
                        Some(file_name) => file_name,
                        _ => Err(anyhow!("Invalid file name"))?,
                    };
                    config.remove_file(file_name.to_string());
                    Ok(())
                }
                Some(("ls", _)) => {
                    let files = config.get_files();
                    let prices = config.get_prices();

                    for (hash, path) in files {
                        println!(
                            "File: {}, Price: {}",
                            path.to_string_lossy(),
                            *prices.get(&hash).unwrap_or(&0)
                        );
                    }
                    Ok(())
                }
                Some(("port", port_matches)) => {
                    let port = match port_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => String::from("8080"),
                    };
                    config.set_port(port);
                    Ok(())
                }
                //  handle invalid subcommand
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        Some(("consumer", consumer_matches)) => {
            match consumer_matches.subcommand() {
                Some(("upload", upload_matches)) => {
                    // Add your implementation for the upload subcommand here
                    Ok(())
                }
                Some(("ls", ls_matches)) => {
                    let file_hash = match ls_matches.get_one::<String>("FILE_HASH") {
                        Some(file_hash) => file_hash.clone(),
                        None => Err(anyhow!("No file hash provided"))?,
                    };
                    consumer::list_producers(file_hash, market).await?;
                    Ok(())
                }
                Some(("get", get_matches)) => {
                    let file_hash = match get_matches.get_one::<String>("FILE_HASH") {
                        Some(file_hash) => file_hash.clone(),

                        None => Err(anyhow!("No file hash provided"))?,
                    };
                    consumer::run(market, file_hash).await?;
                    Ok(())
                }
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        Some(("exit", _)) => Ok(()),
        _ => Err(anyhow!("Invalid subcommand")),
    }
}
