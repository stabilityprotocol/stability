# Docker Based Development

1. [Introduction](#introduction)
2. [Running the image](#running-the-image)
   1. [Build](#build)
   2. [Run](#run)
   3. [Example](#example)
3. [Test](#test)
   1. [Build](#build-testing-image)
   2. [Run](#run-testing-image)

## Introduction

Optionally, You can build and run the stability node within Docker directly. it is also possible to run tests in a docker.

## Running the image

### Build

To build the Docker container, run the following command at the root of the project.

```
$ docker build -f ./docker/client/Dockerfile -t stability .
```

### Run

```
docker run -d -p 30333:30333 -p 9933:9933 -p 9615:9615 -e CHAIN="/stability/chain-specs/testnet.json" stability
```

Optional environment variables:

- SEED: This environment variable allows the node to authenticate with a specific account.
- CHAIN and CHAIN_NAME: If you want to use a pre-defined chain spec, you can use the CHAIN_NAME environment variable to specify the name of the chain spec to use. If you want to use a custom chain spec, you can use the CHAIN environment variable to specify the path to the chain spec file to use. If both are specified, CHAIN will be used. By default, the node will use the `dev` chain spec.
- NODE_KEY: This environment variable allows specifying the node key to use. If not specified, the node will generate a random node key. This applies to the P2P key, not the account key.
- BOOTNODES: This environment variable allows specifying the bootnodes to use, separated by commas. If not specified, the node will use the default bootnodes for the chain spec.
- MODE: This environment variable allows the node to run in different pruning modes. Possible values are "full_node" or "archive". The default value is "full_node".
- BACKEND_TYPE: The only available option is `sql` for now. This option allows the node to use a SQL backend instead of the default `key-value` backend for faster Ethereum log queries.
- ZERO_GAS_TX_POOL: This environment variable allows the node to run with a zero gas price transaction pool using the **inline** mode (HTTP fetch during block proposal). By default the feature is disabled. The expected value is a string containing a URL. Check the [Zero Gas Transaction Pool](../docs/ZERO-GAS-TRANSACTIONS.md) document for more information.
- ZERO_GAS_TX_POOL_TIMEOUT: This environment variable allows specifying the timeout for the zero gas transaction HTTP requests in milliseconds. Applies to both inline and enqueue modes. If not specified, the node will use the default timeout (1000ms).
- ZERO_GAS_TX_POOL_ENQUEUE: This environment variable enables the **enqueue** mode, where a background worker polls the external pool and submits zero-gas transactions into the Substrate mempool. This decouples the HTTP fetch from block production, improving block proposal performance. The expected value is a string containing a URL. Check the [Zero Gas Transaction Pool](../docs/ZERO-GAS-TRANSACTIONS.md) document for more information.
- ZERO_GAS_TX_POOL_ENQUEUE_INTERVAL: This environment variable allows specifying the poll interval for the enqueue mode background worker in milliseconds. If not specified, the node will use the default interval (3000ms).

To set an environment variable in the docker run, use the flag -e NAME=VALUE

### Example

```
docker run -d -p 30333:30333 -p 9933:9933  -p 9615:9615 -e SEED=account -e CHAIN="/stability/chain-specs/testnet.json" -e MODE=archive -e BOOTNODES=/ip4/... stability
```

## Test

### Build Testing Image

To build the Docker container for testing, run the following command in the root of the project

```
$ docker build -f ./docker/test/Dockerfile -t stability-test .
```

### Run Testing Image

```
$ docker run stability-test
```
