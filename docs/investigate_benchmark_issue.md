# Overview

In this branch we upgraded the code to `stable2512` polkadot branch. Some benchmarks do 
not work. Some of them are new in this version and other could be regressions leaded by wrong 
runtime configuration.

## Commands

* Compile: `cargo build -p zkv-relay --features runtime-benchmarks`
* Given a <pallet> (could be a list) and <extrinsic> (could be a list) : `./target/release/zkv-relay benchmark pallet --runtime ./target/release/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm --pallet <pallet> --extrinsic "<extrinsic>" -s 2 -r 1`
* Given a <pallet> (could be a list) run benchmark for all extrinsics : `./target/release/zkv-relay benchmark pallet --runtime ./target/release/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm --pallet <pallet> --extrinsic "*" -s 2 -r 1`
* Run All benchmarks: `./target/release/zkv-relay benchmark pallet --runtime ./target/release/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm --pallet "*" --extrinsic "*" -s 2 -r 1`

## Process

Ask me for pallet and an extrinsic

* Run the benchmark and check the result
* Use Planning mode
* Understand if that benchmark was still present in the previous version (check it against main branch)
* Understand how Polkadot, Kusama and Westend runtimes implements the configuration for this benchmarks
* Compare with our implementation
* Fix the issue
* Provide issue description and solving path

## Take care of

* Only code in `runtime` folder should be changed and mostly should be gated by `runtime-benchmarks` feature.
* The previous code was based on `stable2412` branch.
* If you need more information, ask me
* If you are not sure, ask me