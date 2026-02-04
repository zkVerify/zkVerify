# Polkadot SDK Upgrade: polkadot-stable2412 to polkadot-stable2512

## Overview

This document tracks the upgrade of zkVerify from Polkadot SDK stable December 2024 release (`polkadot-stable2412-7`) to the January 2025 release (`polkadot-stable2512`).

## Release Information

- **From**: `polkadot-stable2412-7`
- **To**: `polkadot-stable2512`
- **Repository**: https://github.com/paritytech/polkadot-sdk

## Upstream Changes Summary

### Runtime API Changes

1. **LazyBlock Execution Change** (Breaking)
   - `execute_block` and `check_inherents` runtime APIs now receive a `LazyBlock` instead of a `Block`
   - Both encode identically, but metadata consumers may observe `LazyBlock` in type representations
   - **Impact**: Low - encoding is compatible, primarily affects type-level consumers

2. **MmrApi Version Bump (v2 → v3)**
   - Added `generate_ancestry_proof` method
   - Previous `BeefyApi::generate_ancestry_proof` deprecated
   - **Impact**: Low - zkVerify doesn't use BEEFY/MMR extensively

3. **SlotSchedule Runtime API**
   - New API to determine parachain blocks per relay chain slot
   - Currently implemented for parachain runtimes only
   - **Impact**: None for relay chain

4. **Unapplied Slashes v2 Runtime API**
   - New `unapplied_slashes_v2` in v15
   - Original API preserved for backward compatibility
   - **Impact**: Low - additive change

### XCM Changes

1. **XcmPaymentApi `query_delivery_fees` Breaking Change** (Breaking)
   - Now requires specifying an asset ID parameter
   - Enables fee estimation for non-native assets
   - **Impact**: Medium - requires updating XCM config if using this API

2. **XCMP Double-Encoded Message Support**
   - New `XcmpMessageFormat::ConcatenatedOpaqueVersionedXcm` format
   - Receive-only for now; reduces decoding overhead
   - **Impact**: None - receive-only, automatic

3. **Transfer Abstraction for Remote Execution**
   - New `Transfer` abstraction for remote asset transfers via XCM V5
   - **Impact**: None - new feature, opt-in

### Subsystem/Overseer Changes

1. **Collator Protocol Pre-connect Mechanism**
   - Collators maintain persistent connections to backing group assignments
   - Improves connection reliability
   - **Impact**: None - internal improvement

2. **Backing Group Connectivity Optimization**
   - Reserved peer connections open immediately instead of waiting for timer tick
   - **Impact**: None - performance improvement

3. **Prospective Parachains Logging**
   - Enhanced logging for candidate processing errors
   - **Impact**: None - logging improvement

### PVF Changes

1. **Wasmtime Perfmap Support**
   - Added `perfmap` profiling via `WASMTIME_PROFILING_STRATEGY=perfmap`
   - **Impact**: None - opt-in debugging feature

### Other API Changes

1. **CheckWeight Extension Update**
   - Now factors in transaction length during validation
   - **Impact**: Low - prevents oversized transactions, mostly transparent

2. **Frame System Tasks Restriction**
   - Tasks now local-only; external source tasks invalid
   - **Impact**: None - security improvement

3. **Transactional Extension Hooks**
   - Extensions integrate with transactional system
   - Receive callbacks on transaction create/commit/revert
   - **Impact**: Low - internal improvement

## zkVerify-Specific Impact Assessment

### High Impact Areas

1. **Patched Crates** - Must be updated to match new upstream:
   - `polkadot-node-core-pvf-common` (adds native host functions)
   - `cumulus-relay-chain-inprocess-interface` (uses zkv-service)
   - `cumulus-relay-chain-minimal-node` (uses zkv-service)

2. **zkv-relay Service** - May need updates for:
   - Any changed subsystem configurations
   - New overseer parameters
   - Updated type signatures

### Medium Impact Areas

1. **XCM Configuration** - Review `query_delivery_fees` usage
2. **Runtime APIs** - Check for any direct usage of changed APIs

### Low Impact Areas

1. **Crate version bumps** - ~80+ crates to update
2. **Type changes** - LazyBlock/Block encoding compatible

## Migration Checklist

### Phase 1: Documentation
- [x] Fetch release notes
- [x] Analyze breaking changes
- [x] Document zkVerify-specific impacts

