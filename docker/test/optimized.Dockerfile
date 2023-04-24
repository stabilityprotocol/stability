FROM ghcr.io/stabilityprotocol/stability-test:latest

WORKDIR /stability

# Copy source code
COPY node ./node
COPY runtime ./runtime
COPY precompiles ./precompiles
COPY pallets ./pallets
COPY primitives ./primitives
COPY test-utils ./test-utils
COPY client ./client
COPY stability-rpc ./stability-rpc
COPY Cargo.lock Cargo.toml rust-toolchain ./

RUN cargo build --release --tests

CMD [ "cargo", "test", "--release", "--verbose" ]