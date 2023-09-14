#!/bin/bash

if [ -n "$ENV_PATH" ] && [ -f "$ENV_PATH" ]; then
    source "$ENV_PATH"
fi



CHAIN_TARGET=${CHAIN:-"dev"}
if [ ! -z "$CHAIN_NAME" ]; then
  CHAIN_TARGET="$CHAIN_NAME"
fi

if [[ "$CHAIN_TARGET" == "dev" ]]; then
  echo "Starting dev chain"
  ./target/release/stability --base-path /tmp/node --dev --unsafe-rpc-external --rpc-cors all --unsafe-ws-external --pruning archive --prometheus-external
  exit 0
fi

if [ -n "$SEED" ]; then
  ./target/release/stability key insert \
  --base-path /tmp/node \
  --chain $CHAIN_TARGET \
  --scheme ecdsa \
  --suri "$SEED" \
  --key-type aura

  ./target/release/stability key insert \
  --base-path /tmp/node \
  --chain $CHAIN_TARGET \
  --scheme Ed25519 \
  --suri "$SEED" \
  --key-type gran
fi

START_COMMAND="./target/release/stability --base-path /tmp/node --validator --unsafe-rpc-external --rpc-cors all --unsafe-ws-external --pruning archive --prometheus-external --chain=$CHAIN_TARGET"

if [ -n "$NODE_KEY" ]; then
  START_COMMAND="$START_COMMAND --node-key $NODE_KEY"
fi

if [ "$DEFAULT_BOOTNODE" = "yes" ]; then
  START_COMMAND="$START_COMMAND --bootnodes /ip4/3.21.88.38/tcp/30333/p2p/12D3KooWPaen1igo2WYUFCt3EAg4AWjWoMYgmr4tCa2Yb1WfgoDB"
elif [ -n "$BOOTNODES" ]; then
  START_COMMAND="$START_COMMAND --bootnodes $BOOTNODES"
fi

echo "Starting $CHAIN_TARGET chain"
eval $START_COMMAND

