# Common Benchmark Failure Patterns

Quick-reference for diagnosing the most frequent pallet benchmark failures in
Substrate runtimes.

## Missing `BenchmarkHelper` type

**Symptom**: compilation error or `BenchmarkError` at runtime referencing a
`BenchmarkHelper` associated type that has no concrete implementation.

**Cause**: The pallet's benchmark code requires a helper type (e.g.
`type BenchmarkHelper = SomeHelper`) in `impl pallet::Config for Runtime`, but
the runtime configuration either omits it or sets it to `()`.

**Fix**: Add a `#[cfg(feature = "runtime-benchmarks")]` gated implementation
that provides the helper, matching the pattern used in Polkadot/Kusama/Westend.

## Missing `SetupHandler` or benchmark setup trait

**Symptom**: benchmark panics during setup (before the measured call) or fails
with a trait-not-satisfied error.

**Cause**: Some pallets require a setup handler that prepares state (e.g.
creating accounts, funding them, registering validators). The runtime must wire
a concrete type that implements the setup trait.

**Fix**: Implement or wire the setup handler behind
`#[cfg(feature = "runtime-benchmarks")]`. Often the pallet provides a default
implementation that can be used directly.

## `WhitelistedCaller` not configured

**Symptom**: benchmark fails because the caller account doesn't have expected
privileges, or balance/permission checks fail during the benchmark extrinsic.

**Cause**: The benchmark framework uses `whitelisted_caller()` to create an
account with pre-funded balance and whitelisted storage access. If the runtime
doesn't configure this correctly (or requires additional setup like council
membership), the benchmark caller lacks the authority to execute the call.

**Fix**: Ensure the runtime's benchmark configuration whitelists the caller
account for the required privileges. Check how production runtimes handle this
for the same pallet.

## Feature-gated config mismatch

**Symptom**: benchmark compiles but fails at runtime with unexpected state or
missing storage.

**Cause**: The runtime uses `#[cfg(feature = "runtime-benchmarks")]` to swap in
benchmark-specific types, but the gating is incomplete — some types use the
production config while others use the benchmark config, leading to
inconsistencies.

**Fix**: Audit all `#[cfg(feature = "runtime-benchmarks")]` blocks in the
runtime config for the pallet and ensure they form a consistent set.

## New benchmark added upstream (not a regression)

**Symptom**: benchmark fails on the current branch but didn't exist on the
previous branch.

**Cause**: An upstream polkadot-sdk update introduced new benchmark extrinsics
that require runtime configuration not yet present in zkVerify.

**Fix**: Compare with the production runtimes (Step 3 of the workflow) to see
how they configure the new benchmark, and replicate the pattern.

## Incorrect `parameter_types!` values

**Symptom**: benchmark runs but produces absurdly high or zero weights, or
panics with arithmetic overflow.

**Cause**: `parameter_types!` values used by the pallet differ significantly
from production runtimes, causing benchmark iterations to hit edge cases.

**Fix**: Align `parameter_types!` values with Polkadot/Kusama/Westend defaults,
adjusting only where zkVerify's chain parameters genuinely differ.
