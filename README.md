![stability](docs/media/header.jpeg)

Built using Substrate and Rust, the Stability blockchain serves as a scalable solution for optimized gas market access.

- ⛓️ Read more about [Stability Protocol](https://stabilityprotocol.com)
- 📖 Find more resources in our [Documentation](/docs/README.md)
- 🐦 Follow us in [Twitter](https://twitter.com/stabilityinc) and [Medium](https://medium.com/stabilitynetwork)

## Building and Running

### Compiling the Blockchain

Navigate to the root directory of the project and run the following command to compile the Stability chain:

```bash
cargo build --release
```

### Initiating the Node

To start the Stability chain in development mode, execute this command:

```bash
./target/release/stability --dev
```

### Manual Block Sealing

For manually sealing blocks via RPC, which is particularly useful for TypeScript tests, use the following command:

```bash
./target/release/stability --dev --manual-seal
```

### Docker Deployment

For containerized deployment options, please refer to the Docker [guidelines](docker/README.md) available in the `docker/` directory.

## Architecture

The Stability Substrate chain is built upon Polkadot version `0.9.36` and leverages the `frontier` framework to ensure full Ethereum compatibility. Our architecture incorporates a carefully selected set of pallets, each serving a unique role in the overall functionality of the blockchain.

### Core Pallets

#### Consensus Mechanisms

- **AuRa**: An adaptive and reactive consensus algorithm.
- **GRANDPA**: GHOST-based Recursive Ancestor Deriving Prefix Agreement.

#### Ethereum Virtual Machine (EVM)

- **pallet-evm**: Provides EVM functionalities, enabling the execution of Ethereum-based applications.
- **pallet-ethereum**: Ensures compatibility with the Ethereum API.

### Substrate Native Pallets

- **session**: Manages session keys and validator sets.
- **timestamp**: Responsible for on-chain timekeeping.
- **collective**: Facilitates governance functionalities.

### Third-Party Pallets

- **Moonbeam's precompile-utils**: A utility pallet offering precompiled contract support.
