FROM rustlang/rust:nightly

WORKDIR /stability

# Upcd dates core parts
RUN apt-get update -y && \
    apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev protobuf-compiler

# Install rust wasm. Needed for substrate wasm engine
RUN rustup target add wasm32-unknown-unknown

# Install cargo expand. Needed for testing
RUN cargo install cargo-expand

# Copy source code
COPY node ./node
COPY runtime ./runtime
COPY precompiles ./precompiles
COPY pallets ./pallets
COPY primitives ./primitives
COPY client ./client
COPY stability-rpc ./stability-rpc
COPY vendor ./vendor
COPY Cargo.lock Cargo.toml rust-toolchain.toml ./

RUN cargo build --release --tests

CMD [ "cargo", "test", "--release", "--verbose" ]
