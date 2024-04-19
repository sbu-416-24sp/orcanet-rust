# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] (2024-04-17)
### Added
- A basic project skeleton structure that has no implementation for the protocols
for now, but we list the protocols that we will use and its purpose.
  - Uses Kademlia DHT and the identify protocol for peer discovery
    - Further uses Kademlia DHT for storing info on which peer provides what 
    - Uses the request_response protocol for retrieving the info the peer is providing
  - Uses the ping protocol for checking if a peer is still alive
  - Uses Autonat, DCUtR, and relay protocols for setting up connections between peers behind NATs
- Communication between the peer and the coordinator through a channel.
- Handlers for every protocol that we will end up using to implement the market
- Config for initializing the peer
- A peer struct that provides some basic functionality for the peer
  - Retrieve the peer id 
  - Retrieve the keypair
  - Retrieve all the current addresses the peer is listening on 
  - Retrieve all of the connected peers
  - Status if a peer is currently connected to some other peer given a peer id
