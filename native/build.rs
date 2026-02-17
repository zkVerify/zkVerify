// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This build script declares `substrate_runtime` as an expected cfg condition.
//!
//! The `#[runtime_interface]` macro from `sp-runtime-interface` generates code that uses
//! `#[cfg(substrate_runtime)]` to conditionally compile different implementations:
//! - When `substrate_runtime` is set: code runs inside WASM runtime (calls host functions)
//! - When NOT set: code runs natively (direct implementation)
//!
//! The `substrate_runtime` cfg is set externally by `substrate-wasm-builder` via
//! `RUSTFLAGS="--cfg substrate_runtime"` during WASM compilation. Since it's not
//! declared in Cargo.toml, Rust >= 1.80 warns about "unexpected cfg condition".
//!
//! This build script tells the compiler that `substrate_runtime` is a valid cfg,
//! suppressing the warning while preserving typo detection for other cfgs.
//!
//! References:
//! - <https://doc.rust-lang.org/rustc/check-cfg.html>
//! - <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html>
//! - <https://blog.rust-lang.org/2024/05/06/check-cfg.html>

fn main() {
    println!("cargo::rustc-check-cfg=cfg(substrate_runtime)");
}
