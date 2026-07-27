# Fast check if a dependency could be use in the runtime

This project is a small template that contains some examples of how to check if your code can be used in the runtime.
There are some cases where you cannot compile a crate as bare metal (`thumbv7em-none-eabi` is the most used target to
check it) even if you can compile it in the runtime. This is due because when you compile every single dependency the
target `thumbv7em-none-eabi` dosn't provide any interface for the standard library like `wasm32-unknown-unknown` does;
on the other side `wasm32-unknown-unknown` provide a some base implementation for the standard library that will
leverage on the VM but that implementations can be in conflict with the ones provided by substrate runtime.

So, this project will compile the library like a substrate runtime to spot out all possible issues.

## Before You Start

If you would try it or check your dependency you should add this member to the workspace: in `Cargo.toml` at the root
of the project, in `workspace` section add `"verifier-nostd-check"` in members.

## Usage

In `src/lib.rs` you should add your

```rust
#[unsafe(no_mangle)]
pub fn my_own_check() {
    my_dependecy::something();
}
```

- `#[unsafe(no_mangle)]` is important because make your entry point visible from the vm and prevent the compiler
  to optimize it out.
- Of course `my_dependency` should be a `no-std` crate and, you should call some methods from it, put it in the crate
  dependencies is not enough, you should use it in the code. Take care that every call is enough, also a simple static
  value or a constant.

Now if `my_dependecy` use some transitive dependency that is not `no-std` you will get an error like this:

```
 error[E0152]: duplicate lang item in crate `std` (which `maybe_fail` depends on): `panic_impl`
    |
    = note: the lang item is first defined in crate `sp_io` (which `sp_application_crypto` depends on)
    = note: first definition in `sp_io` loaded from /home/mdamico/devel/zkVerify/target/debug/wbuild/checker-template/target/wasm32-unknown-unknown/release/deps/libsp_io-c8d8384c9929e2ff.rmeta
    = note: second definition in `std` loaded from /home/mdamico/devel/zkVerify/target/debug/wbuild/checker-template/target/wasm32-unknown-unknown/release/deps/libstd-1f46d38d5f38c7c6.rmeta

```

## Examples

In the template you can find three examples:

1. `ultraplonk`: load a proof and verify it.
2. `risc0`: just load the context
3. `maybe-fail`: a simple crate that include a dependency `bit-vec` with `std` feature enabled, till the this crate
   remains a dependency but not used everything compile fine, but if you gate in the `bit-vec` usage by enable the
   feature `fail` it will fail.

```
mdamico@miklap:~/devel/zkVerify$ cargo build -p checker-template # This WORKS
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.35s
mdamico@miklap:~/devel/zkVerify$ cargo build -p checker-template --features fail # This FAILS
   Compiling maybe-fail v0.1.0 (/home/mdamico/devel/zkVerify/verifier-nostd-check/maybe-fail)
   Compiling checker-template v0.1.0 (/home/mdamico/devel/zkVerify/verifier-nostd-check)
error: failed to run custom build command for `checker-template v0.1.0 (/home/mdamico/devel/zkVerify/verifier-nostd-check)`

Caused by:
  process didn't exit successfully: `/home/mdamico/devel/zkVerify/target/debug/build/checker-template-328fa844c07b1f50/build-script-build` (exit status: 1)
  --- stdout
  Information that should be included in a bug report.
  Executing build command: env -u CARGO_ENCODED_RUSTFLAGS -u RUSTC CARGO_MAKEFLAGS="-j --jobserver-fds=8,9 --jobserver-auth=8,9" CARGO_TARGET_DIR="/home/mdamico/devel/zkVerify/target/debug/wbuild/checker-template/target" RUSTC_BOOTSTRAP="1" RUSTFLAGS="-C target-cpu=mvp -C target-feature=-sign-ext -C link-arg=--export-table -Clink-arg=--export=__heap_base -C link-arg=--import-memory  --cfg substrate_runtime " SKIP_WASM_BUILD="" "/home/mdamico/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" "rustc" "--target=wasm32-unknown-unknown" "--manifest-path=/home/mdamico/devel/zkVerify/target/debug/wbuild/checker-template/Cargo.toml" "--color=always" "--profile" "release" "-Z" "build-std"
  Using rustc version: rustc 1.88.0 (6b00bc388 2025-06-23)


  --- stderr
  warning: Patch `cumulus-relay-chain-inprocess-interface v0.22.0 (/home/mdamico/devel/zkVerify/patches/cumulus/client/relay-chain-inprocess-interface)` was not used in the crate graph.
  Patch `cumulus-relay-chain-minimal-node v0.22.1 (/home/mdamico/devel/zkVerify/patches/cumulus/client/relay-chain-minimal-node)` was not used in the crate graph.
  Patch `polkadot-omni-node-lib v0.4.2 (/home/mdamico/devel/zkVerify/patches/cumulus/polkadot-omni-node/lib)` was not used in the crate graph.
  Check that the patched package version and available features are compatible
  with the dependency requirements. If the patch has a different version from
  what is locked in the Cargo.lock file, run `cargo update` to use the new
  version. This may also occur with an optional dependency that is not enabled.
     Compiling maybe-fail v0.1.0 (/home/mdamico/devel/zkVerify/verifier-nostd-check/maybe-fail)
     Compiling checker-template v0.1.0 (/home/mdamico/devel/zkVerify/verifier-nostd-check)
  error[E0152]: duplicate lang item in crate `std` (which `bit_vec` depends on): `panic_impl`
    |
    = note: the lang item is first defined in crate `sp_io` (which `sp_application_crypto` depends on)
    = note: first definition in `sp_io` loaded from /home/mdamico/devel/zkVerify/target/debug/wbuild/checker-template/target/wasm32-unknown-unknown/release/deps/libsp_io-c8d8384c9929e2ff.rmeta
    = note: second definition in `std` loaded from /home/mdamico/devel/zkVerify/target/debug/wbuild/checker-template/target/wasm32-unknown-unknown/release/deps/libstd-1f46d38d5f38c7c6.rmeta

  For more information about this error, try `rustc --explain E0152`.
  error: could not compile `checker-template` (lib) due to 1 previous error
```