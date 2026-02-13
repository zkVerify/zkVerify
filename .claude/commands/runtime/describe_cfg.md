---
description: >
  Describe pallet configuration types from the zkv-runtime crate.
  Provide a pallet name and optionally specific type names to describe.
  If no types are given, all types from the pallet Config impl are listed
  and you are asked which ones to describe.
argument-hint: <pallet> [type1 type2 ...]
---

# Describe Runtime Configuration Types

You are analyzing pallet configuration types for the `zkv-runtime` crate.

## Input

- **Pallet**: `$0`
- **Types to describe**: `$ARGUMENTS`

If `$0` is empty or missing, ask the user for the pallet crate name (e.g. `pallet_staking`, `pallet_session`, `pallet_bags_list`).

## Step 1: Find the Config implementation

Search the `zkv-runtime` crate source (under `runtime/zkverify/src/`) for
`impl $0::Config for Runtime` (or the appropriate pattern — some pallets use
generic instances like `pallet_bags_list::Config<VoterBagsListInstance>`).

Read the full `impl` block to extract all associated type assignments.

## Step 2: Determine which types to describe

Parse the arguments after the pallet name. If specific type names were provided
(e.g. `MaxValidatorSet Filter Sort`), describe only those types.

If NO type names were provided beyond the pallet name, list ALL associated types
found in the `impl` block and ask the user which ones they want described.
Present them as a numbered list so the user can pick.

## Step 3: For each selected type, provide

For each type, research thoroughly using:
- The pallet source code in the local cargo registry (`~/.cargo/registry/src/`)
  to find the trait definition, doc comments, and trait bounds
- The polkadot-fellows/runtimes repository (https://github.com/polkadot-fellows/runtimes)
  for Polkadot and Kusama configurations

Produce the following for each type:

### 3a. Description

What this type is and what trait bound it has. Include the doc comment from
the pallet's `Config` trait if available.

### 3b. Purpose and application examples

How this type is used within the pallet. What behavior does it control?
What happens with different values? Reference specific pallet code paths
where this type is used (e.g. hooks, extrinsics, internal functions).

### 3c. Possible values

List the concrete types that can satisfy the trait bound. For each, explain
the behavioral difference:
- SDK-provided implementations (e.g. `()`, `ConstU32<N>`, `ConstBool<B>`,
  `Disabled`, `HoldConsideration`, etc.)
- Common patterns from production runtimes

### 3d. Polkadot and Kusama configuration

Look up how this type is configured in the Polkadot and Kusama relay chain
runtimes from https://github.com/polkadot-fellows/runtimes.

To determine the correct source version, check the currently deployed spec
version on each chain:
- Polkadot: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frpc-polkadot.luckyfriday.io#/explorer
- Kusama: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fkusama-rpc.n.dwellir.com#/explorer

Then find the corresponding tag/release in the polkadot-fellows/runtimes repo
and look at the exact configuration. Present a comparison table:

| Parameter | Polkadot | Kusama | zkVerify |
|-----------|----------|--------|----------|
| `TypeName` | value | value | value |

### 3e. zkVerify current value

Show the exact value configured in the zkv-runtime `impl` block, including
the constant/parameter definition if it references a `parameter_types!` value
or a `const`.

## Output format

For each type, use this structure:

```
#### `TypeName`

**Trait bound**: `TraitName<...>`

**Description**: ...

**Purpose**: ...

**Possible values**:
- `Value1` -- explanation
- `Value2` -- explanation

**Polkadot/Kusama comparison** (spec version XXXX):

| Parameter | Polkadot | Kusama | zkVerify |
|-----------|----------|--------|----------|
| `TypeName` | ... | ... | ... |

**zkVerify value**: `type TypeName = ...;` (defined as `CONST_NAME = value`)
```
