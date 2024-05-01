use std::path::PathBuf;
use std::str::FromStr;

use peernode::consumer;
use peernode::consumer::encode;
use peernode::producer;
use peernode::store;

use anyhow::{anyhow, Result};
use clap::{arg, Command};
use orcanet_market::BootNodes;
use orcanet_market::Multiaddr;
use proto::market::FileInfoHash;
use store::Configurations;

#[cfg(test)]
mod tests;

// Create a CLI command with several subcommands to encapsulate the different functionalities of the peer node
pub fn cli() -> Command {
    Command::new("peernode")
        .about("Orcanet Peernode CLI")
        .no_binary_name(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("producer")
                .about("Producer node commands")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand_required(true)
                .subcommand(
                    Command::new("register")
                        .about("Registers with all known market servers")
                        .arg(arg!(<PORT> "The port to run the HTTP server on").required(false))
                        .arg(
                            arg!(<MARKET> "The market to connect to")
                                .required(false)
                                .short('m'),
                        )
                        .arg(
                            arg!(<IP> "The public IP address to announce")
                                .required(false)
                                .short('i')
                                .long("ip"),
                        ),
                )
                .subcommand(
                    Command::new("add")
                        .about("Adds a dir/file to be registered with the market server")
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
                .arg_required_else_help(true)
                .subcommand_required(true)
                .subcommand(
                    Command::new("consumer")
                        .about("Consumer node commands")
                        .arg_required_else_help(true)
                        .subcommand(
                            Command::new("send")
                                .about("transfer funds to another user")
                                .arg(arg!(<AMOUNT> "The amount to transfer").required(true))
                                .arg(arg!(<RECIPIENT> "The recipient of the funds").required(true))
                                .arg_required_else_help(true),
                        ),
                )
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
                        .arg(arg!(<PRODUCER> "The producer to download from").required(true))
                        .arg(
                            arg!(<CHUNK_NUM> "The chunk number to download")
                                .required(false)
                                .short('n'),
                        )
                        .arg(
                            arg!(<CONTINUE> "Continue downloading a file")
                                .required(false)
                                .short('c'),
                        ),
                ),
        )
        .subcommand(
            Command::new("market")
                .about("Market node commands")
                .subcommand_required(true)
                // .ignore_errors(true)
                .subcommand(
                    Command::new("set")
                        .about("Set the options for the market connection")
                        .arg_required_else_help(true)
                        .arg(
                            clap::Arg::new("BOOT_NODES")
                                .short('b')
                                .long("boot-nodes")
                                .value_name("BOOT_NODES")
                                .help("The bootstrap nodes to connect to")
                                .long_help(
"A space-separated list of Multiaddr for bootstrap nodes to connect to
e.g. -b /ip4/192.168.0.0.1/tcp/6881/peer_id_hash1 /ip4/127.0.0.2/tcp/6881/peer_id_hash2"
                                )
                                // optional
                                .num_args(0..)
                        )
                        .arg(
                            clap::Arg::new("PUBLIC_ADDRESS")
                                .short('p')
                                .long("public-address")
                                .value_name("PUBLIC_ADDRESS")
                                .num_args(0..=1)
                                .help("The optional public address to run a market server on")
                                .long_help(
"If this is provided, the market server will connect to the Kademlia network and serve data.
Otherwise, it will run in client mode and only retrieve data from the network.
The address must be provided as a Multiaddr,
e.g. /ip4/0.0.0.0/tcp/6881")
                    )
                ),
        )
        .subcommand(Command::new("exit").about("Exits the CLI"))
}

// Handle the different subcommands of the CLI and execute the appropriate functions
pub async fn handle_arg_matches(
    matches: clap::ArgMatches,
    config: &mut Configurations,
) -> Result<()> {
    // take in result to pass up and print errors to user directly
    match matches.subcommand() {
        // match subcommands, get the inputs, and execute the appropriate functions
        Some(("producer", producer_matches)) => {
            match producer_matches.subcommand() {
                Some(("register", register_matches)) => {
                    let files = config.get_files();
                    // register files with the market service
                    let port = match register_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => config.get_port(),
                    };
                    let market_client = config.get_market_client().await?;
                    let ip = match register_matches.get_one::<String>("IP") {
                        Some(ip) => Some(ip.clone()),
                        None => None,
                    };
                    producer::register_files(files, market_client, port.clone(), ip).await?;
                    config.start_http_client(port).await;
                    Ok(())
                }
                Some(("restart", restart_matches)) => {
                    // restart the HTTP server
                    let port = match restart_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => config.get_port(),
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
                            // eprintln!("Invalid price");
                            return Err(anyhow!("Invalid price"));
                        }
                    };
                    match config.add_file_path(&PathBuf::from(file_name), price).await {
                        Ok(num_added) => {
                            println!("{num_added} file(s) at {file_name} have been registered at price {price}")
                        }
                        Err(e) => println!("Failed to register file: {e}"),
                    }
                    // print
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
                    config.remove_file(&PathBuf::from(file_name)).await?;
                    Ok(())
                }
                Some(("ls", _)) => {
                    let files = config.get_files();
                    let prices = config.get_prices();

                    if files.is_empty() {
                        println!("No files registered!");
                    }
                    for (hash, info) in files {
                        println!(
                            "File: {}, Price: {}, Hash: {hash}",
                            info.path.to_string_lossy(),
                            *prices.get(&hash).unwrap_or(&0)
                        );
                    }
                    Ok(())
                }
                Some(("port", port_matches)) => {
                    let port = match port_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => Err(anyhow!("No port provided"))?,
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
                Some(("upload", _upload_matches)) => {
                    // Add your implementation for the upload subcommand here
                    Ok(())
                }
                Some(("send", _send_matches)) => {
                    // Add your implementation for the send subcommand here
                    Ok(())
                }
                Some(("ls", ls_matches)) => {
                    let file_hash = match ls_matches.get_one::<String>("FILE_HASH") {
                        Some(file_hash) => file_hash.clone(),
                        None => Err(anyhow!("No file hash provided"))?,
                    };
                    let market_client = config.get_market_client().await?;
                    let file_info_hash = FileInfoHash::new(file_hash);
                    consumer::list_producers(file_info_hash, market_client).await?;
                    Ok(())
                }
                // get file from producer
                Some(("get", get_matches)) => {
                    let file_hash = match get_matches.get_one::<String>("FILE_HASH") {
                        Some(file_hash) => file_hash.clone(),
                        None => Err(anyhow!("No file hash provided"))?,
                    };
                    let file_info_hash = FileInfoHash::new(file_hash);
                    let producer = match get_matches.get_one::<String>("PRODUCER") {
                        Some(producer) => producer.clone(),
                        None => Err(anyhow!("No producer provided"))?,
                    };
                    let producer_user = match encode::try_decode_user(&producer) {
                        Ok(user) => user,
                        Err(e) => Err(anyhow::anyhow!("Failed to decode producer: {e}"))?,
                    };
                    let producer = encode::verify_encoding(&producer)
                        .expect("We just successfully decoded it.");
                    let chunk_num = match get_matches.get_one::<u64>("CHUNK_NUM") {
                        Some(chunk_num) => *chunk_num,
                        None => 0,
                    };
                    let continue_download = match get_matches.get_one::<bool>("CONTINUE") {
                        Some(continue_download) => *continue_download,
                        None => true,
                    };
                    let token = config.get_token(producer.clone());
                    let ret_token = match consumer::get_file(
                        producer_user,
                        file_info_hash,
                        token,
                        chunk_num,
                        continue_download,
                    )
                    .await
                    {
                        Ok(token) => token,
                        Err(e) => {
                            match e.to_string().as_str() {
                                "Request failed with status code: 404" => {
                                    println!("Consumer: File downloaded successfully (404 EOF)");
                                }
                                _ => eprintln!("{e}"),
                            };
                            return Ok(());
                        }
                    };
                    config.set_token(producer, ret_token);
                    Ok(())
                }
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        Some(("market", market_matches)) => match market_matches.subcommand() {
            Some(("set", set_matches)) => {
                let boot_nodes = match set_matches.get_many::<String>("BOOT_NODES") {
                    Some(boot_nodes) => {
                        let parsed_addrs: Vec<_> = boot_nodes
                            .filter_map(|x| match Multiaddr::from_str(x) {
                                Ok(x) => Some(x),
                                Err(e) => {
                                    println!("Failed to parse multiaddr {x}: {e}");
                                    None
                                }
                            })
                            .collect();
                        println!("Boot nodes parsed: {parsed_addrs:?}");
                        if parsed_addrs.is_empty() {
                            None
                        } else {
                            Some(BootNodes::with_nodes(parsed_addrs))
                        }
                    }
                    None => {
                        let boot_nodes = config.get_boot_nodes();
                        println!("Using existing boot nodes {boot_nodes:?}");
                        boot_nodes
                    }
                };
                let public_address = match set_matches.get_many::<String>("PUBLIC_ADDRESS") {
                    Some(public_addr) => {
                        match public_addr
                            .filter_map(|x| match Multiaddr::from_str(x) {
                                Ok(x) => Some(x),
                                Err(e) => {
                                    println!("Failed to parse multiaddr {x}: {e}");
                                    None
                                }
                            })
                            .next()
                        {
                            Some(parsed_addr) => {
                                println!("Public address parsed: {:?}", parsed_addr);
                                Some(parsed_addr)
                            }
                            None => {
                                println!("Setting public address to None");
                                None
                            }
                        }
                    }
                    None => {
                        let public_address = config.get_public_address();
                        println!("Using existing public address {public_address:?}");
                        public_address
                    }
                };
                config.set_boot_nodes(boot_nodes);
                config.set_public_address(public_address);
                config.get_market_client().await?;
                Ok(())
            }
            _ => Err(anyhow!("Invalid subcommand")),
        },
        Some(("exit", _)) => Ok(()),
        _ => Err(anyhow!("Invalid subcommand")),
    }
}
