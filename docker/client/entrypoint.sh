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
  START_COMMAND_DEV="./target/release/stability --base-path /tmp/node --dev --unsafe-rpc-external --rpc-cors all --prometheus-external"

  if [ "$MODE" = "archive" ]; then
    START_COMMAND_DEV="$START_COMMAND_DEV --pruning archive"
  fi
  eval $START_COMMAND_DEV
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

START_COMMAND="./target/release/stability --base-path /tmp/node --validator --unsafe-rpc-external --rpc-cors all --rpc-port 9933 --prometheus-external --chain=$CHAIN_TARGET --rpc-max-response-size 100"

if [ "$MODE" = "archive" ]; then
  START_COMMAND="$START_COMMAND --pruning archive"
  
  if [ "$BACKEND_TYPE" = "sql" ]; then
    START_COMMAND="$START_COMMAND --frontier-backend-type sql"
  fi
fi

if [ -n "$NODE_KEY" ]; then
  START_COMMAND="$START_COMMAND --node-key $NODE_KEY"
else
  # Generate a new key if not provided
  ./target/release/stability key generate-node-key --base-path /tmp/node --chain $CHAIN_TARGET
fi

if [ -n "$BOOTNODES" ]; then
  START_COMMAND="$START_COMMAND --bootnodes $BOOTNODES"
fi

if [ -n "$ZERO_GAS_TX_POOL" ]; then
  START_COMMAND="$START_COMMAND --zero-gas-tx-pool $ZERO_GAS_TX_POOL"
fi

if [ -n "$ZERO_GAS_TX_POOL_TIMEOUT" ]; then
  START_COMMAND="$START_COMMAND --zero-gas-tx-pool-timeout $ZERO_GAS_TX_POOL_TIMEOUT"
fi

if [ -n "$CUSTOM_ETH_APIS" ]; then
    START_COMMAND="$START_COMMAND --ethapi=$CUSTOM_ETH_APIS"
else
    START_COMMAND="$START_COMMAND --ethapi=txpool,debug,trace"
fi

echo "Starting $CHAIN_TARGET chain"
eval $START_COMMAND

