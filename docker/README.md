# Docker Based Development

1. [Introduction](#introduction)
2. [Alphanet](#alphanet)
   1. [Build](#build-alphanet)
   2. [Run](#build-alphanet)
   3. [Example](#example)
3. [Test](#test)
   1. [Build](#build-testing-image)
   2. [Run](#run-testing-image)

## Introduction

Optionally, You can build and run the stability node within Docker directly. it is also possible to run tests in a docker.

## Alphanet

### Build Alphanet

To build the Docker container, run the following command in the root of the project

```
$ docker build -f ./docker/alphanet/Dockerfile -t stability .
```

### Run Alphanet

```
docker run -d -p 30333:30333 -p 9933:9933 -p 9944:9944 -p 9615:9615 stability
```

Optional environment variables:

- SEED: This environment variable allows the node to authenticate with a specific account.
- BOOTNODE: This environment variable allows specifying the bootnode to which the node will connect.

To set an environment variable in the docker run, use the flag -e NAME=VALUE

### Example

```
docker run -d -p 30333:30333 -p 9933:9933 -p 9944:9944 -p 9615:9615 -e SEED=account -e BOOTNODE=/ip4/... stability
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
