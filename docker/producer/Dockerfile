# Build image
FROM rust:slim-buster as build

# Install the necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    protobuf-compiler

# Create empty project
WORKDIR /orcanet-rust

# Copy in files
COPY Cargo.toml .
COPY Cargo.lock .
COPY build.rs .
COPY src src
COPY market market

# Build the project with caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/orcanet-rust/target \
	cargo build --release && \
    cp ./target/release/peer-node ./peer-node

# Runner image
FROM debian:buster-slim

# Install SSL
RUN apt-get update && apt-get install -y \
    libssl1.1

# Copy the binary from the build image
COPY --from=build /orcanet-rust/peer-node .

# Environment variables
ENV ORCA_MARKET_ADDRESS localhost:50051
ENV ORCA_EXTERNAL_IP 127.0.0.1
ENV ORCA_HTTP_PORT 8080

# Run the binary
CMD if [ -z "$ORCA_EXTERNAL_IP" ]; then ./peer-node -p -m $ORCA_MARKET_ADDRESS --port $ORCA_HTTP_PORT; \
    else ./peer-node -p -m $ORCA_MARKET_ADDRESS --port $ORCA_HTTP_PORT --ip $ORCA_EXTERNAL_IP ; fi
