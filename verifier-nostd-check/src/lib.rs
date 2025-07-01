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

#![cfg_attr(not(feature = "std"), no_std)]

#[unsafe(no_mangle)]
pub fn ultraplonk() {
    ultraplonk::verify();
}

#[unsafe(no_mangle)]
pub fn risc0() {
    risc0::verify();
}

#[cfg(not(feature = "dont-link-maybe-fail"))]
#[unsafe(no_mangle)]
pub fn maybe_fail() {
    let _ = maybe_fail::TRUE;
    maybe_fail::verify();
}

// ==========================================================
// Don't change follow code: Is just for compilation
// ==========================================================

extern crate alloc;

use alloc::borrow::Cow;
use sp_version::{create_apis_vec, RuntimeVersion};

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("fake-spec"),
    impl_name: Cow::Borrowed("fake-node"),
    authoring_version: 1,
    spec_version: 1_000,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

pub const RUNTIME_API_VERSIONS: sp_version::ApisVec = create_apis_vec! { [] };

// ==========================================================
// END OF TEMPLATE CODE
// ==========================================================
