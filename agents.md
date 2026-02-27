# agents.md

This file provides behavioral guidelines for AI coding agents working autonomously in this repository. It complements `CLAUDE.md` (project knowledge) with agent-specific safety boundaries, decision rules, and workflows.

> **Prerequisites**: Read and understand `CLAUDE.md` before using this file.

## Safety Zones — Require User Confirmation

These areas are security-critical or have chain-wide impact. **Never modify without explicit user approval:**

1. **Cryptographic verification code**
   - `native/` — all sub-crates (the actual proof verification implementations)
   - `verifiers/*/src/lib.rs` — `Verifier` trait implementations
   - Any code path that decides whether a proof is valid or invalid

2. **Runtime storage and migrations**
   - `runtime/zkverify/src/lib.rs` — runtime version, `Executive`, storage migrations
   - Any `#[pallet::storage]` type changes (requires a storage migration)
   - Pallet indices (reordering or reassigning breaks on-chain state)

3. **Aggregate dispatch path**
   - `pallets/aggregate/` — Merkle root computation, cross-chain dispatch logic
   - `primitives/hp-dispatch/` — `DispatchAggregation` trait and implementations

4. **Vendored patches**
   - `patches/` — upstream polkadot-sdk patches; edits here can silently break the upstream diff

## Decision Guidelines

### NEVER do without user confirmation

- Modify anything under `native/` (proof verification implementations)
- Change `#[pallet::storage]` types or add storage migrations
- Alter pallet indices in the runtime
- Edit the `patches/` directory
- Delete or rename public extrinsics (breaking the on-chain API)
- Change weight calculations, fee logic, or economic parameters
- Modify genesis config or chain spec defaults

### ASK the user when

- A change affects multiple pallets or crosses pallet boundaries
- Unsure whether a change requires a storage migration
- Adding new external crate dependencies to the workspace
- Modifying runtime configuration constants (e.g., epoch length, aggregation size)
- Performance implications of a change are unclear
- A fix touches both `native/` and `verifiers/` (may need chain sync verification)

## Workflow: Adding a New Verifier

This is the most common non-trivial extension to the codebase. The steps encode project-specific knowledge that is not obvious from reading code alone.

1. **Create the verifier crate** — copy an existing one (e.g., `verifiers/groth16/`) as a template; implement the `Verifier` trait
2. **Consider a native implementation** — only create `native/<name>/` if the verification is too slow in WASM and the performance gain justifies the added complexity; prefer a pure WASM implementation when possible
3. **Instantiate in runtime** — add the pallet to `runtime/zkverify/src/lib.rs` with a **new, unused** pallet index (do not reuse or reorder existing indices)
4. **Wire up callbacks** — register the new pallet with the `Aggregate` pallet via `OnProofVerified`
5. **Implement benchmarks** — if the verification weight does not depend on proof parameters (size, public inputs), implement only `pallet_verifiers::WeightInfo::verify_proof()` as a fixed upper bound (see `pallet-tee-verifier` for an example). If the weight varies with parameters, `WeightInfo::verify_proof()` returns the upper bound and `Verifier::verify_proof()` returns the actual weight as `Ok(Some(weight))` (see `pallet-risc0-verifier` for this pattern)
6. **Run `zepter`** — `zepter run check-fix && zepter run format-fix` (or `cargo make zepter`) to fix feature propagation in all affected `Cargo.toml` files
6. **Add zombienet test data** — create `zombienet-tests/js_scripts/<name>_data.js` with proof/vk/pubs fixtures
7. **Add zombienet test** — create a `.zndsl` test (start from `0007-proof_with_vk.zndsl` as template)
8. **Verify** — run pallet tests, runtime tests, and the new zombienet test

## Verification After Changes

After modifying a pallet, also verify its dependents. To find what depends on a package:

```bash
cargo tree -p <package> --invert
```

For changes to `native/`: the agent **cannot** perform the required full chain sync test. Flag this to the user so they can run it before merging.
