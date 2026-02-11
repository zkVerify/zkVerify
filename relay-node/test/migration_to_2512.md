# Migration of relay-node/test crates from polkadot-stable2412 to polkadot-stable2512

## Forked Crates

| Local crate | Path | Upstream (polkadot-sdk) |
|---|---|---|
| `node-subsystem-test-helpers` | `relay-node/test/subsystem-helpers/` | `polkadot-node-subsystem-test-helpers` |
| `test-runtime` | `relay-node/test/runtime/` | `polkadot-test-runtime` |
| `test-runtime-constants` | `relay-node/test/runtime/constants/` | `polkadot-test-runtime-constants` |
| `substrate-test-client` | `relay-node/test/substrate/client/` | `substrate-test-client` |
| `test-client` | `relay-node/test/client/` | `polkadot-test-client` |
| `test-service` | `relay-node/test/service/` | `polkadot-test-service` |

## Upstream Changes (stable2412 -> stable2512)

### 1. subsystem-helpers
- **Cargo.toml**: Formatting only (dotted style, alphabetical sort). No dep changes.
- **Source**: No changes.

### 2. test-runtime (MAJOR)
- **Cargo.toml**: Formatting. Removed `[dev-dependencies]` section. Removed `only-staking` feature. Added `pallet-session/runtime-benchmarks` and `xcm/runtime-benchmarks` to runtime-benchmarks feature.
- **src/lib.rs**:
  - `runtime_api_impl::v11` -> `v13` + new `vstaging as staging_runtime_impl`
  - `vstaging::*` types (CandidateEvent, CommittedCandidateReceiptV2, CoreState, ScrapedOnChainVotes, InherentData) promoted to top-level `polkadot_primitives`
  - New import: `async_backing::Constraints`
  - `CreateInherent` renamed to `CreateBare`
  - New `CreateAuthorizedTransaction` trait impl
  - `execute_block` / `check_inherents` now take `LazyBlock` instead of `Block`
  - `ParachainHost` API bumped from v11 to v15
  - New runtime API methods: `unapplied_slashes_v2`, `backing_constraints`, `scheduling_lookahead`, `validation_code_bomb_limit`, `para_ids`
  - `slashing::PendingSlashes` -> `slashing::LegacyPendingSlashes`
  - Pallet config changes:
    - `pallet_session::Config`: `ValidatorIdOf` -> `ConvertInto`, new `DisablingStrategy`, `Currency`, `KeyDeposit`
    - `pallet_session::historical::Config`: new `RuntimeEvent`, simplified `FullIdentification`/`FullIdentificationOf`
    - `frame_election_provider_support::onchain::Config`: `MaxWinners` -> `MaxWinnersPerPage`, new `MaxBackersPerWinner`, `Sort`
    - `pallet_staking::Config`: new `OldCurrency`, `RuntimeHoldReason`, `MaxValidatorSet`, `Filter`; removed `DisablingStrategy`
    - `parachains_inclusion::Config`: `RewardValidatorsWithEraPoints` now takes 2 generics
    - `parachains_paras::Config`: new `Fungible`, `CooldownRemovalMultiplier`, `AuthorizeCurrentCodeOrigin`
    - `coretime::Config`: `Currency` removed
  - TxExtension: new `AuthorizeCall` (beginning) and `WeightReclaim` (end)
  - `GrandpaApi::current_set_id()` changed to storage read
  - `generate_ancestry_proof` moved from BeefyApi to MmrApi (with different return type)
- **src/xcm_config.rs**: New `Disabled` import, new `XcmEventEmitter` on xcm_executor, new `AuthorizedAliasConsideration` on pallet_xcm.

### 3. test-runtime-constants
- **Cargo.toml**: Formatting only.
- **Source**: No changes.

### 4. substrate-test-client
- **Cargo.toml**: Formatting. Removed `sc-offchain`, `sp-state-machine`. `sc-client-db` explicit `default-features = false`. `sc-service` removed `features = ["test-helpers"]`, added `default-features = false`.
- **src/lib.rs**: `sp_keyring::{ed25519::Keyring as Ed25519Keyring, sr25519::Keyring as Sr25519Keyring, AccountKeyring}` -> `sp_keyring::{Ed25519Keyring, Sr25519Keyring}`. `AccountKeyring` re-export removed.
- **src/client_ext.rs**: No changes.

### 5. test-client (polkadot-test-client)
- **Cargo.toml**: Formatting. Removed `polkadot-node-subsystem`, `sc-offchain`, `sp-core` deps.
- **src/lib.rs**: No changes.
- **src/block_builder.rs**: `vstaging::InherentData` -> `InherentData` (promoted). `BlockBuilder` return types now have explicit lifetime `'_`.

