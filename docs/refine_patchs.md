# Overview

We need to refine the patches contained in `patches/` folder. We would minimize the difference with the upstream version.

## Branch

The upstream repo is polkadot-sdk and the branch is `stable2512`

## Outputs

* In every patch folder we need 
  * `PATCH.md` with the reference to the original crate (repo and branch/tag/commit), why we need to patch it 
  and the changes we made. You can see an example in `patches/polkadot/node/core/pvf/common/patch.md`
  * `patch.patch` with the patch file.
  * the `rustfmt.toml` file to avoid formatting.
* The changes related to the upstream version should be just the necessary ones to make the patch work.

## Guidelines

* If you have any question or doubt ask a feedback.
* Work with a patch one at a time, do not start to work on another patch until the previous one is committed.
* When you are ready run test in the crate and then compile runtime and node. Node with 
`none/runtime-benchmark/try-runtime/fast-runtime` features one at the time.
* Run node tests.