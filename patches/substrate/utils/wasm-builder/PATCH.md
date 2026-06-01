# Patch: substrate-wasm-builder

**Upstream crate**: `substrate-wasm-builder` from `polkadot-sdk` branch `polkadot-stable2512`

## Why patched

Adds `WASM_BUILD_LEGACY_TARGET` env var to force `wasm32-unknown-unknown` target when
dependencies don't support the stricter `wasm32v1-none` target. Also restores
`--allow-undefined` linker flag removed from the `wasm32-unknown-unknown` target spec
in Rust 1.96, and adapts to the `polkavm-linker` API used in the workspace.

## Changes from upstream

### Cargo.toml
- Changed version from `31.0.0` to `31.1.0`

### src/lib.rs
- Added `WASM_BUILD_LEGACY_TARGET` constant and doc entry
- Added `is_forced_wasm_build_legacy_target()` top-level helper function
- Added `RuntimeTarget::should_use_wasm32v1_none()` helper method
- Modified `rustc_target()`, `rustc_target_dir()`, `rustc_target_build_std()` to use
  `should_use_wasm32v1_none()` instead of `is_wasm32v1_none_target_available()` directly
- Changed `polkavm_linker::target_json_32_path()` to `polkavm_linker::target_json_path(args)`
  to match the workspace `polkavm-linker` API

### src/prerequisites.rs
- Added `is_forced_wasm_build_legacy_target` import
- Added `!is_forced_wasm_build_legacy_target()` condition to suppress "missing wasm32v1-none"
  warning when legacy target is forced

### src/wasm_project.rs
- Added `use polkavm_linker::TargetInstructionSet`
- Changed target-cpu=mvp condition to use `target.rustc_target()` instead of
  `cargo_cmd.is_wasm32v1_none_target_available()`
- Added `-C target-feature=-sign-ext` alongside `-C target-cpu=mvp`
- Added `-C link-arg=--allow-undefined` to restore the flag removed from the
  `wasm32-unknown-unknown` target spec in Rust 1.96
- Changed `polkavm_linker::program_from_elf(config, &blob_bytes)` to
  `polkavm_linker::program_from_elf(config, TargetInstructionSet::Latest, &blob_bytes)`
  to match the workspace `polkavm-linker` API
