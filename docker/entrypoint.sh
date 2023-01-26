#!/bin/bash

if [ -n "$SEED" ]; then
  ./target/release/frontier-template-node key insert --base-path /tmp/node \
  --keystore-path  /tmp/node/chains/alphanet/keystore \
  --scheme ecdsa \
  --suri "$SEED" \
  --key-type aura

  ./target/release/frontier-template-node key insert \
  --keystore-path  /tmp/node/chains/alphanet/keystore \
  --base-path /tmp/node \
  --scheme ecdsa \
  --suri "$SEED" \
  --key-type gran
fi

if [ -n "$BOOTNODE" ]; then
  echo "Starting node with bootnode: $BOOTNODE"
  ./target/release/frontier-template-node --base-path /tmp/node --validator --unsafe-rpc-external --rpc-cors all --unsafe-ws-external --bootnodes "$BOOTNODE" --chain alphanet
else
  echo "Starting node without bootnode"
  ./target/release/frontier-template-node --base-path /tmp/node --validator --unsafe-rpc-external --rpc-cors all --unsafe-ws-external --chain alphanet
fi
