# Running an Archive Node Using Docker

This guide provides instructions for running an archive node on the Stability Protocol network using Docker. An archive node stores the entire blockchain history, including all past states, making it ideal for querying historical data or running analytics.

## Prerequisites

- Docker installed on your system. Download it from [docker.com](https://www.docker.com/).
- Hardware requirements: At least 8 CPU cores, 16 GB RAM, and 1 TB SSD storage (depending on the network size).
- The genesis file (e.g., `testnet.json`) provided separately by the Stability Protocol team. Place this file in a known location on your host machine, such as `~/stability/genesis/testnet.json`.

## Step 1: Pull the Docker Image

Pull the latest Stability node image from the official repository:

```
docker pull ghcr.io/stabilityprotocol/stability:latest
```

If the image is not publicly available, refer to the [Stability repository](https://github.com/stabilityprotocol/stability) for instructions on building from source.

## Step 2: Prepare the Genesis File

The archive node requires a genesis file to initialize the chain. Mount this file as a volume in the Docker container.

Assume your genesis file is located at `~/stability/genesis/testnet.json` on the host. You will mount it to `/genesis/testnet.json` inside the container.

## Step 3: Run the Docker Container

Run the archive node using the following Docker command. This command:

- Mounts a volume for persistent data storage.
- Mounts the provided genesis file.
- Sets the `CHAIN_NAME` environment variable to point to the genesis file.
- Exposes ports for P2P, RPC, and WebSocket connections.
- Configures the node to run in archive mode.

```
docker run -d --name stability-archive-node \
  -v ~/stability/data:/data \
  -v ~/stability/genesis/testnet.json:/genesis/testnet.json \
  -e CHAIN_NAME=/genesis/testnet.json \
  -e MODE="archive" \
  -e BACKEND_TYPE="sql" \
  -p 30333:30333 \
  -p 9933:9933 \
  ghcr.io/stabilityprotocol/stability:latest
```

### Command Explanation

- `-d`: Runs the container in detached mode.
- `--name stability-archive-node`: Names the container for easy management.
- `-v ~/stability/data:/data`: Mounts a host directory for storing blockchain data (create `~/stability/data` if it doesn't exist).
- `-v ~/stability/genesis/testnet.json:/genesis/testnet.json`: Mounts the genesis file into the container.
- `-e CHAIN_NAME=/genesis/testnet.json`: Specifies the chain specification file path.
- `-p 30333:30333`: Exposes the P2P port.
- `-p 9933:9933`: Exposes the HTTP/WS RPC port.
- `stabilityprotocol/stability:latest`: The Docker image to use.

## Step 4: Verify the Node is Running

Check the container status:

```
docker ps
```

Monitor the logs to confirm the node is syncing and operating in archive mode:

```
docker logs -f stability-archive-node
```

Look for messages indicating block synchronization (e.g., "Imported #...") and confirm the node is storing full state data.

## Troubleshooting

- **Node not starting**: Verify the genesis file is correctly mounted and the `CHAIN_NAME` path is accurate.
- **Port conflicts**: Ensure ports 30333, 9933 are not in use on the host.
- **Storage issues**: Confirm the mounted data directory has sufficient space and permissions (archive nodes require significant storage).
- **Syncing slow**: Archive nodes take longer to sync due to storing full state data. Monitor logs for progress.

For additional configurations or advanced setups, consult the Stability Protocol documentation or Substrate guides.

For support, visit the [Stability GitHub repository](https://github.com/stabilityprotocol/stability) or contact the Stability Protocol team.
