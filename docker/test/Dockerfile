FROM rustlang/rust:nightly AS chef

WORKDIR /stability

# Install system dependencies
RUN apt-get update -y && \
    apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev protobuf-compiler

# Install cargo-chef and cargo-expand using nightly (before toolchain override)
RUN cargo install cargo-chef
RUN cargo install cargo-expand

# Copy rust-toolchain.toml so the pinned toolchain is used for all stages
COPY rust-toolchain.toml ./

# Install the pinned toolchain and wasm target
RUN rustup show && rustup target add wasm32-unknown-unknown && rustup target add wasm32v1-none

# ---------------------
# Stage 2: Plan recipe
# ---------------------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------------------
# Stage 3: Build & Test
# ---------------------
FROM chef AS builder

# Copy recipe and cook dependencies (cached unless Cargo.toml/Cargo.lock change)
COPY --from=planner /stability/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release

# Copy source code and build tests
COPY . .
RUN cargo build --release --tests

CMD [ "cargo", "test", "--release", "--verbose" ]