### 6. test-service (polkadot-test-service) (MAJOR)
- **Cargo.toml**: Formatting. Removed `hex`, `gum`, `tempfile`, `polkadot-rpc`, `sc-authority-discovery`, `sc-consensus-babe`, `sp-consensus-grandpa`, `sp-inherents`, `sc-transaction-pool`, `substrate-test-utils`.
- **src/chain_spec.rs**: New `NodeFeatures` setup (CandidateReceiptV2, ElasticScalingMVP enabled).
- **src/lib.rs**:
  - New `NetworkService` import
  - `new_full` new param `custom_log_prefix: Option<&'static str>`
  - `NewFullParams` changes: removed `enable_approval_voting_parallel`, new `keep_finalized_for`, `invulnerable_ah_collators`, `collator_protocol_hold_off`
  - Transport from in-memory to real TCP
  - New `get_listen_address()` async function
  - `run_validator_node` / `run_collator_node` now `async`
  - TxExtension: new `AuthorizeCall` + `WeightReclaim`
  - `AccountKeyring` -> `Sr25519Keyring`

## zkVerify-Specific Customizations (to preserve)

### subsystem-helpers
- Crate renamed to `node-subsystem-test-helpers`
- Removed `polkadot-erasure-coding`, `polkadot-node-primitives` deps
- Removed `derive_erasure_chunks_with_proofs_and_root()` function
- Simplified `TestSubsystemContext`: removed `message_buffer` field and `peek()` method
- Removed `new_block_import_info()` from mock.rs

### test-runtime
- Crate renamed to `test-runtime`
- Removed BEEFY and MMR impls entirely (BeefyApi + MmrApi blocks deleted)
- `codec` alias replaced with explicit `parity-scale-codec`
- Added `try-runtime` feature
- Some pallet configs already partially updated (session, staking, etc.)
- `runtime_api_impl` already at v13, ParachainHost already at v12
- Already has `LazyBlock`, `create_bare()`, `validation_code_bomb_limit()`

### test-runtime-constants
- Added `sp-weights` and `sp-core` explicit deps
- `smallvec` explicit version instead of workspace

### substrate-test-client
- `AccountKeyring` re-export preserved as `sp_keyring::sr25519::Keyring as AccountKeyring`
- `sc-offchain` already removed
- `array-bytes` explicit version instead of workspace

### test-client
- Crate renamed to `test-client`
- Added `zkv-runtime` dep (for `GetLastTimestamp`)
- `Executor` type uses `test_service::WasmExecutor`

### test-service
- Crate renamed to `test-service`
- Uses `service` (zkv-service) instead of `polkadot-service` for core types
- Removed BEEFY support, Litep2p support
- Hardcoded overseer gen (no `OverseerGen` generic)
- `WasmExecutor` type alias defined locally
- `GetLastTimestamp` from `zkv_runtime` instead of `polkadot_service`
- Already has `keep_finalized_for`, `invulnerable_ah_collators`, `collator_protocol_hold_off` in NewFullParams

## Migration Plan

### Crate 1: subsystem-helpers (TRIVIAL)
No upstream source changes. Nothing to do.

### Crate 2: test-runtime-constants (TRIVIAL)
No upstream source changes. Nothing to do.

### Crate 3: substrate-test-client
- Update `sp_keyring` re-exports: `Ed25519Keyring`/`Sr25519Keyring` moved to top-level
- Check if `AccountKeyring` re-export still works
- Remove `sp-state-machine` dep if present
- Update `sc-client-db` and `sc-service` default-features

### Crate 4: test-client
- Remove `polkadot-node-subsystem` dep if present
- Remove `sp-core` dep if not needed
- Add explicit lifetime `'_` to `BlockBuilder` return types

### Crate 5: test-runtime (MAJOR)
- Bump `ParachainHost` API from v12 to v15
- Add new runtime API methods: `unapplied_slashes_v2`, `backing_constraints`, `scheduling_lookahead`, `para_ids`
- Apply pallet config changes not yet applied:
  - `pallet_session::Config`: `ValidatorIdOf` -> `ConvertInto` (if not already done)
  - `pallet_session::historical::Config`: `FullIdentificationOf` -> `UnitIdentificationOf`
  - `onchain::Config`: `MaxBackersPerWinner`, `Sort` type changes (our `ConstU32<512>` -> upstream `ConstU32<u32::MAX>`, our `Sort = ()` -> upstream `Sort = ConstBool<true>`)
  - `pallet_staking::Config`: check `Filter` (our `()` vs upstream `Nothing`)
  - `parachains_paras::Config`: check if we already have `Fungible`, `CooldownRemovalMultiplier`, `AuthorizeCurrentCodeOrigin`
