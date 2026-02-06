# Patch: polkadot-node-core-pvf-common

**Upstream crate**: `polkadot-node-core-pvf-common` from `polkadot-sdk` branch `polkadot-stable2512`

## Why patched

The PVF (Parachain Validation Function) worker needs zkVerify's native host functions
(`native::HLNativeHostFunctions`) to verify proofs during parachain validation.

## Changes from upstream

### Cargo.toml
- Added `native` dependency (workspace, default-features = true)
- Changed `license.workspace = true` to explicit `license = "GPL-3.0-only"` (workspace lacks `license` field)

### src/executor_interface.rs
- Added `native::HLNativeHostFunctions` to the `HostFunctions` type tuple
