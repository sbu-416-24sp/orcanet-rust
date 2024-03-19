# gRPC Consumer / Producer
## Bootstrap by team: Portuguese Man O' War
Alexander Snit, Daniel Kogan, Dylan Scott, Gretta Halollari

### Running the code

To run the Rust market server, clone the [Rust market server](https://github.com/sbu-416-24sp/orcanet-market-rust) and follow the instructions in the README.

The producer will automatically register any files in the `files` directory. To add files, simply add them to the `files` directory (or any subdirectory) and restart the producer.
```bash
cargo run -- -p  
```

To run the consumer:
```bash
cargo run -- -f <file_hash>
```

### Features / To-Do
- [x] gRPC Client for Market Server
- [x] HTTP Server and Client for File Transfer
- [x] Authorization tokens
- [x] Basic CLI for testing
- [x] Automatic file hashing and registration for Producer
- [x] File chunking and reassembly
- [ ] Extend CLI to allow producer and consumer to be utilized simultaneously
- [ ] Automatic IP address discovery
- [ ] Producer selection algorithm
- [ ] Payment verification
- [ ] File upload
- [ ] File verification on consumer
