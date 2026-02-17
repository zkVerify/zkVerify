---
name: Cargo Audit Triage
description: >
  This skill should be used when the user asks to "run cargo audit",
  "triage cargo audit", "fix audit vulnerabilities", "update audit.toml",
  "check cargo audit ignores", "clean up audit ignore list",
  "review audit.toml", "remove stale audit ignores", or mentions
  resolving Rust security advisories or RUSTSEC identifiers. Provides a
  systematic workflow for
  analyzing each vulnerability, attempting updates, and writing motivated
  ignore entries when updates are not possible.
---

# Cargo Audit Triage

Systematic workflow for running `cargo audit`, analyzing each reported
vulnerability, attempting dependency updates, and documenting justified
ignores in `.cargo/audit.toml`.

## Workflow Overview

1. Read the existing `.cargo/audit.toml` ignore list for context
2. Run `cargo audit`
3. For each vulnerability, follow the resolution decision tree
4. For existing ignore entries, verify they still fire

## Step 1: Read Existing Ignore List

Read `.cargo/audit.toml` before running the audit. Study the existing
ignore entries and their motivations to understand the project's
conventions and writing style for ignore comments.

## Step 2: Run Cargo Audit

```bash
cargo audit
```

Collect all reported vulnerabilities (errors) and warnings. Separate them
into:
- **Vulnerabilities** (errors) — must be resolved or ignored with motivation
- **Warnings** (unmaintained, yanked) — informational, no action required

## Step 3: Resolve Each Vulnerability

For each vulnerability, follow this decision tree in order.

### 3a. Identify the Dependency Chain

```bash
cargo tree -i <crate>@<version> --depth 3
```

Determine whether the vulnerable crate is:
- A **direct dependency** (listed in a workspace or crate `Cargo.toml`)
- A **transitive dependency** (pulled in by another crate)

If `cargo tree` reports "nothing to print", try `--target all --edges all`.
If still nothing, the entry may be a **stale lockfile artifact** — verify
by searching `Cargo.lock` directly:

```bash
grep -B30 '"<crate> <version>"' Cargo.lock | grep 'name ='
```

### 3b. Attempt to Update

**Direct dependency with a patched version available:**
Update the version in `Cargo.toml` to the fixed version and run
`cargo check` to verify compatibility.

**Transitive dependency with a compatible patched version:**
Run `cargo update -p <crate>@<version>` to attempt an in-place update.
Verify with `cargo audit` afterward.

**Transitive dependency where the patched version is incompatible:**
Check if the **parent crate** has a newer version that uses the patched
dependency:

```bash
cargo search <parent-crate>
cargo info <parent-crate>@<latest-version>
```

If a compatible parent update exists, update it. If the parent is pinned
by an upstream framework (e.g., polkadot-sdk), updating may not be feasible.

### 3c. Analyze the Advisory (When Update Is Not Possible)

When no update path exists, perform a thorough analysis before ignoring:

1. **Read the advisory page** at the URL from the audit output
2. **Read the upstream security advisory** (usually linked from the
   rustsec page, often a GitHub Security Advisory / GHSA)
3. **Identify the trigger conditions** — what configuration, API usage,
   or code path activates the vulnerability
4. **Check the project's actual usage** — read the relevant source code
   to verify whether the trigger conditions are met

Common safe-to-ignore patterns:
- Vulnerability requires a feature/config that is not enabled
  (e.g., `wasm_threads(false)` means thread-related vulns don't apply)
- Vulnerability affects a platform not targeted
  (e.g., Windows-only issue on a Linux-only deployment)
- Vulnerable code path is never exercised
  (e.g., `IterMut` on a cache that is only read)
- Crate is a stale lockfile entry not actually used by the workspace

### 3d. Add Ignore Entry

Add the advisory to `.cargo/audit.toml` with a **motivated comment**
explaining:
- Where the dependency comes from (the dependency chain)
- A link to the advisory and/or upstream security advisory
- **Why it is safe to ignore** — the specific condition that makes the
  project unaffected

Follow the comment style already present in the file. See
`references/ignore-examples.md` for examples.

## Step 4: Verify Existing Ignore Entries

To clean up the ignore list:

1. Remove **all** entries from the `ignore` list
2. Run `cargo audit` with the empty list
3. Note which advisories still fire
4. Re-add only the entries that still fire, with refreshed comments
5. Remove entries that no longer fire (dependency was updated or removed)

## Ignore Comment Format

Each ignore entry should follow this pattern:

```toml
"RUSTSEC-YYYY-NNNN", # <crate> v<version> — short summary of the chain.
# Link to advisory and/or upstream security advisory.
# Explanation of why it is safe to ignore in this project's context.
```

Keep comments concise but complete. The goal is that a future reader can
understand the decision without re-investigating.

## Additional Resources

### Reference Files

- **`references/ignore-examples.md`** — Real-world examples of well-motivated
  ignore entries from a production Cargo workspace
