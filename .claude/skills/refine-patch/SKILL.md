---
name: Refine Patch
description: >
  Refine a vendored patch in the patches/ directory to minimize differences
  with the upstream crate. Use when the user asks to "refine a patch",
  "minimize patch diff", "update a patch", "clean up a patch", "work on
  patches", "align patch with upstream", or references refining patches in
  patches/. Guides the full workflow: cloning upstream, comparing, reducing
  diff, producing PATCH.md + patch.patch + rustfmt.toml, testing, and
  compiling.
argument-hint: <patch-path>
---

# Refine Patch

Systematic workflow for refining a vendored patch in the `patches/` directory
to minimize the difference with the upstream version while preserving all
necessary zkVerify customizations.

## Context

The `patches/` directory contains vendored, patched versions of upstream
crates. Each patch directory mirrors the upstream crate structure and contains:
- Source files with zkVerify-specific modifications
- `PATCH.md` documenting why the patch exists and what changed
- `patch.patch` unified diff against upstream
- `rustfmt.toml` with `disable_all_formatting = true`

## Input

- **Patch path**: `$ARGUMENTS`

If `$ARGUMENTS` is empty, ask the user which patch to refine. List the
existing patches under `patches/` for them to choose from.

## Step 0: Determine the upstream repository and branch

