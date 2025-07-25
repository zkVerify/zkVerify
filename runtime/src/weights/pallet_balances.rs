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

//! Autogenerated weights for `pallet_balances`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.2.0
//! DATE: 2025-06-05, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `003c48a2773a`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --runtime
// /app/zkv_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --pallet
// pallet-balances
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --heap-pages=4096
// --header
// /data/benchmark/HEADER-APACHE2
// --output
// /data/benchmark/runtime/src/weights/pallet_balances.rs
// --template
// /data/benchmark/relay-node/benchmarks/zkv-deploy-weight-template.hbs
// --base-path=/tmp/tmp.LQE1fxMBsI

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;
use crate::parachains;

/// Weights for `pallet_balances` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_balances::WeightInfo for ZKVWeight<T> {
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn transfer_allow_death() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `3593`
        // Minimum execution time: 43_531_000 picoseconds.
        Weight::from_parts(44_894_000, 3593)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn transfer_keep_alive() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `3593`
        // Minimum execution time: 34_965_000 picoseconds.
        Weight::from_parts(35_556_000, 3593)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn force_set_balance_creating() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `174`
        //  Estimated: `3593`
        // Minimum execution time: 15_258_000 picoseconds.
        Weight::from_parts(15_950_000, 3593)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn force_set_balance_killing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `174`
        //  Estimated: `3593`
        // Minimum execution time: 21_329_000 picoseconds.
        Weight::from_parts(22_021_000, 3593)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `System::Account` (r:2 w:2)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn force_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `103`
        //  Estimated: `6196`
        // Minimum execution time: 48_491_000 picoseconds.
        Weight::from_parts(49_792_000, 6196)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn transfer_all() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `3593`
        // Minimum execution time: 43_681_000 picoseconds.
        Weight::from_parts(44_853_000, 3593)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn force_unreserve() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `174`
        //  Estimated: `3593`
        // Minimum execution time: 18_234_000 picoseconds.
        Weight::from_parts(18_846_000, 3593)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `System::Account` (r:999 w:999)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// The range of component `u` is `[1, 1000]`.
    fn upgrade_accounts(u: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + u * (136 ±0)`
        //  Estimated: `990 + u * (2603 ±0)`
        // Minimum execution time: 17_111_000 picoseconds.
        Weight::from_parts(17_713_000, 990)
            // Standard Error: 10_838
            .saturating_add(Weight::from_parts(14_038_964, 0).saturating_mul(u.into()))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(u.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(u.into())))
            .saturating_add(Weight::from_parts(0, 2603).saturating_mul(u.into()))
    }
    fn force_adjust_total_issuance() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 4_558_000 picoseconds.
        Weight::from_parts(4_930_000, 0)
    }
    fn burn_allow_death() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 27_140_000 picoseconds.
        Weight::from_parts(27_622_000, 0)
    }
    fn burn_keep_alive() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 17_953_000 picoseconds.
        Weight::from_parts(18_404_000, 0)
    }
}
