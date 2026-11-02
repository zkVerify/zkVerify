# Patch: polkadot-node-core-candidate-validation

**Upstream crate**: `polkadot-node-core-candidate-validation` 28.0.0, from crates.io
(the `polkadot-stable2512` line zkVerify builds against)

## Why patched

Backport of [polkadot-sdk PR 13637](https://github.com/paritytech/polkadot-sdk/pull/13637),
"Check the validation data".

Candidate validation in this version does not check that the supplied
`PersistedValidationData` hashes to the `persisted_validation_data_hash` committed in the
candidate descriptor. The PVD reaching validation is not bound to anything the relay-chain
runtime verifies, so the check has to happen node-side. See the upstream PR for the
rationale.

## Why vendored rather than bumped

There is no patch release on our line: `polkadot-node-core-candidate-validation` goes
28.0.0 → 29.0.0 → … → 36.0.0, with no 28.0.x. The fix is published in 36.0.0, but
reaching it means moving the whole node dependency graph (and
`polkadot-node-primitives` 23.0.0 → 28.0.0), which is a polkadot-sdk line upgrade rather
than a security-fix-sized change. The only upstream backport label is
`A4-backport-stable2609`.

## Changes from upstream

### src/lib.rs
- In `validate_candidate_exhaustive`, before `perform_basic_checks`: reject a candidate
  whose `persisted_validation_data` does not hash to the descriptor's
  `persisted_validation_data_hash`.

### Cargo.toml

- Changed the `rstest` dev-dependency from upstream's pinned `version = "0.18.2"` to
  `workspace = true`, so the whole workspace builds a single `rstest` (0.24.0). Upstream's
  pin was the only thing pulling a second copy of `rstest`/`rstest_macros` into
  `Cargo.lock`. The crate's only use of `rstest` is a plain `#[rstest]` + `#[case]` pair
  in `src/tests.rs`, which works identically under both rstest versions.

- Restored the five dev-dependencies `src/tests.rs` needs:
  `polkadot-node-subsystem-test-helpers`, `polkadot-primitives-test-helpers`, `sp-core`,
  `sp-keyring` and `sp-maybe-compressed-blob`. They are path dependencies in the
  polkadot-sdk workspace and therefore unpublished, so crates.io normalisation strips
  them from the published manifest; upstream never notices because its own CI builds the
  tests against the workspace. Here the crate is a workspace member, so
  `cargo test --lib` builds its lib-test target and fails to compile without them.
  Points to note:
  - `polkadot-node-subsystem-test-helpers` is renamed onto the in-tree
    `relay-node/test/subsystem-helpers` (package `node-subsystem-test-helpers`), the same
    crate `relay-node/service` already uses.
  - `polkadot-primitives-test-helpers` 0.2.0 is taken from crates.io; it depends on
    `polkadot-primitives` 22.0.0, the version this line builds against, and adds no new
    package to `Cargo.lock`.
  - `sp-core` must carry `features = ["std"]`: the workspace entry is
    `default-features = false`, and `sp_core::testing::TaskExecutor` is `std`-gated. A
    `default-features = true` override would be silently ignored by cargo.

Nothing else is modified. The rest of `Cargo.toml` is upstream's as published (crates.io
already normalises `license` and `[lints]` to explicit values, so the rewrites the
`pvf/common` patch needs do not apply here).

### src/tests.rs

- Four fixtures committed to `dummy_hash()` (`Hash::zero()`) as the descriptor's
  `persisted_validation_data_hash` while passing
  `PersistedValidationData { max_pov_size: 1024, .. }` to
  `validate_candidate_exhaustive` — exactly the mismatch the backported check now
  rejects, so they returned `Invalid(BadParent)` instead of their expected results. The
  argument is now `validation_data.hash()` in `candidate_validation_ok_is_ok` (the v2
  branch; the v1 branch was already correct) and at the three descriptors built in
  `invalid_session_or_ump_signals`. The fixtures were only ever accepted because 28.0.0
  performs no such check; the candidates they describe are not well-formed. No other test
  needed changing.

- Added `candidate_validation_persisted_validation_data_mismatch_is_invalid`, a
  regression test for the backported check: a candidate that is valid in every other
  respect (it returns `Valid` with the check removed) must be rejected on all four
  `PvfExecKind` paths when the supplied `PersistedValidationData` does not hash to the
  descriptor's commitment. It asserts `InvalidCandidate::BadParent` specifically, pinning
  the deviation described below.

Because of these, `src/tests.rs` is no longer byte-identical to upstream and the re-sync
diff below now carries a `tests.rs` hunk.

### Deliberate deviations from upstream

Upstream 36.0.0 places the check in `pre_validate_candidate`, a function introduced by a
later refactor that does not exist in 28.0.0. Here:

- the check sits in `validate_candidate_exhaustive`, which is the single common path for
  `Backing`, `Approval` and `Dispute` in this version — the same coverage as upstream's
  call site in the `ValidateFromExhaustive` handler;
- it reports the existing `InvalidCandidate::BadParent`, because the new variant lives in
  `polkadot-node-primitives`, which is a dependency of almost every node crate; adding a
  variant to that non-`non_exhaustive` enum risks breaking exhaustive matches elsewhere.
  Approval-voting and the dispute coordinator treat every `ValidationResult::Invalid`
  identically, so behaviour is unchanged and only the log label differs.
- it returns `Ok(ValidationResult::Invalid(..))`, which matches upstream: upstream's
  `PreValidationError::Invalid` is internal control flow that its call site converts to
  exactly that, and only `PreValidationError::RuntimeError` becomes `ValidationFailed`.
  Returning `Err` here would mean "validity undeterminable", which callers treat as an
  internal error rather than a verdict.

Because of that refactor the upstream diff does **not** apply to 28.0.0; this is a
hand-port.

## Re-syncing on a polkadot bump

`patch.patch` is generated against the pristine crates.io source. The registry directory
carries files we do not vendor (`Cargo.lock`, `Cargo.toml.orig`, `.cargo_vcs_info.json`,
`.cargo-ok`); without excluding them the diff balloons from ~39 lines to ~10,700 of
spurious deletions.

```bash
cd patches/polkadot/node/core
R=~/.cargo/registry/src/index.crates.io-*/polkadot-node-core-candidate-validation-<ver>
diff -ruN '--exclude=*.patch' '--exclude=PATCH.md' '--exclude=rustfmt.toml' \
          '--exclude=Cargo.lock' '--exclude=Cargo.toml.orig' \
          '--exclude=.cargo-ok' '--exclude=.cargo_vcs_info.json' \
          "$R" candidate-validation > candidate-validation/patch.patch
```

## When this can be dropped

When zkVerify moves to a polkadot-sdk line that contains PR 13637 (candidate-validation
≥ 36.0.0), or if upstream backports it to stable2512.
