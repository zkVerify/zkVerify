# Patch: cumulus-relay-chain-inprocess-interface

**Upstream crate**: `cumulus-relay-chain-inprocess-interface` from `polkadot-sdk` branch `polkadot-stable2512`

## Why patched

The in-process relay chain interface needs `zkv-cli` and `zkv-service` instead of `polkadot-cli`
and `polkadot-service`, since zkVerify has its own service layer and CLI.

## Changes from upstream

### Cargo.toml
- Replaced `polkadot-cli` with `zkv-cli`
- Replaced `polkadot-service` with `service` (zkv-service)
- Removed `cumulus-client-bootnodes` dependency (no `PolkadotServiceBuilder`)
- Replaced dev-dep `polkadot-test-client` with `test-client` (local path)

### src/lib.rs
- Added `#![allow(unused_parens)]` for Rust 1.90+ compatibility
- Removed `HashSet` import (no `invulnerable_ah_collators` field)
- Removed `cumulus_client_bootnodes::bootnode_request_response_config` import
- Replaced `polkadot_service::*` with `service::*`
- Removed `build_polkadot_with_paranode_protocol` function (`PolkadotServiceBuilder` not available)
- Restructured `build_polkadot_full_node` to call `service::build_full` directly
- Adjusted `NewFullParams` fields for zkv-service API (no `enable_beefy`, `keep_finalized_for`,
  `invulnerable_ah_collators`, `collator_protocol_hold_off`)
- Replaced `polkadot_cli::Cli` with `zkv_cli::Cli`
- Created dummy channel for paranode requests (not used in inprocess mode)
- Replaced `polkadot_test_client` with `test_client` in tests
