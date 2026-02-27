# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rust Toolchain

The codebase compiles with the **stable** Rust toolchain, including `cargo fmt` (no nightly required, unlike polkadot-sdk).

## Build Commands

Uses `cargo-make` (`cargo make <task>`) or plain `cargo`:

```bash
# Check compilation (fast, skips WASM build)
SKIP_WASM_BUILD=1 cargo check --all-features

# Build release binaries
cargo build --release                                          # everything
cargo build -p zkv-relay --release                            # relay node only
cargo build -p zkv-relay --release --features fast-runtime    # dev relay (shorter epochs)
cargo build -p zkv-relay --release --features runtime-benchmarks

# Tests
SKIP_WASM_BUILD=1 cargo test --release --lib -p zkv-runtime --all-features      # runtime unit tests
cargo test --lib --bins --release --all-features              # all unit tests
cargo test --test '*' --release --all-features                # integration tests

# Single test (example)
SKIP_WASM_BUILD=1 cargo test --release --lib -p pallet-aggregate my_test_name

# Lint/format
SKIP_WASM_BUILD=1 cargo clippy --all-targets -- --deny warnings
cargo fmt
zepter run check-fix    # lint and fix feature propagation in Cargo.toml files
zepter run format-fix   # format feature sections in Cargo.toml files
# or run both with: cargo make zepter

# Full CI pipeline (very slow)
cargo make ci
```

The main binary is `zkv-relay` (not `zkv-node`). Dev node: `./target/release/zkv-relay --dev`.

## Architecture Overview

zkVerify is a Substrate relay chain specialized for ZK proof verification. The core flow: a user submits a proof → the verifier pallet verifies it → a statement hash is emitted → the aggregate pallet collects statements for cross-chain dispatch.

### Verifier System (the core abstraction)

**`primitives/hp-verifiers/src/lib.rs`** — defines the `Verifier` trait that every proof system implements:

```rust
pub trait Verifier {
    type Proof: Arg;
    type Pubs: Arg;   // Public inputs
    type Vk: VkArg;   // Verification key
    fn verify_proof(vk, proof, pubs) -> Result<Option<Weight>, VerifyError>;
    fn validate_vk(vk) -> Result<(), VerifyError>;
    fn vk_hash(vk) -> H256;
    fn hash_context_data() -> &'static [u8];  // e.g. b"groth16"
    fn pubs_bytes(pubs) -> Cow<[u8]>;
    // ...
}
```

**`pallets/verifiers/`** — a generic pallet parametrized by `I: Verifier`. The macro `#[pallet_verifiers::verifier]` in `pallets/verifiers/macros/` generates a concrete pallet for each proof system. Each pallet stores VKs (with reference counting and fund holds via `Tickets`), and exposes:
- `register_vk(vk)` — stores VK, holds deposited funds
- `submit_proof(vk_or_hash, proof, pubs, domain_id)` — verifies and emits `ProofVerified { statement: H256 }`
- `unregister_vk(vk_hash)` — removes VK, releases funds

**Statement hash formula**: `keccak256(keccak256(context) || vk_hash || verifier_version_hash || keccak256(pubs))`

### Verifier Implementations (`verifiers/`)

Each sub-crate (groth16, risc0, sp1, fflonk, plonky2, ultrahonk, ultraplonk, ezkl, tee) implements the `Verifier` trait and is instantiated in the runtime as a named pallet (e.g., `SettlementGroth16Pallet`). Actual crypto is in `native/` (native Rust, bypasses WASM for performance).

### Aggregate Pallet (`pallets/aggregate/`)

Receives `OnProofVerified` callbacks from every verifier pallet and collects statement hashes per domain. When a domain fills up (or `aggregate()` is called), it dispatches the Merkle root aggregation cross-chain via the `DispatchAggregation` trait (`primitives/hp-dispatch/`).

Key concepts:
- **Domain**: a registered destination identified by a `u32` domain_id, with its own aggregation size and delivery fee
- **Aggregation**: a batch of up to N statement hashes whose Merkle root is dispatched
- Domain states: `Active → Held → Removable`

### Runtime (`runtime/zkverify/`)

Pallet indices relevant to custom pallets:
- `Aggregate` = 81, `TokenClaim` = 83, `Crl` = 84
- Verifiers: `Groth16` = 161, `Risc0` = 162, `Ultraplonk` = 163, `Plonky2` = 165, `Fflonk` = 166, `Sp1` = 167, `Ultrahonk` = 168, `Ezkl` = 169, `Tee` = 170

