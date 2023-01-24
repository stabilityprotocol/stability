#!/bin/bash

if [ -n "$SEED" ]; then
  ./target/release/frontier-template-node key insert --base-path /tmp/node \
  --scheme Sr25519 \
  --suri "$SEED" \
  --key-type aura

  ./target/release/frontier-template-node key insert \
  --base-path /tmp/node \
  --scheme Ed25519 \
  --suri "$SEED" \
  --key-type gran
fi

if [ -n "$BOOTNODE" ]; then
  echo "Starting node with bootnode: $BOOTNODE"
  ./target/release/frontier-template-node --base-path /tmp/node --validator --bootnodes "$BOOTNODE" --chain ./genesis.json
else
  echo "Starting node without bootnode"
  ./target/release/frontier-template-node --base-path /tmp/node --validator --chain ./genesis.json
fi