# Relay Node Migration from polkadot-stable2412 to polkadot-stable2512

## Overview

This document describes the relay-node-specific changes required to migrate zkVerify from Polkadot SDK `polkadot-stable2412` to `polkadot-stable2512` (corresponding to node version v1.21.0).

### Migration Process

1. Update patched crates to match new upstream
2. Update service code for new subsystem APIs
3. Update CLI for new benchmark APIs
4. Update test infrastructure for SDK changes

## Major Changes

### 1. BABE Block Import Type Signature

**Location**: `relay-node/service/src/lib.rs`

**Change**: `BabeBlockImport` now requires 5 generic parameters instead of 3.

```rust
// Before (polkadot-stable2412)
type FullBabeBlockImport = babe::BabeBlockImport<Block, FullClient, FullGrandpaBlockImport<ChainSelection>>;

// After (polkadot-stable2512)
type FullBabeBlockImport = babe::BabeBlockImport<
    Block,
    FullClient,
    FullGrandpaBlockImport<ChainSelection>,
    BabeCreateInherentDataProviders<Block>,
    ChainSelection,
>;
```

**Additional**: New type alias and import required:

```rust
use sp_consensus_babe::inherents::BabeCreateInherentDataProviders;

type BabeCreateInherentDataProviders<Block> = Arc<
    dyn CreateInherentDataProviders<
        Block,
        (),
        InherentDataProviders = (
            sp_consensus_babe::inherents::InherentDataProvider,
            sp_timestamp::InherentDataProvider,
        ),
    > + Send + Sync,
>;
```

### 2. Inherent Data Providers Construction

**Location**: `relay-node/service/src/lib.rs` (in `new_full` function)

**Change**: Create inherent data providers must now be wrapped in `Arc` and passed to BABE:

```rust
let create_inherent_data_providers: BabeCreateInherentDataProviders<Block> =
    Arc::new(move |_, ()| async move {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
        let slot = sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
            *timestamp,
            slot_duration,
        );
        Ok((slot, timestamp))
    });
```

### 3. NewFullParams Changes

**Location**: `relay-node/service/src/lib.rs`, `relay-node/cli/src/command.rs`

**Change**: The `enable_approval_voting_parallel` field has been removed from `NewFullParams`.

```rust
// Before
service::NewFullParams {
    enable_approval_voting_parallel: false,
    // ...
}

// After - field removed entirely
service::NewFullParams {
    // enable_approval_voting_parallel removed
    // ...
}
```

### 4. OverseerHandleWithPriorityT Trait

**Location**: `relay-node/service/src/relay_chain_selection.rs`

**Change**: New trait `OverseerHandleWithPriorityT` added for priority-based message sending. `SelectRelayChainInner` now requires both `OverseerHandleT` and `OverseerHandleWithPriorityT`.

```rust
#[async_trait::async_trait]
pub trait OverseerHandleWithPriorityT: Clone + Send + Sync {
    async fn send_msg_with_priority<M: Send + Into<AllMessages>>(
        &mut self,
        msg: M,
        origin: &'static str,
        priority: polkadot_overseer::PriorityLevel,
    );
}
```

### 5. SelectRelayChainInner::new Signature

**Location**: `relay-node/service/src/relay_chain_selection.rs`

**Change**: The 5th parameter (`approval_voting_parallel_enabled`) has been removed.

```rust
// Before (5 parameters)
SelectRelayChainInner::new(backend, overseer, metrics, spawn_handle, approval_voting_parallel_enabled)

// After (4 parameters)
SelectRelayChainInner::new(backend, overseer, metrics, spawn_handle)
```

### 6. ApprovalVotingParallelMessage

**Location**: Various subsystem code

**Change**: `ApprovalVotingMessage::ApprovedAncestor` has been replaced with `ApprovalVotingParallelMessage::ApprovedAncestor` for approval voting queries with priority support.

```rust
// Before
AllMessages::ApprovalVoting(ApprovalVotingMessage::ApprovedAncestor(...))

// After
AllMessages::ApprovalVotingParallel(ApprovalVotingParallelMessage::ApprovedAncestor(...))
```

### 7. StorageCmd::run API Change

**Location**: `relay-node/cli/src/command.rs`

**Change**: `StorageCmd::run` now requires a 5th argument for trie cache.

