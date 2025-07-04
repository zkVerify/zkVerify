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

//! Autogenerated weights for `parachains :: on_demand`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.2.0
//! DATE: 2025-06-05, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `7873ac728693`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --runtime
// /app/zkv_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --pallet
// parachains :: on-demand
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
// /data/benchmark/runtime/src/weights/parachains/on_demand.rs
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

/// Weights for `parachains :: on_demand` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> parachains :: on_demand::WeightInfo for ZKVWeight<T> {
    /// Storage: `OnDemandAssignmentProvider::QueueStatus` (r:1 w:1)
    /// Proof: `OnDemandAssignmentProvider::QueueStatus` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// Storage: `OnDemandAssignmentProvider::Revenue` (r:1 w:1)
    /// Proof: `OnDemandAssignmentProvider::Revenue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `OnDemandAssignmentProvider::ParaIdAffinity` (r:1 w:0)
    /// Proof: `OnDemandAssignmentProvider::ParaIdAffinity` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `OnDemandAssignmentProvider::FreeEntries` (r:1 w:1)
    /// Proof: `OnDemandAssignmentProvider::FreeEntries` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// The range of component `s` is `[1, 9999]`.
    fn place_order_keep_alive(s: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `185 + s * (8 ±0)`
        //  Estimated: `3648 + s * (8 ±0)`
        // Minimum execution time: 34_745_000 picoseconds.
        Weight::from_parts(26_463_246, 3648)
            // Standard Error: 175
            .saturating_add(Weight::from_parts(17_884, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
            .saturating_add(Weight::from_parts(0, 8).saturating_mul(s.into()))
    }
    /// Storage: `OnDemandAssignmentProvider::QueueStatus` (r:1 w:1)
    /// Proof: `OnDemandAssignmentProvider::QueueStatus` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// Storage: `OnDemandAssignmentProvider::Revenue` (r:1 w:1)
    /// Proof: `OnDemandAssignmentProvider::Revenue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `OnDemandAssignmentProvider::ParaIdAffinity` (r:1 w:0)
    /// Proof: `OnDemandAssignmentProvider::ParaIdAffinity` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `OnDemandAssignmentProvider::FreeEntries` (r:1 w:1)
    /// Proof: `OnDemandAssignmentProvider::FreeEntries` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// The range of component `s` is `[1, 9999]`.
    fn place_order_allow_death(s: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `185 + s * (8 ±0)`
        //  Estimated: `3648 + s * (8 ±0)`
        // Minimum execution time: 34_865_000 picoseconds.
        Weight::from_parts(28_713_134, 3648)
            // Standard Error: 155
            .saturating_add(Weight::from_parts(16_983, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
            .saturating_add(Weight::from_parts(0, 8).saturating_mul(s.into()))
    }
}