### Phase 2: Root Dependencies
- [ ] Run `python3 scripts/update-polkadot-sdk-deps.py polkadot-stable2512 --no-check --no-commit`
- [ ] Verify critical crate version updates
- [ ] Update patch section if needed

### Phase 3: Patched Crates
- [ ] Update `polkadot-node-core-pvf-common`
  - [ ] Copy upstream source
  - [ ] Re-apply native host functions patch
  - [ ] Update version in Cargo.toml
- [ ] Update `cumulus-relay-chain-inprocess-interface`
  - [ ] Copy upstream source
  - [ ] Re-apply zkv-service substitution
  - [ ] Update version in Cargo.toml
- [ ] Update `cumulus-relay-chain-minimal-node`
  - [ ] Copy upstream source
  - [ ] Re-apply zkv-service substitution
  - [ ] Update version in Cargo.toml

### Phase 4: zkv-relay Service
- [ ] Update `relay-node/service/src/lib.rs`
- [ ] Update `relay-node/service/src/overseer.rs`
- [ ] Update `relay-node/service/src/relay_chain_selection.rs`
- [ ] Update `relay-node/service/src/workers.rs`
- [ ] Update `relay-node/service/src/rpc.rs`
- [ ] Update `relay-node/service/Cargo.toml`

### Phase 5: Runtime Configuration
- [ ] Update `runtime/zkverify/src/lib.rs`
- [ ] Update `runtime/zkverify/src/xcm_config.rs`
- [ ] Update `runtime/zkverify/src/parachains.rs`
- [ ] Update `runtime/zkverify/Cargo.toml`

### Phase 6: Test Crates
- [ ] Update `relay-node/test/client/`
- [ ] Update `relay-node/test/runtime/`
- [ ] Update `relay-node/test/service/`
- [ ] Update `relay-node/test/subsystem-helpers/`
- [ ] Update `relay-node/test/substrate/client/`

### Phase 7: CLI and Workers
- [ ] Update `relay-node/Cargo.toml`
- [ ] Update `relay-node/src/main.rs`
- [ ] Update `relay-node/src/bin/execute-worker.rs`
- [ ] Update `relay-node/src/bin/prepare-worker.rs`
- [ ] Update `relay-node/cli/src/command.rs`

### Phase 8: Paratest
- [ ] Update `paratest/runtime/`
- [ ] Update `paratest/node/`

### Phase 9: Documentation
- [ ] Update README.md version references

### Phase 10: Verification
- [ ] `cargo build --release`
- [ ] `cargo build --release --features runtime-benchmarks,try-runtime`
- [ ] `cargo test`
- [ ] Manual verification with dev node
- [ ] Zombienet tests

## Version Mapping (Key Crates)

| Crate | Old Version | New Version |
|-------|-------------|-------------|
| polkadot-node-core-pvf-common | 17.0.1 | TBD |
| polkadot-overseer | 21.1.0 | TBD |
| polkadot-service | 22.2.0 | TBD |
| cumulus-relay-chain-inprocess-interface | 0.22.0 | TBD |
| cumulus-relay-chain-minimal-node | 0.22.1 | TBD |
| sc-executor | 0.41.0 | TBD |
| sp-runtime | 40.1.0 | TBD |
| frame-support | 39.1.0 | TBD |

*TBD = To be determined after running update script*

## Preserved zkVerify Customizations

The following customizations must be preserved through the upgrade:

### 1. Native Host Functions (PVF Common)
```rust
// In executor_interface.rs
type HostFunctions = (
    sp_io::misc::HostFunctions,
    sp_io::crypto::HostFunctions,
    sp_io::hashing::HostFunctions,
    sp_io::allocator::HostFunctions,
    sp_io::logging::HostFunctions,
    sp_io::trie::HostFunctions,
    native::HLNativeHostFunctions,  // zkVerify custom
);
```

### 2. zkv-service Integration (Cumulus Patches)
- `cumulus-relay-chain-inprocess-interface`: Uses `service` (zkv-service) instead of `polkadot-service`
- `cumulus-relay-chain-minimal-node`: Uses `service` (zkv-service) for overseer configuration

### 3. Chain Configuration (Service)
- `Chain` enum with `Volta`, `ZkVerify`, `Unknown` variants
- `IdentifyVariant` trait for chain identification
- `zkv_reference_hardware()` for hardware benchmarks

## Notes

- This upgrade follows a minimal sync approach: only apply changes necessary for compatibility
- The zkVerify-specific code structure should be preserved where possible
- Test thoroughly with zombienet after completing all phases
