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

//! Autogenerated weights for `pallet_ultraplonk_verifier`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 43.0.0
//! DATE: 2025-03-01, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `5ad58e09d1d0`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet-ultraplonk-verifier
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
// /data/benchmark/runtime/src/weights/pallet_ultraplonk_verifier.rs
// --template
// /data/benchmark/relay-node/benchmarks/zkv-deploy-weight-template.hbs
// --base-path=/tmp/tmp.1rbLszjw13

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weights for `pallet_ultraplonk_verifier` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_ultraplonk_verifier::WeightInfo for ZKVWeight<T> {
    fn verify_proof() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_226_513_000 picoseconds.
        Weight::from_parts(2_243_546_000, 0)
    }
    /// Storage: `SettlementUltraplonkPallet::Vks` (r:1 w:0)
    /// Proof: `SettlementUltraplonkPallet::Vks` (`max_values`: None, `max_size`: Some(1672), added: 4147, mode: `MaxEncodedLen`)
    fn get_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1716`
        //  Estimated: `5137`
        // Minimum execution time: 5_310_000 picoseconds.
        Weight::from_parts(5_470_000, 5137)
            .saturating_add(T::DbWeight::get().reads(1_u64))
    }
    fn validate_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 28_533_000 picoseconds.
        Weight::from_parts(28_894_000, 0)
    }
    fn compute_statement_hash() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 8_436_000 picoseconds.
        Weight::from_parts(8_566_000, 0)
    }
    /// Storage: `SettlementUltraplonkPallet::Disabled` (r:1 w:0)
    /// Proof: `SettlementUltraplonkPallet::Disabled` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
    /// Storage: `SettlementUltraplonkPallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementUltraplonkPallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(121), added: 2596, mode: `MaxEncodedLen`)
    /// Storage: `SettlementUltraplonkPallet::Vks` (r:1 w:1)
    /// Proof: `SettlementUltraplonkPallet::Vks` (`max_values`: None, `max_size`: Some(1672), added: 4147, mode: `MaxEncodedLen`)
    fn register_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `4`
        //  Estimated: `5137`
        // Minimum execution time: 76_895_000 picoseconds.
        Weight::from_parts(77_986_000, 5137)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `SettlementUltraplonkPallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementUltraplonkPallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(121), added: 2596, mode: `MaxEncodedLen`)
    /// Storage: `SettlementUltraplonkPallet::Vks` (r:1 w:1)
    /// Proof: `SettlementUltraplonkPallet::Vks` (`max_values`: None, `max_size`: Some(1672), added: 4147, mode: `MaxEncodedLen`)
    fn unregister_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1881`
        //  Estimated: `5137`
        // Minimum execution time: 41_237_000 picoseconds.
        Weight::from_parts(42_029_000, 5137)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
}
