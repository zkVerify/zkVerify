# Ignore Entry Examples

Real-world examples of well-motivated `.cargo/audit.toml` ignore entries,
grouped by the reason pattern they demonstrate.

## Pattern: Stale Lockfile Entry

The crate exists in `Cargo.lock` but nothing in the workspace depends on
it. Cargo resolves optional dependencies into the lockfile even when the
feature is not enabled.

```toml
"RUSTSEC-2025-0009", # ring 0.16.20 — stale lockfile entry (libp2p-quic -> libp2p-tls -> rcgen -> ring 0.16).
# Nothing in the workspace actually depends on this chain (`cargo tree` confirms it).
# It remains in Cargo.lock because Cargo always resolves optional dependencies in the
# lockfile (libp2p's optional `quic` feature pulls it in). Cannot be removed.
```

**Investigation method:** Run `cargo tree -i <crate>@<version> --target all --edges all`.
If "nothing to print", confirm with `grep -B30 '"<crate> <version>"' Cargo.lock | grep 'name ='`
to trace the lockfile chain.

## Pattern: Feature / Config Not Enabled

The vulnerability only triggers under a specific configuration that is
not used in the project.

```toml
"RUSTSEC-2025-0118", # coming from indirect dependency wasmtime v35.0.0 via sc-executor-wasmtime v0.43.0.
# From https://rustsec.org/advisories/RUSTSEC-2025-0118 ->
# https://github.com/bytecodealliance/wasmtime/security/advisories/GHSA-hc7m-r6v8-hg9q
# "Embeddings which do not use the wasm threads proposal nor created shared memories
# nor actually share shared memories across threads are unaffected."
# Substrate explicitly sets config.wasm_threads(false) in sc-executor-wasmtime.
```

```toml
"RUSTSEC-2026-0006", # coming from indirect dependency wasmtime v35.0.0 via sc-executor-wasmtime v0.43.0.
# From https://rustsec.org/advisories/RUSTSEC-2026-0006 ->
# https://github.com/bytecodealliance/wasmtime/security/advisories/GHSA-vc8c-j3xm-xj73
# The vulnerability (segfault or out-of-sandbox load with f64.copysign on x86-64) only triggers
# when signals-based-traps is DISABLED in the wasmtime Config.
# Substrate's sc-executor-wasmtime uses wasmtime::Config::new() which has signals_based_traps
# enabled by default, and never disables it. Therefore this code path is never exercised.
```

**Investigation method:** Read the advisory and upstream GHSA to find the
trigger conditions. Then read the project's source code (or the
intermediate crate's source) to verify the condition is not met. Quote
the relevant config call or code path in the comment.

## Pattern: Vulnerable Code Path Not Exercised

The crate has a vulnerability in a specific API or code path, but the
project (or its dependencies) never calls that API.

```toml
"RUSTSEC-2026-0002", # lru v0.12.5 unsound IterMut — used by libp2p-identify v0.45.0 and libp2p-swarm v0.45.1
# (polkadot-sdk transitive dependency via libp2p v0.54.1).
# We checked the libp2p-identify code at v0.45.0 and libp2p-swarm at v0.45.1:
# no mutable iterator (IterMut) is used on lru::LruCache, so the unsound code path
# is never exercised.
```

**Investigation method:** Identify which crates use the vulnerable
dependency (`cargo tree -i <crate>@<version>`). Then inspect the source
of each consumer to check whether the vulnerable API is called. Mention
the specific version checked and the result.

## Pattern: Upstream Issue With No Available Fix

The vulnerability exists in a transitive dependency and no version of
the parent crate avoids it.

```toml
"RUSTSEC-2025-0055", # coming from indirect dependency tracing-subscriber v0.2.25
# via ark-relations v0.4.0 (and even v0.5.1 still depends on it).
# The std feature of ark-relations unconditionally enables tracing-subscriber 0.2.x.
# This is an upstream arkworks issue; no patched version of ark-relations avoids it.
# On the other end the tracing-subscriber v0.3.20 breaks console log output formatting
# as reported in https://github.com/tokio-rs/tracing/issues/3369
```

**Investigation method:** Check the latest version of the parent crate
to see if it still depends on the vulnerable version. Document this in
the comment so future reviewers know updating the parent won't help.

## Pattern: Platform Not Affected

The vulnerability only affects a platform the project does not target.

```toml
"RUSTSEC-2024-0438", # Just affects Windows where devices are not fully sandboxed.
```

**Investigation method:** Read the advisory for platform-specific
conditions. Document which platform is affected and confirm the project
does not target it.
