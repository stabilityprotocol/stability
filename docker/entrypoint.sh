#!/bin/bash

if [ -n "$SEED" ]; then
  ./target/release/stability key insert --base-path /tmp/node \
  --keystore-path  /tmp/node/chains/alphanet/keystore \
  --scheme Sr25519 \
  --suri "$SEED" \
  --key-type aura

  ./target/release/stability key insert \
  --keystore-path  /tmp/node/chains/alphanet/keystore \
  --base-path /tmp/node \
  --scheme Ed25519 \
  --suri "$SEED" \
  --key-type gran
fi

START_COMMAND="./target/release/stability --base-path /tmp/node --validator --unsafe-rpc-external --rpc-cors all --unsafe-ws-external --chain alphanet --pruning archive"

if [ -n "$BOOTNODE" ]; then
  START_COMMAND="$START_COMMAND --bootnodes $BOOTNODE"
fi

eval $START_COMMAND

