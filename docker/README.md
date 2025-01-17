# Dockers and Scripts

We provide some scripts and Dockerfiles to simplify developing and testing.

## TLDR

Compile all the different node implementations (*solo*, *relay*, *parachain*):

```bash
> . cfg
> bootstrap.sh
```

Start a _**solo**_ chain with 2 validators (Alice and Bob) and a simple client node with:

```bash
> docker compose -f docker/dockerfiles/zkv-docker-compose.yaml up
```

Start a _**relay**_ chain with a test _**parachain**_ with:
- 2 relay chain validators (Alice and Bob) and a simple relay chain client node
- 2 parachain collators (Alice and Bob) and a simple parachain client node

```bash
> docker compose -f docker/dockerfiles/zkv-relay-docker-compose.yaml up
```

## Scripts

```text
├── cfg                   # Add scripts path to your PATH environment variable
├── docker
│   └── scripts
│       ├── bootstrap.sh  # Compile and generate an image with injected node binary
│       ├── build-chain-image-injected.sh # Create an image with injected solo node binary
│       ├── build-zkv-relay-image-injected.sh # Create an image with injected relay chain node binary
│       ├── build-paratest-image-injected.sh # Create an image with injected parachain node binary
│       ├── build-injected.sh # Create an image with injected binary
│       └── my_cargo      # Use cargo from a docker image with all dependencies installed but
                          # leverage host environment for caching: useful to avoid
                          # problems related to incompatible glibc versions without lost 
                          # cache compilation
...
```

### `bootstrap.sh`

The simple workflow is:

```bash
> . cfg # Just the first time
> bootstrap.sh
```

Once you have the docker images on your local docker repository, you can run:
- a _**solo**_ chain with

```bash
> docker run -ti --rm -p 9944:9944 horizenlabs/zkverify --dev --rpc-cors all --rpc-external
```

- a _**relay**_ chain with

```bash
> docker run -ti --rm -p 9944:9944 horizenlabs/zkv-relay --dev --rpc-cors all --rpc-external
```

- a _**para**_ chain with

```bash
> docker run -ti --rm -p 8844:8844 paratest --dev --rpc-cors all --rpc-external
```

Where:

* `-p 9944:9944`: provide the access to the rpc interface on your host
* `--rpc-cors all --rpc-external`: enable the access from _polkadot.js_ by relaxing the cors policy

The `zkv-node`, `zkv-relay`, and `paratest-node` binaries are also available on your host environment at `target/release/zkv-node`.

### `my_cargo`

`my_cargo` is a simple `cargo` replacement that execute the command inside a `rbuilder` docker container. `rbuilder` has all Rust's dependencies installed and use the host's environment to inherit cargo's repository cache, user github credentials to fetch the private repository and the local target folder to save binaries and incremental compilation artifacts.

### `build-injected.sh`

A script that generate a docker image with the base dependencies and the given executables injected. See `build-chain-image-injected.sh` as example of how to use it.

## Docker and Compose

All Dockerfile and compose definitions are located in `docker/dockerfile` folder.

* `zkv-builder.Dockerfile`: create an image with all dependencies needed to compile the node and is used by `my_cargo` script
* `binary_injected.Dockerfile`: Is mainly used by the scripts and inject one or more binaries in a standard ubuntu 22:04 image
* `zkv-node.Dockerfile`: generate a solo node image with a fresh source compilation (leverage on docker layers to create a small docker image)
* `zkv-docker-compose.yaml`: the cluster definition that runs
2 validator nodes (Alice and Bob) and a simple node that expose its rpc and P2P ports on localhost.
* `zkv-relay.Dockerfile`: generate a relay node image with a fresh source compilation (leverage on docker layers to create a small docker image)
* `zkv-relay-docker-compose.yaml`: the cluster definition that runs 3 relay chain nodes (Alice, Bob, and rpc on port 9944), and 3 parachain nodes (Alice, Bob, and rpc on port 8844)

To generate a solo node image without bothering about local resources, Rust installation and so on you can simply use:

```bash
> docker build -f docker/dockerfiles/zkv-node.Dockerfile -t horizenlabs/zkverify:latest .
```

and run it with

```bash
> docker run -ti --rm horizenlabs/zkverify --dev
```

To generate a relay node image without bothering about local resources, Rust installation and so on you can simply use:

```bash
> docker build -f docker/dockerfiles/zkv-relay.Dockerfile -t horizenlabs/zkv-relay:latest .
```

and run it with

```bash
> docker run -ti --rm horizenlabs/zkv-relay --dev
```

All arguments after `horizenlabs/zkv*` image names will be passed to the node executable.

### Production

If you want to build the docker image compiled for production you can add the following flags:

```sh
--build-arg PROFILE=production # To enable all the optimizations \
--build-arg FEATURES=metadata-hash # Add metadata hash computation to runtime
```

### Compose

Some notes about compose cluster configuration: if you want to run a node with some specific environment variables you can just edit the files in `docker/resources/envs`.
