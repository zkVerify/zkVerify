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

//! Autogenerated weights for `pallet_sp1_verifier`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.2.0
//! DATE: 2025-07-25, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `5faa184bc936`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --runtime
// /app/zkv_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --pallet
// pallet-sp1-verifier
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
// /data/benchmark/runtime/src/weights/pallet_sp1_verifier.rs
// --template
// /data/benchmark/relay-node/benchmarks/zkv-deploy-weight-template.hbs
// --base-path=/tmp/tmp.RYO1xdABsn

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use crate::weight_aliases::*;
use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;
use crate::parachains;

/// Weights for `pallet_sp1_verifier` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_sp1_verifier::WeightInfo for ZKVWeight<T> {
    fn verify_proof() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 65_660_673_000 picoseconds.
        Weight::from_parts(65_701_118_000, 0)
    }
    /// Storage: `SettlementSp1Pallet::Vks` (r:1 w:0)
    /// Proof: `SettlementSp1Pallet::Vks` (`max_values`: None, `max_size`: Some(72), added: 2547, mode: `MaxEncodedLen`)
    fn get_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `115`
        //  Estimated: `3537`
        // Minimum execution time: 3_998_000 picoseconds.
        Weight::from_parts(4_118_000, 3537)
            .saturating_add(T::DbWeight::get().reads(1_u64))
    }
    fn validate_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 120_000 picoseconds.
        Weight::from_parts(160_000, 0)
    }
    fn compute_statement_hash() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_224_000 picoseconds.
        Weight::from_parts(2_305_000, 0)
    }
    /// Storage: `SettlementSp1Pallet::Disabled` (r:1 w:0)
    /// Proof: `SettlementSp1Pallet::Disabled` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
    /// Storage: `SettlementSp1Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementSp1Pallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
    /// Storage: `SettlementSp1Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementSp1Pallet::Vks` (`max_values`: None, `max_size`: Some(72), added: 2547, mode: `MaxEncodedLen`)
    fn register_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `4`
        //  Estimated: `3604`
        // Minimum execution time: 42_188_000 picoseconds.
        Weight::from_parts(43_371_000, 3604)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `SettlementSp1Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementSp1Pallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
    /// Storage: `SettlementSp1Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementSp1Pallet::Vks` (`max_values`: None, `max_size`: Some(72), added: 2547, mode: `MaxEncodedLen`)
    fn unregister_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `280`
        //  Estimated: `3604`
        // Minimum execution time: 38_111_000 picoseconds.
        Weight::from_parts(39_082_000, 3604)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
}
