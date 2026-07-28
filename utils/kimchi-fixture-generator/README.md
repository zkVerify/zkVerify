# Kimchi fixture generator

This standalone development tool generates deterministic Kimchi proofs,
verifier indexes, and public inputs for the tests and benchmarks in
`verifiers/kimchi`.

The generator enables the upstream Kimchi prover and therefore has a different
dependency and feature profile from the verifier used by the runtime. It has
its own Cargo workspace and lockfile and is excluded from the repository's root
workspace. Building it does not add prover or `std` features to the runtime
dependency graph.

## Usage

Run the generator from the repository root:

```console
cargo run --release \
  --manifest-path utils/kimchi-fixture-generator/Cargo.toml -- \
  [domain-log2] [max-poly-size-log2] [public-input-count] [shape]
```

The arguments and their defaults are:

| Argument | Default | Accepted values |
| --- | ---: | --- |
| `domain-log2` | `17` | `12` through `20` |
| `max-poly-size-log2` | `16` | `2` through `16` |
| `public-input-count` | `64` | Any value that fits in the selected circuit |
| `shape` | `lookup-runtime` | `lookup-runtime`, `all-features`, or `simple` |

The evaluation domain must not require more than 16 polynomial-commitment
chunks for the selected maximum polynomial size.

For example, this command generates a lookup-and-runtime-table fixture for a
`2^17` domain, a `2^16` maximum polynomial size, and 64 public inputs:

```console
cargo run --release \
  --manifest-path utils/kimchi-fixture-generator/Cargo.toml -- \
  17 16 64 lookup-runtime
```

## Output

The generator writes a shape-specific directory under
`verifiers/kimchi/src/resources/`. Each generated fixture contains:

- `proof.bin`: the bincode-encoded Kimchi proof;
- `verifier_index.bin`: the bincode-encoded verifier index;
- `pubs.bin`: the canonically serialized public inputs.

The random-number generator is seeded from `domain-log2`, so the same arguments
and pinned dependencies produce deterministic fixtures. Running the same
command again overwrites the corresponding fixture files.

Generated fixtures can be large and proof creation can take several minutes for
the largest domains. After regenerating a fixture, run the Kimchi verifier tests
and inspect the binary fixture changes before committing them.
