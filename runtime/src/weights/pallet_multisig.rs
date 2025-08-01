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

//! Autogenerated weights for `pallet_multisig`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.2.0
//! DATE: 2025-06-05, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `8e3da28b8f95`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --runtime
// /app/zkv_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --pallet
// pallet-multisig
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
// /data/benchmark/runtime/src/weights/pallet_multisig.rs
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

/// Weights for `pallet_multisig` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_multisig::WeightInfo for ZKVWeight<T> {
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_threshold_1(z: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 12_894_000 picoseconds.
        Weight::from_parts(13_245_507, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(338, 0).saturating_mul(z.into()))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_create(s: u32, z: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `333 + s * (2 ±0)`
        //  Estimated: `6811`
        // Minimum execution time: 39_334_000 picoseconds.
        Weight::from_parts(31_988_723, 6811)
            // Standard Error: 756
            .saturating_add(Weight::from_parts(82_991, 0).saturating_mul(s.into()))
            // Standard Error: 7
            .saturating_add(Weight::from_parts(2_057, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[3, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_approve(s: u32, z: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `353`
        //  Estimated: `6811`
        // Minimum execution time: 25_808_000 picoseconds.
        Weight::from_parts(18_990_697, 6811)
            // Standard Error: 694
            .saturating_add(Weight::from_parts(76_131, 0).saturating_mul(s.into()))
            // Standard Error: 6
            .saturating_add(Weight::from_parts(2_075, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_complete(s: u32, z: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `456 + s * (33 ±0)`
        //  Estimated: `6811`
        // Minimum execution time: 43_631_000 picoseconds.
        Weight::from_parts(34_643_380, 6811)
            // Standard Error: 766
            .saturating_add(Weight::from_parts(103_207, 0).saturating_mul(s.into()))
            // Standard Error: 7
            .saturating_add(Weight::from_parts(2_092, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    fn approve_as_multi_create(s: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `334 + s * (2 ±0)`
        //  Estimated: `6811`
        // Minimum execution time: 29_866_000 picoseconds.
        Weight::from_parts(30_199_078, 6811)
            // Standard Error: 1_061
            .saturating_add(Weight::from_parts(80_311, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    fn approve_as_multi_approve(s: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `353`
        //  Estimated: `6811`
        // Minimum execution time: 17_172_000 picoseconds.
        Weight::from_parts(17_233_882, 6811)
            // Standard Error: 1_070
            .saturating_add(Weight::from_parts(73_484, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    fn cancel_as_multi(s: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `525 + s * (1 ±0)`
        //  Estimated: `6811`
        // Minimum execution time: 30_718_000 picoseconds.
        Weight::from_parts(31_000_094, 6811)
            // Standard Error: 1_140
            .saturating_add(Weight::from_parts(78_398, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}
