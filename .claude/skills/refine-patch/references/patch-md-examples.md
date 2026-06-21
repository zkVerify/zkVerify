# PATCH.md Examples

These are real examples from the project's existing patches. Use them as style
reference when writing new PATCH.md files.

## Example 1: polkadot-node-core-pvf-common

```markdown
# Patch: polkadot-node-core-pvf-common

**Upstream crate**: `polkadot-node-core-pvf-common` from `polkadot-sdk` branch `polkadot-stable2512`

## Why patched

The PVF (Parachain Validation Function) worker needs zkVerify's native host functions
(`native::HLNativeHostFunctions`) to verify proofs during parachain validation.

## Changes from upstream

### Cargo.toml
- Added `native` dependency (workspace, default-features = true)
- Changed `license.workspace = true` to explicit `license = "GPL-3.0-only"` (workspace lacks `license` field)
- Replaced `[lints] workspace = true` with explicit `[lints.clippy]` section containing workspace lints plus upstream code suppressions (`doc_lazy_continuation`, `missing_safety_doc`, `redundant_closure`, `io_other_error`, `needless_return`, `redundant_pattern_matching`)

### src/executor_interface.rs
- Added `native::HLNativeHostFunctions` to the `HostFunctions` type tuple
```

## Example 2: cumulus-relay-chain-inprocess-interface

```markdown
# Patch: cumulus-relay-chain-inprocess-interface

**Upstream crate**: `cumulus-relay-chain-inprocess-interface` from `polkadot-sdk` branch `polkadot-stable2512`

## Why patched

The in-process relay chain interface needs `zkv-cli` and `zkv-service` instead of `polkadot-cli`
and `polkadot-service`, since zkVerify has its own service layer and CLI.

## Changes from upstream

### Cargo.toml
- Replaced `polkadot-cli` with `zkv-cli`
- Replaced `polkadot-service` with `service` (zkv-service)
- Replaced dev-dep `polkadot-test-client` with `test-client` (local path)
- Replaced `[lints] workspace = true` with explicit `[lints.clippy]` section containing workspace lints plus upstream code suppression (`type_complexity`)

### src/lib.rs
- Replaced `polkadot_service::*` with `service::*`
- Replaced `polkadot_service::builder::PolkadotServiceBuilder` with `service::builder::ServiceBuilder`
- Replaced `polkadot_cli::Cli` with `zkv_cli::Cli`
- Replaced `polkadot_test_client` with `test_client` in tests
- Removed `enable_beefy` field from `NewFullParams` (not supported in zkv-service)
```

## Example 3: substrate-wasm-builder

```markdown
# Patch: substrate-wasm-builder

**Upstream crate**: `substrate-wasm-builder` from `polkadot-sdk` branch `stable2512`

## Why patched

Adds `WASM_BUILD_LEGACY_TARGET` env var to force `wasm32-unknown-unknown` target when
dependencies don't support the stricter `wasm32v1-none` target.

## Changes from upstream

### src/lib.rs
- Added `WASM_BUILD_LEGACY_TARGET` constant and doc entry
- Added `is_forced_wasm_build_legacy_target()` top-level helper function
- Added `RuntimeTarget::should_use_wasm32v1_none()` helper method
- Modified `rustc_target()`, `rustc_target_dir()`, `rustc_target_build_std()` to use it

### src/prerequisites.rs
- Suppress "missing wasm32v1-none" warning when `WASM_BUILD_LEGACY_TARGET` is set
- Import and use `is_forced_wasm_build_legacy_target()` in the toolchain check

### src/wasm_project.rs
- Changed target-cpu=mvp condition to use `target.rustc_target()` instead of
  `cargo_cmd.is_wasm32v1_none_target_available()`
- Added `-C target-feature=-sign-ext` alongside `-C target-cpu=mvp`
```

## Style Rules

1. **Title**: `# Patch: <crate-name>` (use the Cargo crate name)
2. **Upstream reference**: Always specify repo and branch
3. **"Why patched"**: Brief, one-paragraph explanation of the motivation
4. **"Changes from upstream"**: Organized by file, with one bullet per logical change
5. **Bullets**: Start with a verb (Added, Changed, Replaced, Removed, Modified)
6. **Cargo.toml changes**: Always mention license and lints changes explicitly,
   including which upstream suppressions were added
7. **Source changes**: Reference specific types, functions, or imports that changed
