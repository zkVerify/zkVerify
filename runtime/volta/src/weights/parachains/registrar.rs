// Copyright 2025, Horizen Labs, Inc.
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

/// ZKV weights for the paras_registrar crate. Notice, differently from most of the other weights,
/// these have been created manually.
/// These weights are deliberately too big to fit in a block, so that these extrinsics can never be
/// used, actually. This is the intended behavior, where we want paras_registrar to be included in
/// our runtime for the sake of data visualization, but at the same time we do not want regular
/// users to be able to register new parachains.
use core::marker::PhantomData;
use frame_support::weights::Weight;

/// Weights for `crate::parachains::configuration` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> crate::parachains::paras_registrar::WeightInfo for ZKVWeight<T> {
    fn reserve() -> Weight {
        Weight::MAX
    }

    fn register() -> Weight {
        Weight::MAX
    }

    fn force_register() -> Weight {
        Weight::MAX
    }

    fn deregister() -> Weight {
        Weight::MAX
    }

    fn swap() -> Weight {
        Weight::MAX
    }

    fn schedule_code_upgrade(_b: u32) -> Weight {
        Weight::MAX
    }

    fn set_current_head(_b: u32) -> Weight {
        Weight::MAX
    }
}
