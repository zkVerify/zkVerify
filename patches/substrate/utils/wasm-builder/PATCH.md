# Patch: substrate-wasm-builder

**Upstream crate**: `substrate-wasm-builder` from `polkadot-sdk` branch `stable2512`

## Why patched

Adds `WASM_BUILD_LEGACY_TARGET` env var to force `wasm32-unknown-unknown` target when
dependencies don't support the stricter `wasm32v1-none` target.

## Changes from upstream

### src/lib.rs
- Added `WASM_BUILD_LEGACY_TARGET` constant and doc entry
- Added `RuntimeTarget::should_use_wasm32v1_none()` helper method
- Modified `rustc_target()`, `rustc_target_dir()`, `rustc_target_build_std()` to use it

### src/wasm_project.rs
- Changed target-cpu=mvp condition to use `target.rustc_target()` instead of
  `cargo_cmd.is_wasm32v1_none_target_available()`
- Added `-C target-feature=-sign-ext` alongside `-C target-cpu=mvp`
