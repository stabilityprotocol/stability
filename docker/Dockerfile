FROM rustlang/rust:nightly


ARG PROFILE=release
WORKDIR /stability

# Upcd dates core parts
RUN apt-get update -y && \
	apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev protobuf-compiler

# Install rust wasm. Needed for substrate wasm engine
RUN rustup target add wasm32-unknown-unknown

# Copy source code
COPY node ./node
COPY runtime ./runtime
COPY precompiles ./precompiles
COPY Cargo.lock Cargo.toml ./



# Download rust dependencies and build the rust binary
RUN cargo build "--$PROFILE"

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615

# Copy entrypoint

COPY ./docker/entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
