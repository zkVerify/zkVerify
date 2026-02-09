# zkv-service vs upstream polkadot-service (polkadot-stable2512)

This document summarizes the differences between `zkv-service` (at `relay-node/service/`) and
the upstream `polkadot-service` crate from the `polkadot-stable2512` branch of `polkadot-sdk`.

It is intended as a reference for future upgrades and to identify differences that could
potentially be removed to reduce maintenance burden.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Cargo.toml](#cargotoml)
- [lib.rs](#librs)
- [builder.rs](#builderrs)
- [overseer.rs](#overseerrs)
- [benchmarking.rs](#benchmarkingrs)
- [chain_spec.rs](#chain_specrs)
- [grandpa_support.rs](#grandpa_supportrs)
- [relay_chain_selection.rs](#relay_chain_selectionrs)
- [workers.rs](#workersrs)
- [parachains_db/](#parachains_db)
- [rpc.rs (zkv-only)](#rpcrs-zkv-only)
- [fake_runtime_api.rs (upstream-only)](#fake_runtime_apirs-upstream-only)
- [tests.rs](#testsrs)
- [tests/helpers.rs (zkv-only)](#testshelpers-zkv-only)
- [Summary of Removable Differences](#summary-of-removable-differences)

---

## Architecture Overview

The fundamental architectural difference is **single-runtime vs multi-runtime**:

| Aspect | Upstream | zkv-service |
|--------|----------|-------------|
| Runtimes | Polkadot, Kusama, Westend, Rococo | Single zkv-runtime |
| Chain variants | 4 + Unknown | 2 (Volta, ZkVerify) + Unknown |
| Runtime selection | Feature-gated (`westend-native`, `rococo-native`) | Always `zkv-runtime` |
| Fake runtime API | Yes (`fake_runtime_api.rs`) | Not needed |
| BEEFY consensus | Full support | Intentionally absent |
| MMR gadget | Conditional | Absent |
| Litep2p network backend | Supported | Not used (Libp2p only) |
| Custom host functions | Standard | `native::HLNativeHostFunctions` for proof verification |
| Custom RPC | Via `polkadot-rpc` crate | Inline `rpc.rs` module |

---

## Cargo.toml

### Dependencies only in upstream (not in zkv-service)

| Dependency | Purpose | Reason for removal |
|------------|---------|-------------------|
| `sc-consensus-beefy` | BEEFY consensus | BEEFY not supported |
| `sp-consensus-beefy` | BEEFY primitives | BEEFY not supported |
| `sp-mmr-primitives` | MMR primitives | MMR gadget not used |
| `mmr-gadget` | MMR subsystem | MMR gadget not used |
| `sp-genesis-builder` | Genesis state builder | Not needed for single runtime |
| `polkadot-rpc` | Upstream RPC definitions | Replaced by custom `rpc.rs` |
| `rococo-runtime` | Rococo runtime | Single-chain |
| `rococo-runtime-constants` | Rococo constants | Single-chain |
| `westend-runtime` | Westend runtime | Single-chain |
| `westend-runtime-constants` | Westend constants | Single-chain |
| `xcm` | Cross-chain messaging | Not applicable |
| `xcm-runtime-apis` | XCM runtime APIs | Not applicable |

### Dependencies only in zkv-service (not in upstream)

| Dependency | Purpose |
|------------|---------|
| `zkv-runtime` | zkVerify runtime |
| `zkv-benchmarks` | Custom benchmarks |
| `native` | Custom host functions for proof verification |
| `aggregate-rpc` | Aggregate proof RPC endpoints |
| `vk-hash` | Verification key hash RPC |
| `schnellru` | LRU cache |
| `jsonrpsee` | JSON-RPC server (upstream gets this transitively via `polkadot-rpc`) |
| `substrate-frame-rpc-system` | System RPC |
| `pallet-transaction-payment-rpc` | Payment RPC |
| `sc-consensus-babe-rpc` | BABE RPC |
| `sc-consensus-grandpa-rpc` | GRANDPA RPC |
| `substrate-state-trie-migration-rpc` | Trie migration RPC |
| `sc-rpc`, `sc-rpc-spec-v2` | Core RPC traits |

### Feature flags

| Feature | Upstream | zkv-service |
|---------|----------|-------------|
| `westend-native` | Enables Westend runtime | Not present |
| `rococo-native` | Enables Rococo runtime | Not present |
| `metadata-hash` | `rococo-runtime?/metadata-hash`, `westend-runtime?/metadata-hash` | `zkv-runtime/metadata-hash` |
| `runtime-benchmarks` | Includes `polkadot-test-client`, `xcm`, `xcm-runtime-apis` | Includes `zkv-runtime`, `test-client`, `pallet-babe`, `pallet-staking` |
| `try-runtime` | Includes `rococo-runtime?`, `westend-runtime?` | Includes `zkv-runtime`, `pallet-babe`, `pallet-staking` |
| `fast-runtime` | `rococo-runtime?`, `westend-runtime?` | `zkv-runtime/fast-runtime` |

### Dependency version pinning

zkv-service pins some external crates explicitly instead of using workspace versions:
`is_executable = "1.0.1"`, `schnellru = "0.2.1"`, `thiserror = "1.0.48"`,
`kvdb = "0.13.0"`, `kvdb-rocksdb = "0.21.0"`, `parity-db = "0.4.12"`.

Also `serde_json` has `arbitrary_precision` feature enabled (upstream does not).

---

## lib.rs

### Module declarations

| Module | Upstream | zkv-service | Notes |
|--------|----------|-------------|-------|
| `fake_runtime_api` | Yes | No | Not needed for single runtime |
| `rpc` | No | Yes | Custom RPC module |

### Re-exports and type aliases

- **Block/RuntimeApi**: Upstream re-exports from `fake_runtime_api`; zkv-service imports directly
  from `zkv_runtime::opaque::Block` and `zkv_runtime::RuntimeApi`.
- **FullWasmExecutor**: zkv-service adds `native::HLNativeHostFunctions` to the host functions
  tuple for proof verification support.
- **Convenience aliases**: zkv-service adds `pub use sc_service as service` and
  `pub use sc_consensus_babe as babe`.

### Chain enum and IdentifyVariant trait

**Upstream:**
```rust
enum Chain { Polkadot, Kusama, Rococo, Westend, Unknown }
trait IdentifyVariant {
    fn is_polkadot(&self) -> bool;
    fn is_kusama(&self) -> bool;
    fn is_westend(&self) -> bool;
    fn is_rococo(&self) -> bool;
    fn is_versi(&self) -> bool;
    fn is_dev(&self) -> bool;
    fn identify_chain(&self) -> Chain;
}
// Impl for Box<dyn ChainSpec> only
```

**zkv-service:**
```rust
enum Chain { Volta, ZkVerify, Unknown }
impl Chain {
    fn ss58_format(&self) -> Ss58AddressFormat { ... }
}
trait IdentifyVariant {
    fn is_volta(&self) -> bool;
    fn is_zkverify(&self) -> bool;
    fn is_dev(&self) -> bool;
    fn identify_chain(&self) -> Chain;
}
// Impl for both &dyn ChainSpec AND Box<dyn ChainSpec>
```

### Error enum

zkv-service removes the `NoRuntime` variant (not needed for single-runtime).

### NewFullParams

zkv-service omits the `enable_beefy` field. All other fields match upstream.

### Key functions

| Function | Upstream | zkv-service | Difference |
|----------|----------|-------------|------------|
| `new_partial_basics` | In `builder/partial.rs` | In `lib.rs` | Code organization |
| `new_partial` | In `builder/partial.rs` | In `lib.rs` | Code organization; no BEEFY block import |
| `new_chain_ops` | Multi-chain dispatch via macro | Single call, no dispatch | Simplification |
| `build_full` | Litep2p/Libp2p dispatch + Polkadot channel warning | Libp2p only, no warning | Simplification |
| `availability_config` | Inline in builder | Extracted as helper function | Code organization |

---

## builder.rs

### File structure

- **Upstream**: `builder/mod.rs` + `builder/partial.rs` (two files)
- **zkv-service**: Single `builder.rs` file

### PolkadotServiceBuilder struct

| Field | Upstream | zkv-service | Notes |
|-------|----------|-------------|-------|
| `config` | Yes | Yes | Same |
| `params` | Yes | Yes | Same |
| `overseer_connector` | Yes | Yes | Same |
| `partial_components` | Yes (stores full PartialComponents) | No | - |
| `basics` | No | Yes (stores Basics struct) | Defers `new_partial` to `build()` |
| `select_chain` | No | Yes (stored separately) | Extracted from partial |
| `net_config` | Yes | Yes | Same |

The zkv-service builder defers `new_partial()` to the `build()` method instead of calling it
in `new()`. This avoids needing to name the `impl Fn(...)` return type.

### BEEFY (completely absent in zkv-service)

Upstream builder includes:
- BEEFY block import layer (`FullBeefyBlockImport` type)
- `beefy_block_import_and_links()` in `new_partial()`
- `beefy_voter_links` as 4th element of `import_setup` tuple
- BEEFY gossip protocol and notification service setup
- BEEFY on-demand justifications handler
- BEEFY gadget spawning (`sc_consensus_beefy::start_beefy_gadget()`)
- BEEFY RPC deps

None of this exists in zkv-service.

### MMR Gadget (absent in zkv-service)

Upstream conditionally spawns MMR gadget when offchain indexing is enabled:
```rust
if is_offchain_indexing_enabled {
    task_manager.spawn_essential_handle().spawn_blocking(
        "mmr-gadget", None, MmrGadget::start(...)
    );
}
```

Not present in zkv-service.

### Behavioral differences

| Aspect | Upstream | zkv-service |
|--------|----------|-------------|
| Backoff authoring | Chain-specific: disabled on Polkadot/Kusama, `max_interval=10` on Rococo/Versi/dev | Parameter-based: `force_authoring_backoff.then(default)` |
| Fetch chunks threshold | `None` on Polkadot (conservative) | `None` on Volta (testnet), threshold on mainnet |
| GRANDPA hard forks | Kusama-specific hard forks | Always empty |
| Genesis hash | `client.chain_info().genesis_hash` | `client.block_hash(0)` with explicit unwrap |
| Hardware benchmark | Detailed CPU core metric checks with differentiated warnings | Simplified: single warning for authority failures |

---

## overseer.rs

Nearly identical between upstream and zkv-service.

### Single critical difference

In `validator_overseer_builder`:
- **Upstream**: `.collation_generation(DummySubsystem)`
- **zkv-service**: `.collation_generation(CollationGenerationSubsystem::new(...))`

This enables validators in zkv-service to participate in collation generation.

### Cosmetic differences

- Return type formatting (single-line vs multi-line)
- `HashSet` imported directly in upstream vs `std::collections::HashSet` qualified in zkv-service
- `Duration` imported separately in zkv-service

---

## benchmarking.rs

| Aspect | Upstream | zkv-service |
|--------|----------|-------------|
| Chain support | Multi-chain via `identify_chain!` macro | Single-chain, no branching |
| ExtrinsicBuilder | Requires `chain: Chain` parameter | No chain parameter |
| EXISTENTIAL_DEPOSIT | `runtime::ExistentialDeposit::get()` | `zkv_runtime::currency::EXISTENTIAL_DEPOSIT` |
| TxExtension | 11-tuple (includes `AuthorizeCall`, `WeightReclaim`) | 9-tuple (without those two) |
| SignedPayload | 11-element tuple | 9-element tuple |

---

## chain_spec.rs

| Aspect | Upstream (256 lines) | zkv-service (92 lines) |
|--------|----------------------|------------------------|
| Chains | Polkadot, Kusama, Westend, Rococo, Versi, Paseo | Volta, ZkVerify |
| ChainSpec types | `WestendChainSpec`, `RococoChainSpec`, etc. | `ZkvChainSpec`, `VoltaChainSpec` |
| Extensions | `fork_blocks`, `bad_blocks`, `light_sync_state` | `light_sync_state` only |
| Config functions | 14 functions | 2 functions (`development_config`, `local_config`) |
| Telemetry endpoints | Configured per network | Not present |

---

## grandpa_support.rs

| Aspect | Upstream (166 lines) | zkv-service (56 lines) |
|--------|----------------------|------------------------|
| Functions | `walk_backwards_to_target_block` + `kusama_hard_forks` | `walk_backwards_to_target_block` only |
| Hard forks | 37 validator addresses, 9 Kusama fork definitions | Removed entirely |

---

## relay_chain_selection.rs

| Aspect | Upstream | zkv-service |
|--------|----------|-------------|
| Format strings | Old-style `format!("{:?}", e)` | New-style `format!("{e:?}")` |
| PriorityLevel import | Direct import | Fully qualified usage |
| Dispute message dispatch | Uses `send_msg_with_priority()` with `PriorityLevel::High` | Uses `send_msg()` without priority |

---

## workers.rs

| Aspect | Upstream | zkv-service |
|--------|----------|-------------|
| Worker binary names | `polkadot-prepare-worker`, `polkadot-execute-worker` | `zkv-relay-prepare-worker`, `zkv-relay-execute-worker` |
| Test temp dir storage | `thread_local! { RefCell<Option<TempDir>> }` (per-thread) | `OnceLock<Mutex<Option<PathBuf>>>` (global shared) |
| Test concurrency | No `#[serial]` (parallel-safe via thread-local isolation) | `#[serial]` on all tests (sequential execution) |
| Test initialization | `sp_tracing::init_for_tests()` | `env_logger::builder().is_test(true)...try_init()` |

Both approaches are thread-safe. Upstream uses per-thread isolation via `thread_local!` allowing
parallel test execution. zkv-service uses a global mutex with forced sequential execution via
`#[serial]`. The upstream approach is arguably more parallel-friendly.

---

## parachains_db/

### mod.rs

- **Upstream**: Includes `v0`, `v1`, `v2`, `v3` column definitions for migrations
- **zkv-service**: Only `v4` columns (starts fresh, no migration from old versions)

### upgrade.rs

- **Upstream**: Includes approval voting DB migration helpers (v1, v2, v3 adapters)
- **zkv-service**: No approval voting migrations; `db_kind` parameter marked unused

---

## rpc.rs (zkv-only)

Custom RPC module providing:
- `BabeDeps`, `GrandpaDeps`, `FullDeps` structures
- `create_full()` function instantiating all RPC extensions
- Integrates: system RPC, transaction payment, BABE, GRANDPA, aggregate proof, VK hash endpoints

Upstream handles RPC via the separate `polkadot-rpc` crate.

---

## fake_runtime_api.rs (upstream-only)

Provides stub runtime API implementations (`unimplemented!()`) for use without actual native
runtime binaries. Implements: Core, Metadata, BlockBuilder, TaggedTransactionQueue,
OffchainWorker, ParachainHost (40+ methods), BABE, GRANDPA, and more.

Not needed in zkv-service because the single `zkv-runtime` is always available.

---

## tests.rs

| Aspect | Upstream | zkv-service |
|--------|----------|-------------|
| Test client | `polkadot_test_client` | `test_client` (custom) |
| Subsystem helpers | `polkadot_node_subsystem_test_helpers` | `node_subsystem_test_helpers` (custom) |
| Priority handling | Uses `PriorityLevel::High`/`::Normal` | Simplified, ignores priority |
| Test init | `sp_tracing::init_for_tests()` | `env_logger::builder().is_test(true)` |

---

## tests/helpers.rs (zkv-only)

Test helper module providing dummy data generators for:
- Candidate receipts and committed candidate receipts
- Candidate descriptors and commitments
- Collator and validator IDs
- Head data, hashes, digests

Upstream has these in `polkadot-primitives-test-helpers`.

---

## Summary of Removable Differences

Differences that could potentially be aligned with upstream to reduce maintenance burden:

### Could remove (low effort)

1. **Format string modernization** in `relay_chain_selection.rs`: upstream may adopt `{e:?}`
   style in future versions, making this a temporary diff.
2. **Genesis hash retrieval**: Could switch to `client.chain_info().genesis_hash` to match
   upstream pattern.
3. **`HashSet` import style** in `overseer.rs`: cosmetic, could match upstream.
4. **Return type formatting** in `overseer.rs`: cosmetic, could match upstream.
5. **Test temp dir approach** in `workers.rs`: could adopt upstream's `thread_local!` pattern
   to allow parallel test execution and reduce diff.

### Could consider adding (medium effort)

6. **Litep2p network backend support**: Re-add network backend dispatch in `build_full()` if
   Litep2p is desired in the future.
7. **Priority-based message dispatch** in `relay_chain_selection.rs`: Re-add if dispute
   prioritization is needed.

### Intentional and should keep

8. **BEEFY removal**: Intentional design decision.
9. **MMR gadget removal**: Intentional, not needed.
10. **Single-runtime architecture**: Fundamental design choice.
11. **Custom host functions** (`native::HLNativeHostFunctions`): Essential for proof verification.
12. **Custom RPC** (`rpc.rs`): Needed for aggregate-rpc and vk-hash endpoints.
13. **CollationGeneration in validator overseer**: Intentional for zkVerify validators.
14. **Worker binary names**: Must match zkVerify binaries.
15. **Fetch chunks threshold logic** (inverted for zkVerify networks): Intentional tuning.
