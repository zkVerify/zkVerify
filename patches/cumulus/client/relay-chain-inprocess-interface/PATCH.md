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
