
## Table of Contents

1. [Introduction](#introduction)
2. [Generating Keys](#generating-keys)
3. [Insert Keys in the Node](#)
3. [Running a Stability Validator Node](#running-a-stability-validator-node)

---

## Introduction


In this README, we'll provide initial instructions for booting a Stability node.

## Generating Keys

A Stability node uses 3 keys to operate: 
1. ECDSA key for ImOnline Pallet
2. ECDSA key for Aura (Stability's consensus mechanism)
3. ED25519 key for Grandpa (Stability's finality mechanism)

At Stability, we recommend generating a single SEED for all these keys and using it to produce all the necessary keys from it.

To generate a SEED, you can either use a reliable SEED generation service (ensure its safety) or use the Stability node itself, which incorporates a SEED generation mechanism. To achieve this, execute the following command:

```sh
./target/release/stability key generate
```

This command will produce the SEED.


## Insert Keys in the Node


To insert the keys using the seed previously generated, use the following script


```sh
   ## TO RUN THIS COMMAND YOU SHOULD DEFINE 
   ## $SEED - Seed phrase from which Stability keys are derived
   ## $PATH - Path where you save your node data
  ./target/release/stability key insert  \
  --keystore-path  $PATH/chains/alphanet/keystore \
  --base-path $PATH \
  --scheme ecdsa \
  --suri "$SEED" \
  --key-type aura

  ./target/release/stability key insert \
  --keystore-path  $PATH/chains/alphanet/keystore \
  --base-path $PATH \
  --scheme Ed25519 \
  --suri "$SEED" \
  --key-type gran

  ./target/release/stability key insert \
  --keystore-path  $PATH/chains/alphanet/keystore \
  --base-path $PATH \
  --scheme ecdsa \
  --suri "$SEED" \
  --key-type imon
```

With this, you will have inserted the keys into the node, ready to run a Stability validator.

## Running a Stability Validator Node

To run a Stability node, execute the following command:

```sh
./target/release/stability --chain alphanet
```

Additionally, you can configure the following flags:

- `--validator`: Assumes that you are a validator and begins validation if you are on the validators' list.
- `--pruning archive`: Runs your node in archival mode. An archival node maintains a complete copy and allows queries on any historical chain state.
- `--unsafe-rpc-external --rpc-cors all --unsafe-ws-external`: Use if your node will be accessed from a system external to your localhost.
- `--bootnodes`: A list of p2p nodes for Stability to join the network. What is known as bootnodes