```rust
// Before
cmd.run(config, client.clone(), db, storage)

// After
cmd.run(config, client.clone(), db, storage, None)
```

### 8. Configuration Struct Changes

**Location**: `relay-node/test/service/src/lib.rs`

**New required fields**:
- `request_logger_limit: u32` (typically `0`)
- `warm_up_trie_cache: Option<TrieCacheWarmUpStrategy>` (typically `None`)

```rust
Configuration {
    // ...
    rpc: RpcConfiguration {
        // ...
        request_logger_limit: 0,  // NEW
    },
    // ...
    warm_up_trie_cache: None,  // NEW
}
```

## Test Infrastructure Changes

### 1. TestSubsystemSender Trait Implementation

**Location**: `relay-node/service/src/tests.rs`

**Change**: `TestSubsystemSender` must now implement `OverseerHandleWithPriorityT` in addition to `OverseerHandleT`.

```rust
#[async_trait::async_trait]
impl OverseerHandleWithPriorityT for TestSubsystemSender {
    async fn send_msg_with_priority<M: Send + Into<AllMessages>>(
        &mut self,
        msg: M,
        _origin: &'static str,
        _priority: polkadot_overseer::PriorityLevel,
    ) {
        // For tests, ignore priority and just send the message
        TestSubsystemSender::send_message(self, msg.into()).await;
    }
}
```

### 2. VirtualOverseer Type Update

**Location**: `relay-node/service/src/tests.rs`

**Change**: Update to use `ApprovalVotingParallelMessage`:

```rust
// Before
type VirtualOverseer = TestSubsystemContextHandle<ApprovalVotingMessage>;

// After
type VirtualOverseer = TestSubsystemContextHandle<ApprovalVotingParallelMessage>;
```

### 3. Test Assertion Updates

**Location**: `relay-node/service/src/tests.rs`

**Change**: Update message matching in test assertions:

```rust
// Before
AllMessages::ApprovalVoting(ApprovalVotingMessage::ApprovedAncestor(_block_hash, _block_number, tx))

// After
AllMessages::ApprovalVotingParallel(ApprovalVotingParallelMessage::ApprovedAncestor(_block_hash, _block_number, tx))
```

### 4. AccountKeyring Import Change

**Location**: `relay-node/test/substrate/client/src/lib.rs`, `relay-node/test/service/src/lib.rs`

**Change**: `AccountKeyring` has been removed from `sp_keyring` root.

```rust
// Before
use sp_keyring::AccountKeyring;

// After
pub use sp_keyring::sr25519::Keyring as AccountKeyring;
// or use Sr25519Keyring directly
```

## Test Runtime Changes

### 1. LazyBlock Type

**Location**: `relay-node/test/runtime/src/lib.rs`

**Change**: `execute_block` and `check_inherents` now use `LazyBlock` instead of `Block`.

```rust
pub type LazyBlock = generic::LazyBlock<Header, UncheckedExtrinsic>;

impl_runtime_apis! {
    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn check_inherents(block: LazyBlock, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block.into())
        }
    }

    impl sp_api::Core<Block> for Runtime {
        fn execute_block(block: LazyBlock) {
            Executive::execute_block(block.into());
        }
    }
}
```

### 2. vstaging Module Removals

**Location**: `relay-node/test/runtime/src/lib.rs`

**Change**: Many types moved from `polkadot_primitives::vstaging` to main module:

```rust
// Before
use polkadot_primitives::vstaging::{CandidateEvent, CoreState, ScrapedOnChainVotes, ...};

// After
use polkadot_primitives::{CandidateEvent, CoreState, ScrapedOnChainVotes, ...};
```

### 3. Pallet Session Config

**Location**: `relay-node/test/runtime/src/lib.rs`

**Change**: `DisabledValidatorsThreshold` removed, `DisablingStrategy` added:

```rust
impl pallet_session::Config for Runtime {
    // REMOVED: type DisabledValidatorsThreshold = ...;
    type DisablingStrategy = pallet_session::disabling::UpToLimitDisablingStrategy;  // NEW
}
```

### 4. Pallet Staking Config

**Location**: `relay-node/test/runtime/src/lib.rs`

**Change**: `StashOf` removed, need custom `ValidatorIdOf`:

