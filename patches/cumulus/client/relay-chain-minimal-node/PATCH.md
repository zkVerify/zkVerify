# Patch: cumulus-relay-chain-minimal-node

**Upstream crate**: `cumulus-relay-chain-minimal-node` from `polkadot-sdk` branch `polkadot-stable2512`

## Why patched

The minimal relay chain node needs `zkv-service` instead of `polkadot-service` for overseer
generation, since zkVerify has its own service layer.

## Changes from upstream

### Cargo.toml
- Replaced `polkadot-service` dependency with `service` (zkv-service)
- Replaced `[lints] workspace = true` with explicit `[lints.clippy]` section containing workspace lints plus upstream code suppressions (`type_complexity`, `too_many_arguments`, `extra_unused_type_parameters`)

### src/lib.rs
- Changed `polkadot_service::{overseer::OverseerGenArgs, IsParachainNode}` to `service::{..}`

### src/collator_overseer.rs
- Changed `polkadot_service::overseer::{collator_overseer_builder, OverseerGenArgs}` to `service::overseer::{..}`
