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

The Stability Substrate chain is built upon Polkadot version `stable-2407` and leverages the `frontier` framework to ensure full Ethereum compatibility. Our architecture incorporates a carefully selected set of pallets, each serving a unique role in the overall functionality of the blockchain.

### Core Pallets

#### Consensus Mechanisms

- **AuRa**: An adaptive and reactive consensus algorithm for block production.
- **GRANDPA**: GHOST-based Recursive Ancestor Deriving Prefix Agreement for finality.
- **Validator Set**: Manages the dynamic validator set with automatic rotation and offline detection.

### Token and Fee Management

- **pallet-custom-balances**: Manages native token balances with EVM compatibility.
- **pallet-supported-tokens-manager**: Handles supported token configurations and validations.
- **pallet-user-fee-selector**: Allows users to select preferred tokens for fee payment.
- **pallet-validator-fee-selector**: Manages validator fee token preferences.
- **pallet-dnt-fee-controller**: Controls decentralized native token fee mechanisms.
- **pallet-fee-rewards-vault**: Manages fee rewards distribution and storage.

### Transaction Management

- **pallet-sponsored-transactions**: Enables transaction sponsorship functionality.
- **pallet-zero-gas-transactions**: Handles zero-gas transaction processing.
- **pallet-transaction-payment**: Manages transaction fee payments and calculations.

### Governance and Control

- **pallet-collective**: Facilitates governance functionalities through collective decision-making.
- **pallet-root-controller**: Manages root-level administrative controls.
- **pallet-upgrade-runtime-proposal**: Handles runtime upgrade proposals and execution.
- **pallet-validator-keys-controller**: Controls validator key management and rotation.

### Third-Party Pallets

- **Moonbeam's precompile-utils**: A utility pallet offering precompiled contract support.
- **Moonbeam's RPC debug**: Provides enhanced debugging capabilities and transaction pool management.
- **Frontier**: Provides Ethereum compatibility layer and RPC support.
