FROM rustlang/rust:nightly AS chef

LABEL org.opencontainers.image.source https://github.com/stabilityprotocol/stability
WORKDIR /stability

# Install system dependencies
RUN apt-get update -y && \
	apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev protobuf-compiler

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef

# Copy rust-toolchain.toml so the pinned toolchain is used for all stages
COPY rust-toolchain.toml ./

# Install the pinned toolchain and wasm target
RUN rustup show && rustup target add wasm32-unknown-unknown

# ---------------------
# Stage 2: Plan recipe
# ---------------------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------------------
# Stage 3: Build
# ---------------------
FROM chef AS builder
ARG PROFILE=release

# Copy recipe and cook dependencies (cached unless Cargo.toml/Cargo.lock change)
COPY --from=planner /stability/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json "--$PROFILE"

# Copy source code and build
COPY . .
RUN cargo build "--$PROFILE"

# 30333 for p2p traffic
# 9933 for RPC call
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9615

# Copy entrypoint
COPY ./docker/client/entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
