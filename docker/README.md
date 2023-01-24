## Docker Based Development

Optionally, You can build and run the frontier node within Docker directly.

### Build

To build the Docker container, run the following command in the root of the project

```
$ docker build -f ./docker/Dockerfile -t stability .
```

### Run

```
docker run -d -p 30333:30333 -p 9933:9933 -p 9944:9944 -p 9615:9615 stability
```

Optional environment variables:

- SEED: This environment variable allows the node to authenticate with a specific account.
- BOOTNODE: This environment variable allows specifying the bootnode to which the node will connect.

To set an environment variable in the docker run, use the flag -e NAME=VALUE

#### Example

```
docker run -d -p 30333:30333 -p 9933:9933 -p 9944:9944 -p 9615:9615 -e SEED=account -e BOOTNODE=/ip4/... stability
```