- Add new `CreateAuthorizedTransaction` trait impl
- Update `generate_ancestry_proof` (was BeefyApi, now MmrApi — but we deleted both, so may need MmrApi back or skip)
- TxExtension: verify `AuthorizeCall` and `WeightReclaim` are present
- `xcm_config.rs`: verify `XcmEventEmitter` and `AuthorizedAliasConsideration` are present

### Crate 6: test-service (MAJOR)
- `new_full`: add `custom_log_prefix` parameter
- `NewFullParams`: verify field alignment with upstream (remove `enable_approval_voting_parallel` if still present)
- Transport: change from MemoryOnly to real TCP (if not already done)
- Add `get_listen_address()` async function
- Make `run_validator_node` / `run_collator_node` async
- TxExtension: verify `AuthorizeCall` / `WeightReclaim`
- `AccountKeyring` -> `Sr25519Keyring`
- `chain_spec.rs`: add `NodeFeatures` setup

## Migration Status

- [x] subsystem-helpers — No upstream changes between 2412 and 2512. Nothing to do.
- [x] test-runtime-constants — No upstream changes between 2412 and 2512. Nothing to do.
- [x] substrate-test-client — No upstream source changes needed (already aligned). Nothing to do.
- [x] test-client — No upstream source changes needed (already aligned). Nothing to do.
- [x] test-runtime — Migrated (see changes below)
- [x] test-service — Migrated (see changes below)
- [x] Integration tests pass (all 6 crates, including build-blocks and call-function integration tests)

## Changes Applied

### test-runtime/src/lib.rs
1. **Added `vstaging` runtime API import** — `runtime_api_impl::{v13 as runtime_impl, vstaging as staging_runtime_impl}` (needed for `para_ids` API)
2. **Added `Constraints` and `ConstBool` imports** — needed for new API methods and config
3. **Added `CreateAuthorizedTransaction` trait impl** — new upstream requirement for authorized transaction creation
4. **Updated `TxExtension`** — added `AuthorizeCall<Runtime>` (first) and `WeightReclaim<Runtime>` (last), making it a 10-element tuple
5. **Updated `create_signed_transaction`** — TxExtension construction and implicit data tuple updated to match
6. **Updated `DisablingStrategy`** — `UpToLimitDisablingStrategy` -> `UpToLimitWithReEnablingDisablingStrategy`
7. **Updated `historical::Config`** — `FullIdentification` changed from `Exposure<AccountId, Balance>` to `()`, `FullIdentificationOf` from `DefaultExposureOf` to `UnitIdentificationOf`
8. **Updated `onchain::Config`** — `OnChainMaxWinners` from `u32::MAX` to `MaxAuthorities::get()`, `MaxBackersPerWinner` from `ConstU32<512>` to `ConstU32<{ u32::MAX }>`, `Sort` from `()` to `ConstBool<true>`
9. **Updated `pallet_staking::Config::Filter`** — from `()` to `frame_support::traits::Nothing`
10. **Bumped `ParachainHost` API** — v12 -> v15
11. **Added 4 new runtime API methods** — `unapplied_slashes_v2`, `backing_constraints`, `scheduling_lookahead`, `para_ids`
12. **Added `#[allow(deprecated)]`** — on `backing_state`, `async_backing_params`, and `pallet_test_notifier::RuntimeEvent`
13. **Updated `GrandpaApi::current_set_id`** — from `Grandpa::current_set_id()` to `pallet_grandpa::CurrentSetId::<Runtime>::get()`

### test-runtime/src/xcm_config.rs
1. **Added `Disabled` import** from `frame_support::traits`
2. **Changed `AuthorizedAliasConsideration`** — from `()` to `Disabled`

### test-service/src/lib.rs
1. **Updated `construct_extrinsic`** — TxExtension construction updated with `AuthorizeCall` + `WeightReclaim`, implicit data tuple updated to 10 elements

### test-service/src/chain_spec.rs
1. **Added `NodeFeatures` imports** — `node_features`, `NodeFeatures`
2. **Added `NodeFeatures` configuration** — enables `CandidateReceiptV2` and `ElasticScalingMVP` in test genesis config

### Intentionally NOT applied (zkVerify-specific divergences preserved)
- **Transport**: kept `MemoryOnly` (upstream switched to TCP — simpler for our test setup)
- **`run_validator_node` / `run_collator_node`**: kept synchronous (upstream made async due to TCP address polling)
- **`get_listen_address`**: not added (only needed for TCP transport)
- **`new_full` `custom_log_prefix` parameter**: not added (upstream-only feature, our simplified signature)
- **`OverseerGen` generic parameter**: kept hardcoded (intentional simplification)
- **`ValidatorIdOf` custom struct**: kept (intentional divergence from upstream `ConvertInto`)
- **Removed BEEFY/MMR**: kept removed (not supported in zkVerify)
- **`message_buffer`/`peek` in subsystem-helpers**: kept removed (intentional simplification)
