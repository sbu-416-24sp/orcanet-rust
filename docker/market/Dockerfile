# Build image
FROM rust:slim-buster as build

# Install the necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    git

# Clone market repository (first step is to prevent Docker from using cache)
# TODO: Use the official branch instead of our fork, once PR is merged
ADD https://api.github.com/repos/dyscott/orcanet-market-rust/git/refs/heads/main version.json
RUN git clone https://github.com/dyscott/orcanet-market-rust.git

# Go to the market directory
WORKDIR /orcanet-market-rust

# Build the project with caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/orcanet-market-rust/target \
	cargo build --release && \
    cp ./target/release/orcanet-market-ferrous ./orcanet-market-ferrous

# Runner image
FROM debian:buster-slim

# Install SSL
RUN apt-get update && apt-get install -y \
    libssl1.1

# Copy the binary from the build image
COPY --from=build /orcanet-market-rust/orcanet-market-ferrous .

# Run the binary
CMD ./orcanet-market-ferrous