# Why and what is patched

This crate is patched because it contains the `WasmExecutor` definition used to
verifying the pvf. In our case `zkv-relay-execute-worker` implementation use this
crate to get the list of the `HostFunctions` needed to verify the pvf.

## Main change

The main change is to add `native::HLNativeHostFunctions` to the list of `HostFunctions`
returned by `WasmExecutor::new_with_config`: file `executor_interface.rs` line 111.

## Other changes

The other changes are all in the `Cargo.toml` file where we add `native` dependency,
add the version for the ones that are missed on the workspace and redefined the
package renaming for `tracing-gum` and `codec` where the `crate-name` macron is not
able to recognize the package name in a patch context.

## Patch file

`polkadot-stable2412-7.patch` a patch file against polkadot-stable2412-7 tag is provided.
