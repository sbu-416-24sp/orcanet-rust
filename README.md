# Orcanet Rust

Orcanet is a distributed file network which allows users to buy and sell files based on the hash of their data, with built in checksums per chunk to prevent scams or data corruption. Payment is managed through the Orcacoin fork of the Bitcoin network.

This repository contains the Rust implementation of the peernode-market and provides a CLI and http server.

## Todo (for future semesters?)
- Implement the coin
- Implement the wallet and replace wallet routes placeholders

## Setup
1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Install protoc:

   `apt install protobuf-compiler`

   (May require more a [more recent version](https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os))

## Running

### Server

To run the server:
```bash
cargo run --bin server [-F test_local_market] -- [-p port]
```
- Add "-F test_local_market" to use a local market server.
- You can provide a port using the -p flag. The default value is 3000. See [server/README.md](server/README.md) for the API documentation.

### Peer Node CLI

To run the peer node CLI:
```bash
cargo run --bin peernode
```
- Run the help command within the CLI for possible commands.
```shell
> help
```
- To set up a market connection, set the `boot_nodes` configuration:

```shell
> market set -b /ip4/130.245.173.204/tcp/6881/p2p/QmSzkZ1jRNzM2CmSLZwZgmC9ePa4t2ji3C8WuffcJnb8R
```

In order to provide a market server node, set `public_address`

```shell
> market set -p /ip4/0.0.0.0/tcp/6881
```

Demo:

```shell
> producer add files/giraffe.jpg 1
> producer register
> producer ls
# new instance
> consumer ls 908b7415fea62428bb69eb01d8a3ce64190814cc01f01cae0289939e72909227
# make sure you're on a public ip (or edit producer/register_files)
> consumer get 908b7415fea62428bb69eb01d8a3ce64190814cc01f01cae0289939e72909227 {producer_id}
```

## Running with Docker
We also provide a Docker compose file to easily run the producer and market server together. To run it:
```bash
docker-compose build
docker-compose up
```
This will automatically mount the local `peernode/files` directory to the producer container and expose the producer HTTP and market server gRPC ports.

