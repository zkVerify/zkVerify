---
name: Investigate Benchmark Issue
description: >
  This skill should be used when the user asks to "investigate benchmark",
  "fix benchmark", "debug benchmark", "benchmark fails", "benchmark issue",
  "benchmark error", "BenchmarkError", "run benchmark", "benchmark regression",
  "benchmark helper missing", "benchmark weight", "weight benchmark",
  "pallet benchmark not working", or mentions a pallet benchmark that needs
  investigation. Provides a systematic workflow for running the benchmark,
  comparing against the main branch and Polkadot/Kusama/Westend runtime
  configurations, diagnosing root causes, and applying fixes.
argument-hint: [pallet] [extrinsic]
version: 1.0.0
---

# Investigate Benchmark Issue

Systematic workflow for investigating failing or new pallet benchmarks.
Covers running the benchmark, comparing against a reference branch and
production runtimes (Polkadot, Kusama, Westend), diagnosing the root cause,
and applying a fix.

## Input

- **Pallet**: the first positional argument from `$ARGUMENTS` (e.g. `pallet_staking`)
- **Extrinsic**: the second positional argument from `$ARGUMENTS` (e.g. `force_unstake`), or `"*"` for all extrinsics in the pallet

If `$ARGUMENTS` is empty or incomplete, ask the user for the pallet and
extrinsic to investigate.

## Build profile

By default use **debug** builds — faster compilation; benchmarks run with
minimal iterations so execution speed matters less than compile time. If the
user explicitly requests release, add `--release` and replace `debug` with
`release` in the paths below.

```bash
# Compile (add --release for release profile)
cargo build -p zkv-relay --features runtime-benchmarks

# Run benchmark for specific pallet + extrinsic
./target/debug/zkv-relay benchmark pallet \
  --runtime ./target/debug/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm \
  --pallet <pallet> --extrinsic "<extrinsic>" -s 2 -r 1

# Run all extrinsics for a pallet (use --extrinsic "*")
# Run ALL benchmarks (use --pallet "*" --extrinsic "*")
```

## Constraints

- **Only modify code in the `runtime/` folder** (primarily
  `runtime/zkverify/src/`).
- Changes should **mostly be gated by `#[cfg(feature = "runtime-benchmarks")]`**
  unless the fix genuinely requires a non-benchmark configuration change.
- If you need more information or are unsure about anything, **ask the user**.
- Do not modify pallet source code, benchmark definitions, or anything outside
  `runtime/`.

## Process

Follow these steps in order. Use **planning mode** before making any code
changes.

### Step 1: Run the benchmark

Build the binary (debug by default) and run the benchmark for the specified
pallet/extrinsic using the commands above. Record the output, paying attention
to:

- Whether it completes successfully or errors out
- The error message if it fails
- The extrinsic name and any weight output if it succeeds

### Step 2: Check the reference branch

Determine the **main branch** to compare against. If the conversation context
makes the main branch obvious (e.g. the git status shows it, or `CLAUDE.md`
specifies it), use that. Otherwise, **ask the user** which branch to compare
against.

On the reference branch, check whether this benchmark existed:

```bash
# Search for the benchmark function in the reference branch
git grep "fn <benchmark_name>" <main-branch>
```

Use `git log`, `git show`, or `git diff <main-branch>` to examine the
runtime configuration for the relevant pallet on the reference branch.
Key things to compare:

- Does the benchmark exist on the reference branch?
- If yes, does it pass there? (If confirmed by the user or verifiable by checking out and running the benchmark on that branch)
- What runtime configuration (`impl pallet::Config for Runtime`) differs
  between the branches?

### Step 3: Compare with Polkadot, Kusama, and Westend runtimes

Understand how production runtimes configure the pallet that the benchmark
belongs to. This helps identify the correct configuration pattern.

**Ask the user** to provide the `impl pallet::Config for Runtime` blocks from
the production runtimes (Polkadot, Kusama, Westend). Give them these pointers
so they know where to look:

- **Polkadot / Kusama**: the `polkadot-fellows/runtimes` repository on GitHub,
  at the tag matching the currently deployed spec version
- **Westend**: the `paritytech/polkadot-sdk` repository, on the branch this
  project tracks (check `Cargo.toml` dependencies or existing patches for the
  polkadot-sdk version)

If the user has already provided this information or it is available locally
(e.g. in vendored patches), use that instead of asking.

#### What to look for

For each runtime, extract:
- The `impl pallet::Config for Runtime` block
- Any `parameter_types!` values used by that config
- Any helper types, mock implementations, or benchmark-specific config
  (gated behind `#[cfg(feature = "runtime-benchmarks")]`)

Present a comparison table:

| Configuration item | Polkadot | Kusama | Westend | zkVerify |
|--------------------|----------|--------|---------|----------|
| `TypeName`         | value    | value  | value   | value    |

### Step 4: Compare with our implementation

Read the zkVerify runtime configuration for the pallet under
`runtime/zkverify/src/`. Compare it against the production runtimes from
Step 3, focusing on:

- Missing benchmark helper types (e.g. `BenchmarkHelper`, `SetupHandler`)
- Missing or incorrect `#[cfg(feature = "runtime-benchmarks")]` gated config
- Different type assignments that could cause benchmark failures
- Missing trait implementations required by benchmark code

### Step 5: Plan and fix

Enter **planning mode** to propose a fix. Present:

1. **Root cause**: why the benchmark fails
2. **What production runtimes do**: the pattern from Polkadot/Kusama/Westend
3. **Proposed fix**: the specific changes to make

Wait for user approval before implementing.

### Step 6: Verify the fix

After applying the fix, rebuild and re-run the benchmark using the same
commands from Step 1. Confirm:

- The benchmark now passes
- The output looks reasonable (no absurdly high or zero weights)

If the benchmark still fails, go back to Step 4 and iterate.

### Step 7: Present results

Provide a summary:

- **Pallet / extrinsic**: what was investigated
- **Issue**: what was wrong (new benchmark, regression, missing config, etc.)
- **Root cause**: why it failed
- **Fix**: what was changed and why
- **Comparison**: how the fix aligns with Polkadot/Kusama/Westend
