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

//! Manually generated weights for `crate::parachains::on_demand`
//!
//! The weights are all intentionally set to MAX to block these extrinsics.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weights for `crate::parachains::on_demand` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> crate::parachains::on_demand::WeightInfo for ZKVWeight<T> {
    /// Disabled for the time being
    fn place_order_keep_alive(_s: u32, ) -> Weight {
        Weight::MAX
    }
    /// Disabled for the time being
    fn place_order_allow_death(_s: u32, ) -> Weight {
        Weight::MAX
    }
}