Runtime features: `runtime-benchmarks`, `try-runtime`, `fast-runtime`, `volta`.

### Relay Node (`relay-node/`)

- **`relay-node/cli/`** — CLI entry point (`zkv-cli`)
- **`relay-node/service/`** — `zkv-service` with `PolkadotServiceBuilder` (`builder.rs`): `new()`, `genesis_hash()`, `add_extra_request_response_protocol()`, `build()`
- **`relay-node/benchmarks/`** — benchmark utilities
- **`paratest/`** — test parachain (relay+para integration testing)

### Patches (`patches/`)

Vendored patched versions of upstream polkadot-sdk crates (stable2412). Each patch dir has `PATCH.md`, `patch.patch`, `rustfmt.toml`. Do not edit these without understanding the upstream diff.

## End-to-End Tests (`zombienet-tests/`)

E2E tests use the [Zombienet](https://github.com/paritytech/zombienet) framework to spin up real node instances and exercise on-chain behavior.

### Prerequisites

Both `zkv-relay` and `paratest-node` must be compiled (the test runner checks for both):

```bash
cargo build -p zkv-relay -p paratest-node --release --features fast-runtime
```

Node, npm, and yarn must be installed locally. The `zombienet` binary is downloaded automatically into `zombienet-tests/bin/` on first run.

### Running Tests

```bash
cd zombienet-tests
yarn install       # first time only

yarn test                                         # all tests (release build)
yarn test --debug                                 # use target/debug binaries
yarn test 0007-proof_with_vk.zndsl               # specific test(s)
yarn test 0006-proof_should_pay.zndsl 0012-aggregate_proofs.zndsl  # multiple
```

Direct zombienet invocation (skips the runner script):

```bash
bin/zombienet-linux-x64 -p native test 0003-transaction.zndsl
DEBUG=zombie* bin/zombienet-linux-x64 -p native test 0003-transaction.zndsl  # verbose
```

### Test Structure

Each test consists of:
- **`.zndsl` file** — test definition in Zombienet DSL (natural language assertions)
- **`network_defs/<name>.toml`** — network topology (`single_node.toml`, `two_nodes.toml`, `paratest.toml`)
- **`js_scripts/<name>.js`** (optional) — complex logic invoked via `js-script` in the DSL

### Writing New Tests

- JS/TS test scripts interact with nodes via polkadot.js API and return a single `u64` (0 = pass)
- Proof data fixtures for each verifier (groth16, fflonk, risc0, sp1, etc.) are in `js_scripts/*_data.js`
- Start from an existing JS test (e.g., `0007-proof_with_vk.js`) when adding verifier-related tests

## Code Style

- **Indentation**: 4 spaces (standard `rustfmt` defaults — no custom `rustfmt.toml` at workspace root)
- **Line width**: 100 characters max
- **`unwrap()`**: avoid it; if an invariant makes a panic provably impossible, add a proof comment ending with `; qed`; otherwise prefer `expect("reason")` with a descriptive message over a bare `unwrap()`
- **Arithmetic**: use checked (`checked_add`, `checked_mul`, …) or saturating arithmetic when overflow is plausible; avoid silent wrapping
- **Unsafe code**: requires an explicit safety justification comment
- **`Cargo.toml` feature sections**: managed by `zepter` (formatting and propagation lint); config in `.cargo/zepter.yaml`

## Safety-Critical Code

`native/` contains the actual cryptographic proof verification implementations (one sub-crate per proof system: groth16, fflonk, risc0, sp1, …). This code runs outside WASM for performance and is the security-critical core of the chain. Changes here require extra care: incorrect verification logic could accept invalid proofs silently.

Any change to `native/` — including updates to its dependencies — **must** be followed by a complete chain sync test against both testnet and mainnet before merging.

## Workspace Conventions

- The runtime WASM blob is compiled targeting `wasm32-unknown-unknown`. Some transitive dependencies pull in `std` without actually needing it, which prevents migrating to `wasm32v1-none` (the target used by newer polkadot-sdk versions). Until those transitive dependencies are fixed, `wasm32v1-none` cannot be used.
- Clippy lints: `correctness/suspicious/complexity` = deny, `style` = warn (see root `Cargo.toml` `[lints]`)
- `SKIP_WASM_BUILD=1` skips the WASM blob compilation — safe for `check`, `clippy`, runtime lib tests, and individual pallet tests; **not** safe when building or testing `zkv-relay` (the binary requires the embedded WASM runtime)
- `gum` and `codec` are workspace-level aliases used in patched upstream `.rs` files
- `[workspace.package]` intentionally omits `license` — each crate must declare its own