By default, assume the upstream repository is `polkadot-sdk`
(https://github.com/nicetip/polkadot-sdk.git).

Check the existing `PATCH.md` in the patch directory (if it exists) to
determine the upstream branch. If no `PATCH.md` exists or the branch is
unclear, ask the user for the upstream branch.

If the user has not specified a branch and it cannot be inferred, default to
`stable2512` but confirm with the user first.

## Step 1: Prepare the upstream reference

Clone or fetch the upstream source to a temp directory so you can compare:

```bash
# Create temp dir for upstream
UPSTREAM_TMP=$(mktemp -d)

# Sparse checkout only the relevant crate path
git clone --depth 1 --branch <branch> --filter=blob:none --sparse \
  https://github.com/nicetip/polkadot-sdk.git "$UPSTREAM_TMP/polkadot-sdk"
cd "$UPSTREAM_TMP/polkadot-sdk"
git sparse-checkout set <upstream-crate-path>
```

The `<upstream-crate-path>` maps from the patch directory path. The mapping is:
- `patches/polkadot/...` -> `polkadot/...` in polkadot-sdk
- `patches/cumulus/...` -> `cumulus/...` in polkadot-sdk
- `patches/substrate/...` -> `substrate/...` in polkadot-sdk

## Step 2: Analyze the current patch

Read every file in the patch directory. For each source file (`.rs`,
`Cargo.toml`), compare it side-by-side with the upstream version.

Identify each difference and classify it:

1. **Necessary zkVerify customization** -- changes required for zkVerify
   functionality (e.g., adding native host functions, replacing polkadot-service
   with zkv-service)
2. **Unnecessary formatting/style diff** -- whitespace, import ordering, or
   formatting changes that don't affect functionality
3. **Stale changes** -- modifications that were needed for a previous upstream
   version but are no longer necessary
4. **Missing upstream updates** -- upstream changes that should be incorporated

## Step 3: Minimize the diff

For each file, start from the upstream version and apply ONLY the necessary
zkVerify customizations. This ensures the minimal diff.

**Critical rule: never hand-edit `patch.patch` directly.** Always modify the
source files and regenerate the patch using `diff` (Step 5). Hand-editing
patch files leads to malformed hunk headers, incorrect line counts, and
mismatched context lines that cause application failures.

### Cargo.toml changes

Patches to `Cargo.toml` typically require these standard modifications:

1. **License**: Change `license.workspace = true` to an explicit license string
   (e.g., `license = "GPL-3.0-only"`) because the zkVerify workspace
   `[workspace.package]` does not define a `license` field.

2. **Lints**: Replace `[lints] workspace = true` with an explicit
   `[lints.clippy]` section. Start with the workspace lints:
   ```toml
   [lints.clippy]
   # Workspace lints (copied from [workspace.lints.clippy])
   all = { level = "allow", priority = 0 }
   correctness = { level = "deny", priority = 1 }
   suspicious = { level = "deny", priority = 1 }
   complexity = { level = "deny", priority = 1 }
   style = { level = "warn", priority = 1 }
   unnecessary_cast = { level = "allow", priority = 2 }
   useless_conversion = { level = "allow", priority = 2 }
   zero-prefixed-literal = { level = "allow", priority = 2 }
   ```
   Then add upstream code suppressions as needed (compile the crate and fix
   any clippy warnings by adding `{ level = "allow", priority = 2 }` entries
   with a comment `# Upstream code suppressions`).

3. **Dependencies**: Add/replace only the dependencies that differ from
   upstream. Mark each with a `# zkVerify customization: <reason>` comment.

### Source file changes

- Add inline comments `// zkVerify customization: <description>` next to each
  change to make the patch self-documenting
- Preserve upstream formatting exactly -- do not reformat upstream code
- Only modify the lines that are strictly necessary

## Step 4: Create the `rustfmt.toml`

Ensure the patch directory contains a `rustfmt.toml` with:

```toml
disable_all_formatting = true
```

This prevents `cargo fmt` from reformatting the patched upstream code.

## Step 5: Generate the `patch.patch` file

Generate a unified diff between the upstream source and the patched version:

```bash
diff -ruN \
  '--exclude=*.patch' \
  '--exclude=PATCH.md' \
  '--exclude=rustfmt.toml' \
  <upstream-crate-dir> <patched-crate-dir> > patch.patch
```

You may also need to exclude files like `README.md` or `*.orig` if present.
Check existing patches for the exclude patterns used.

Review the generated patch to verify it contains ONLY the necessary changes.

### 5a. Validate the patch with a round-trip test

After generating the patch, verify it reproduces the patched source exactly:

```bash
# Create a temp working area
VERIFY_TMP=$(mktemp -d)

# Copy patched source and normalize the patch paths
cp -r <patched-crate-dir>/src "$VERIFY_TMP/patched/src"
cp <patched-crate-dir>/Cargo.toml "$VERIFY_TMP/patched/" 2>/dev/null
sed 's|--- <upstream-path>|--- upstream/|g; s|+++ <patched-path>|+++ patched/|g' \
  <patched-crate-dir>/patch.patch > "$VERIFY_TMP/normalized.patch"

# Reverse-apply to recover upstream, then forward-apply to reconstruct
cp -r "$VERIFY_TMP/patched" "$VERIFY_TMP/upstream"
cd "$VERIFY_TMP/upstream" && patch -R -p1 < ../normalized.patch
cp -r "$VERIFY_TMP/upstream" "$VERIFY_TMP/forward"
cd "$VERIFY_TMP/forward" && patch -p1 < ../normalized.patch

# Compare -- must be identical and apply without fuzz or offsets
diff -r "$VERIFY_TMP/patched" "$VERIFY_TMP/forward"
```

If the round-trip fails or `patch` reports fuzz/offsets, **do not fix the patch
file by hand**. Instead, fix the source files and regenerate the patch (go back
to Step 5).

## Step 6: Write/update the `PATCH.md`

Create or update the `PATCH.md` following this structure:

```markdown
# Patch: <crate-name>

**Upstream crate**: `<crate-name>` from `polkadot-sdk` branch `<branch>`

## Why patched

<Brief explanation of why this crate needs to be patched for zkVerify>

## Changes from upstream

### Cargo.toml
- <List each Cargo.toml change>

### src/<file>.rs
- <List each source change, one bullet per logical modification>
```

Use the existing PATCH.md files as style reference (see
`references/patch-md-examples.md`).

## Step 7: Test

Run tests and compilation in this order, stopping at the first failure:

### 7a. Crate-level tests

```bash
SKIP_WASM_BUILD=1 cargo test --release --lib -p <crate-name> --all-features
```

### 7b. Compile the runtime

```bash
cargo build -p zkv-runtime --release
```

### 7c. Compile the node with each feature set (one at a time)

```bash
cargo build -p zkv-relay --release
cargo build -p zkv-relay --release --features runtime-benchmarks
cargo build -p zkv-relay --release --features try-runtime
cargo build -p zkv-relay --release --features fast-runtime
```

### 7d. Run node tests

```bash
cargo test --release --lib -p zkv-relay --all-features
cargo test --release --lib -p zkv-service --all-features
```

If any step fails, diagnose and fix before proceeding. The fix may require
adjusting the patch.

## Step 8: Present results

Once all tests pass, present a summary:
- Files changed and why
- Diff size before vs after refinement (if applicable)
- Any decisions made about what to keep vs remove

**Work on one patch at a time.** Do not start refining the next patch until the
current one is committed.

## Additional Resources

### Reference Files

- **`references/patch-md-examples.md`** -- Examples of well-structured PATCH.md
  files from existing patches
