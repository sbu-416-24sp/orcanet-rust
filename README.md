# Orcanet Rust

## Setup

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Install protoc:

   `apt install protobuf-compiler`

   (May require more a [more recent version](https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os))

## Running

### Server

To run the server:
```bash
cargo run --bin server -- [-p port]
```

You can provide a port using the -p flag. The default value is 3000. See [server/README.md](server/README.md) for the API documentation.

### Peer Node CLI

To run the peer node CLI:
```bash
cargo run --bin peernode
```
Run the help command within the CLI for possible commands.

### CLI Interface

To set up a market connection, set the `boot_nodes` configuration:

```shell
market set -b /ip4/130.245.173.204/tcp/6881/p2p/QmSzkZ1jRNzM2CmSLZwZgmC9ePa4t2ji3C8WuffcJnb8R
```

In order to provide a market server node, set `public_address`

```shell
market set -p /ip4/0.0.0.0/tcp/6881
```

Demo:

```shell
producer add files/giraffe.jpg 1
producer register
producer ls
# new instance
consumer ls 908b7415fea62428bb69eb01d8a3ce64190814cc01f01cae0289939e72909227
# make sure you're on a public ip (or edit producer/register_files)
consumer get 908b7415fea62428bb69eb01d8a3ce64190814cc01f01cae0289939e72909227 {producer_id}
```

To test with HTTP server with local market: (file paths relative to where this is run)

```shell
cargo run --bin peer-node-server -F test_local_market
```

## Running with Docker
We also provide a Docker compose file to easily run the producer and market server together. To run it:
```bash
docker-compose build
docker-compose up
```
This will automatically mount the local `peernode/files` directory to the producer container and expose the producer HTTP and market server gRPC ports.

