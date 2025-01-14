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

//! FAKE

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// FAKE.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> ismp_grandpa::WeightInfo for ZKVWeight<T> {
    fn add_state_machines(n: u32) -> Weight {
        Weight::from_parts(42_000, 2424)
            .saturating_add(Weight::from_parts(n as u64, 0))

    }

    fn remove_state_machines(_n: u32) -> Weight {
        Weight::from_parts(24_000, 4242)
    }
}
