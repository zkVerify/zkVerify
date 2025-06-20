# zkVerify

Implementation of a node for the **zkVerify Proof Verification Layer**.

It is based on the [Substrate](https://substrate.io/) framework.

> [!IMPORTANT]
> ***zkVerify*** is currently in an early **testnet** stage.
> The plan for going live on Mainnet will be communicated later.
> For more information see [zkVerify.io](https://zkverify.io/).

## What is zkVerify
zkVerify is a **L1** blockchain designed to provide verification for zero knowledge proofs.
As a high performance, public, decentralized, reliable, and secure blockchain, zkVerify has dedicated proof verification methods written in Rust, and available to be used in a modular and composable way.

## Motivation
As execution and data availability have been offloaded from monolithic blockchain stacks, we believe that proof verification and settlement need specialization. This system provides such, enabling the next wave of innovation for ZK cryptography within Web3 and beyond.

### Prohibitive Costs
The proof verification market is estimated to incur $100+ million in security expenses alone for zkRollups in 2024, extending to $1.5 billion by 2028 when including ZK applications.
The verification of a single ZK proof on Ethereum can consume upwards of 200,000 to 300,000 gas units, depending on the proof type. Beyond nominal fees today, the variability of future fees inhibits product adoption.

### Hampering Innovation
Ethereum Improvement Proposals (EIPs) are design documents that outline new features, standards, or processes for the Ethereum blockchain, serving as a primary mechanism for proposing changes to the network.
However, the bottlenecks in approving and implementing EIPs hinder the adoption of cutting-edge proving systems, limiting the potential advancements in scalability and privacy. 

For instance, the choice to standardize around the BN254 curve, while practical at the time of implementation, means that operations involving other elliptic curves are not directly supported and are prohibitively expensive to execute.

## Building and running

### Prerequisites

Install Rust toolchain. Instructions available [here](https://www.rust-lang.org/tools/install)

For Mac users, after installation run:

```bash
brew install protobuf
rustup target add wasm32-unknown-unknown
```

### Build from source

To build the client from source, clone this repository and run the following commands from the root of the project:

```bash
git checkout <latest tagged release>
cargo build -p zkv-relay --release
```

In order to build `paratest` node as well (for the E2E test involving a Parachain), you can simply run:

```bash
cargo build --release
```

### Run

It is possible to run tests with:

```bash
cargo test
```

### Run testnet node

To run a testnet node:

```bash
cd target/release
./zkv-relay --chain test
```

The client will connect to `ZKV Testnet` and start syncing blockchain data, with default path at `$HOME/.local/share/` (double check with log `💾 Database: RocksDb at`).

For entirely removing blockchain data:

```bash
cd target/release
./zkv-relay purge --chain test
```

### Run dev node

To run a local dev node:

```bash
cd target/release
./zkv-relay --dev
```

The client will run a chain with a single validator (Alice) and start producing blocks.

```
2024-03-28 11:49:08 zkVerify Mainchain Node
2024-03-28 11:49:08 ✌️  version 0.1.0-deda6a0980c
2024-03-28 11:49:08 ❤️  by Horizen Labs <admin@horizenlabs.io>, 2024-2024
2024-03-28 11:49:08 📋 Chain specification: Development
2024-03-28 11:49:08 🏷  Node name: Alice
2024-03-28 11:49:08 👤 Role: AUTHORITY
2024-03-28 11:49:08 💾 Database: RocksDb at /tmp/substrateVqTiy0/chains/dev/db/full
2024-03-28 11:49:08 [0] 💸 generated 1 npos voters, 1 from validators and 0 nominators
2024-03-28 11:49:08 [0] 💸 generated 1 npos targets
2024-03-28 11:49:08 🔨 Initializing Genesis block/state (state: 0x271d…3d28, header-hash: 0x1b7e…5b3e)
2024-03-28 11:49:08 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.
2024-03-28 11:49:09 Using default protocol ID "sup" because none is configured in the chain specs
2024-03-28 11:49:09 🏷  Local node identity is: 12D3KooWRRRVCzJNGdhAfMW4fzpA3HwQb498uecyp8NsAaLxCuhq
2024-03-28 11:49:09 💻 Operating system: linux
2024-03-28 11:49:09 💻 CPU architecture: x86_64
2024-03-28 11:49:09 💻 Target environment: gnu
2024-03-28 11:49:09 💻 CPU: 11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz
2024-03-28 11:49:09 💻 CPU cores: 8
2024-03-28 11:49:09 💻 Memory: 15856MB
2024-03-28 11:49:09 💻 Kernel: 5.15.146.1-microsoft-standard-WSL2
2024-03-28 11:49:09 💻 Linux distribution: Ubuntu 20.04.6 LTS
2024-03-28 11:49:09 💻 Virtual machine: yes
2024-03-28 11:49:09 📦 Highest known block at #0
2024-03-28 11:49:09 〽️ Prometheus exporter started at 127.0.0.1:9615
2024-03-28 11:49:09 Running JSON-RPC server: addr=127.0.0.1:9944, allowed origins=["*"]
2024-03-28 11:49:12 🙌 Starting consensus session on top of parent 0x1b7ebdeb01f5506a6bcbe83277477696194baf2be903617258113bdc9b385b3e
2024-03-28 11:49:12 🎁 Prepared block for proposing at 1 (0 ms) [hash: 0x831921d2e2c853bd5feedbef5885a7de7c0622668fb49a2a6aba9c2611afcbe6; parent_hash: 0x1b7e…5b3e; extrinsics (1): [0xb95e…ab06]
2024-03-28 11:49:12 🔖 Pre-sealed block for proposal at 1. Hash now 0x89e3b0332a1fc1ba984aba6699872301e1f22e0efa7e80eeb76c3af9b711c6c4, previously 0x831921d2e2c853bd5feedbef5885a7de7c0622668fb49a2a6aba9c2611afcbe6.
2024-03-28 11:49:12 ✨ Imported #1 (0x89e3…c6c4)
2024-03-28 11:49:14 💤 Idle (0 peers), best: #1 (0x89e3…c6c4), finalized #1 (0x89e3…c6c4), ⬇ 0 ⬆ 0
2024-03-28 11:49:18 🙌 Starting consensus session on top of parent 0x89e3b0332a1fc1ba984aba6699872301e1f22e0efa7e80eeb76c3af9b711c6c4
2024-03-28 11:49:18 🎁 Prepared block for proposing at 2 (0 ms) [hash: 0xea3c4edc3223623ccbbfa6871e05e3a1b8b6b8a9ed0b97de37fde441d9860c78; parent_hash: 0x89e3…c6c4; extrinsics (1): [0x2e13…7d95]
2024-03-28 11:49:18 🔖 Pre-sealed block for proposal at 2. Hash now 0x8226727507239e061f089d102f346e0e6c285a7d73a1dce3e000196f1dbedf51, previously 0xea3c4edc3223623ccbbfa6871e05e3a1b8b6b8a9ed0b97de37fde441d9860c78.
2024-03-28 11:49:18 ✨ Imported #2 (0x8226…df51)
2024-03-28 11:49:19 💤 Idle (0 peers), best: #2 (0x8226…df51), finalized #1 (0x89e3…c6c4), ⬇ 0 ⬆ 0
```

## Hardware Requirements for Validators
On-chain weights have been computed on an [Amazon AWS EC2 C7a.2xlarge](https://aws.amazon.com/it/ec2/instance-types/c7a/) instance with a storage of type IO2 8000 IOPS.
We recommend adopting at least an equivalent machine, with at least 50GB of storage space, if you plan to run a validator node.

## Docker

zkVerify includes some Docker files for building the client and running one or more nodes locally.
For more information, see [docker/README.md](docker/README.md).

We also provide Docker images to run a validator, boot or RPC node directly in the public testnet, via a user-friendly installation and deployment process. Please take a look at [Run a New RPC Node](https://docs.zkverify.io/node-operators/run_using_docker/run_new_rpc_node) section of the official documentation. 

## Documentation

The official documentation is available at [docs.zkverify.io](https://docs.zkverify.io/).

## License

zkVerify is released under the following licenses:
- [GPL-3](LICENSE-GPL3) for all the crates under the `zkv-relay`, `zkv-runtime` and `pallet-fflonk-verifier`packages.
- GPL-3 with CLASSPATH exception for `cumulus-relay-chain-inprocess-interface` and `cumulus-relay-chain-minimal-node` crates.
- [Apache 2.0](LICENSE-APACHE2) for all the other crates.
