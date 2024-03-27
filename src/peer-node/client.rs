mod consumer;
mod grpc;
mod producer;

use anyhow::{anyhow, Result};
use clap::{arg, Arg, Command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

/// Peer node client
struct Args {
    /// Market service address
    #[arg(short, long, default_value = "localhost:50051")]
    market: String,

    /// Whether to run as a producer
    #[arg(short, long, default_value = "false")]
    producer: bool,

    /// File hash
    /// Only used when running as a consumer
    #[arg(short, long)]
    file_hash: Option<String>,
}

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
                        .about("Registers with a market server")
                        .arg(arg!(<SERVER> "The market to target"))
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
                                .help("Register all files in the current directory")
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
}

#[tokio::main]
async fn main() -> Result<()> {
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
