![stability](media/header.png)

Implementation of Stability blockchain in Substrate + Rust, a scalability solution for accessing the gas market.

- ⛓️ Read more about [Stability Protocol](https://stabilityprotocol.com)
- 📖 Find more resources in our [Documentation](https://stability-protocol.readme.io/)
- 🐦 Follow us in [Twitter](https://twitter.com/stabilityinc)

## Build & Run

To build the chain, execute the following commands from the project root:

```
$ cargo build --release
```

To execute the chain, run:

```
$ ./target/release/frontier-template-node --dev
```

The node also supports to use manual seal (to produce block manually through RPC).
This is also used by the ts-tests:

```
$ ./target/release/frontier-template-node --dev --manual-seal
```

For using a Dockerized solution you can follow the [instructions](docker/README.md) under the `docker/` folder.

## Architecture

The Stability Substrate chain is based on `polkadot-v0.9.30` using the `frontier-template-node`.
For building this chain, the next pallets have been imported:

- Consensus: _AuRa, GRANDPA_
- EVM: _pallet-evm\*, pallet-ethereum, pallet-dynamic-fee_
- Substrate: _Balances, sudo, session, timestamp_
