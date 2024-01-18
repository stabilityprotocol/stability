## Table of Contents

1. [Introduction](#introduction)
2. [Generating Keys](#generating-keys)
3. [Insert Keys in the Node](#)
4. [Running a Stability Validator Node](#running-a-stability-validator-node)

---

## Introduction

In this README, we'll provide initial instructions for booting a Stability node.

## Generating Keys

A Stability node uses 3 keys to operate:

1. ECDSA key for Aura (Stability's consensus mechanism)
2. ED25519 key for Grandpa (Stability's finality mechanism)

At Stability, we recommend generating a single SEED for all these keys and using it to produce all the necessary keys from it.

To generate a SEED, you can either use a reliable SEED generation service (ensure its safety) or use the Stability node itself, which incorporates a SEED generation mechanism. To achieve this, execute the following command:

```sh
./target/release/stability key generate
```

This command will produce the SEED.

To quickly get started, you can use our dedicated tool at: [Stability Protocol Validator Key Generator](https://stabilityprotocol.github.io/validator-key-generator/). This web-based application simplifies the process of generating SEED mnemonics for validator setup.

## Insert Keys in the Node

To insert the keys using the seed previously generated, use the following script

```sh
   ## TO RUN THIS COMMAND YOU SHOULD DEFINE
   ## $SEED - Seed phrase from which Stability keys are derived
   ## $PATH - Path where you save your node data
   ## $CHAIN_TARGET - Target chain to connect to: dev, testnet, alphanet, betanet
  ./target/release/stability key insert  \
  --base-path $PATH \
  --scheme ecdsa \
  --suri "$SEED" \
  --key-type aura
  --chain-target $CHAIN_TARGET

  ./target/release/stability key insert \
  --base-path $PATH \
  --scheme Ed25519 \
  --suri "$SEED" \
  --key-type gran
  --chain-target $CHAIN_TARGET
```

With this, you will have inserted the keys into the node, ready to run a Stability validator.

## Running a Stability Validator Node

To run a Stability node, execute the following command:

```sh
./target/release/stability --chain testnet
```

Additionally, you can configure the following flags:

- `--validator`: Assumes that you are a validator and begins validation if you are on the validators' list.
- `--pruning archive`: Runs your node in archival mode. An archival node maintains a complete copy and allows queries on any historical chain state.
- `--unsafe-rpc-external --rpc-cors all`: Use if your node will be accessed from a system external to your localhost.
- `--bootnodes`: A list of p2p nodes for Stability to join the network. What is known as bootnodes
- `--base-path`: Specifies a custom base path for the data folder
- `--frontier-backend-type`: The standard database with the system can be modified to use an SQL backend. Since getting Ethereum logs is an everyday use case with a high CPU usage, SQL backend mode is optimized for this. Available options are `sql` or `key-value` (default).

## Generating the a build spec file or Genesis file

This file contains the initial genesis state that all nodes in the network agree on. The genesis state must be established when the blockchain is first started and it cannot be changed thereafter without starting an entirely new blockchain.

To generate the genesis file, execute the following command:

```sh
./target/release/stability build-spec --chain testnet --disable-default-bootnode --raw > specs/testnet.json
```

More info: https://docs.substrate.io/build/chain-spec/
