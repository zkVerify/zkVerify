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

//! TODO
//! 
//! 
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

pub struct ZKVWeight<T>(PhantomData<T>);

// For backwards compatibility and tests.
impl<T: frame_system::Config> pallet_aggregate::WeightInfo for ZKVWeight<T> {
    fn aggregate(n: u32, ) -> Weight {
        Weight::from_parts(53_000_000, 170000)
            .saturating_add(Weight::from_parts(28_966_000, 0).saturating_mul(n.into()))
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
            .saturating_add(Weight::from_parts(0, 80).saturating_mul(n.into()))
    }
    fn aggregate_on_invalid_domain() -> Weight {
        Weight::from_parts(7_000_000, 170000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
    }
    fn aggregate_on_invalid_id() -> Weight {
        Weight::from_parts(9_000_000, 179999)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
    }
    fn register_domain() -> Weight {
        Weight::from_parts(50_000_000, 4000)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
    fn unregister_domain() -> Weight {
        Weight::from_parts(50_000_764_000, 177888)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
}