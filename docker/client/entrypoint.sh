#!/bin/bash

CHAIN_TARGET=${CHAIN:-"dev"}
if [ ! -z "$CHAIN_NAME" ]; then
  CHAIN_TARGET="$CHAIN_NAME"
fi

if [ -n "$SEED" ]; then
  ./target/release/stability key insert --base-path /data \
  --keystore-path  /data/chains/stability/keystore \
  --scheme ecdsa \
  --suri "$SEED" \
  --key-type aura

  ./target/release/stability key insert \
  --keystore-path  /data/chains/stability/keystore \
  --base-path /data \
  --scheme Ed25519 \
  --suri "$SEED" \
  --key-type gran

  ./target/release/stability key insert \
  --keystore-path  /data/chains/stability/keystore \
  --base-path /data \
  --scheme ecdsa \
  --suri "$SEED" \
  --key-type imon
fi

START_COMMAND="./target/release/stability --base-path /data --validator --unsafe-rpc-external --rpc-cors all --unsafe-ws-external --pruning archive --prometheus-external --chain=$CHAIN_TARGET"

if [ -n "$NODE_KEY" ]; then
  START_COMMAND="$START_COMMAND --node-key $NODE_KEY"
fi

if [ "$DEFAULT_BOOTNODE" = "yes" ]; then
  START_COMMAND="$START_COMMAND --bootnodes /ip4/3.21.88.38/tcp/30333/p2p/12D3KooWPaen1igo2WYUFCt3EAg4AWjWoMYgmr4tCa2Yb1WfgoDB"
elif [ -n "$BOOTNODES" ]; then
  START_COMMAND="$START_COMMAND --bootnodes $BOOTNODES"
fi

eval $START_COMMAND