```rust
// NEW: Custom ValidatorIdOf struct
pub struct ValidatorIdOf;
impl sp_runtime::traits::Convert<AccountId, Option<AccountId>> for ValidatorIdOf {
    fn convert(a: AccountId) -> Option<AccountId> {
        Some(a)
    }
}

impl pallet_staking::Config for Runtime {
    type ValidatorIdOf = ValidatorIdOf;  // Changed from pallet_staking::StashOf<Self>
    type DisablingStrategy = pallet_staking::UpToLimitDisablingStrategy;  // NEW
}
```

### 5. Runtime API Implementation Changes

**Location**: `relay-node/test/runtime/src/lib.rs`

**Change**: Use `runtime_api_impl::v13` and update return types:

```rust
use polkadot_runtime_parachains::runtime_api_impl::v13 as runtime_impl;

// unapplied_slashes now returns LegacyPendingSlashes
fn unapplied_slashes() -> Vec<(SessionIndex, CandidateHash, slashing::LegacyPendingSlashes)> {
    // ...
}

// validation_code_bomb_limit moved from staging to v13
fn validation_code_bomb_limit() -> u32 {
    runtime_impl::validation_code_bomb_limit::<Runtime>()
}
```

### 6. XCM Config Updates

**Location**: `relay-node/test/runtime/src/xcm_config.rs`

**New required types**:

```rust
impl xcm_executor::Config for XcmConfig {
    type XcmEventEmitter = super::Xcm;  // NEW
    // ...
}

impl pallet_xcm::Config for Runtime {
    type AuthorizedAliasConsideration = ();  // NEW
    // ...
}
```

## Patched Crates

### polkadot-node-core-pvf-common

**Location**: `patches/polkadot/node/core/pvf/common/`

**Action**: Copy from upstream `polkadot-stable2512` and re-apply zkVerify modifications:

1. Add `native = { workspace = true }` dependency in `Cargo.toml`
2. In `src/executor_interface.rs`, add `native::HLNativeHostFunctions` to the HostFunctions tuple

## Linker Configuration

**Location**: `.cargo/config.toml`

**Change**: Added C++ standard library linking for rocksdb with lld:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "/usr/bin/cc"
rustflags = ["-C", "link-arg=-fuse-ld=lld", "-C", "link-arg=-lstdc++"]
```

## Verification

After the upgrade, verify:

1. `cargo build -p zkv-relay` succeeds
2. `cargo build -p zkv-relay --features runtime-benchmarks` succeeds
3. `cargo build -p zkv-relay --features try-runtime` succeeds
4. `cargo test -p zkv-service` passes (16 tests)
5. `cargo test -p test-service -p test-runtime -p substrate-test-client` passes
6. `./target/release/zkv-relay --version` shows correct version
7. Worker binaries (`zkv-relay-execute-worker`, `zkv-relay-prepare-worker`) build and show correct version

## Known Issues

### trie-db Future Incompatibility Warning

After building, you may see the following warning:

```
warning: the following packages contain code that will be rejected by a future version of Rust: trie-db v0.30.0
```

**Status**: This is an **upstream issue** in Polkadot SDK.

**Details**:
- Polkadot SDK `polkadot-stable2512` uses `trie-db = "0.30.0"`
- The warning is about "never type fallback" that will become a hard error in Rust 2024 edition
- A fix exists in `trie-db v0.31.0` (available on crates.io)
- Polkadot SDK has not yet upgraded to `trie-db v0.31.0`

**Impact**: None for now. The warning only affects Rust 2024 edition. Since zkVerify (and Polkadot SDK) uses `edition = "2021"`, this will not cause build failures.

**Resolution**: This will be fixed when Polkadot SDK upgrades to `trie-db v0.31.0` in a future stable release (likely `polkadot-stable2601` or later).

**Workaround**: If needed, you can patch `trie-db` to the git master branch, but this is not recommended as it may introduce compatibility issues:

```toml
[patch.crates-io]
trie-db = { git = "https://github.com/paritytech/trie", branch = "master" }
```

## References

- [Polkadot SDK Releases](https://github.com/paritytech/polkadot-sdk/releases)
- [Approval Voting Parallel Subsystem](https://github.com/paritytech/polkadot-sdk/pull/5190)
- [BABE Block Import Refactoring](https://github.com/paritytech/polkadot-sdk/pull/6645)
- [trie-db Repository](https://github.com/paritytech/trie)
