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

//! Autogenerated weights for `pallet_plonky2_verifier`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 43.0.0
//! DATE: 2025-04-28, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `miklap`, CPU: `11th Gen Intel(R) Core(TM) i7-11850H @ 2.50GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// target/release/zkv-relay
// benchmark
// pallet
// --runtime
// target/release/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --pallet
// pallet-plonky2-verifier
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --heap-pages=4096
// --header
// /home/mdamico/devel/zkVerify/HEADER-APACHE2
// --output
// verifiers/plonky2/src/weight.rs
// --template
// /home/mdamico/devel/zkVerify/relay-node/benchmarks/zkv-pallets-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_plonky2_verifier`.
pub trait WeightInfo {
    fn get_vk() -> Weight;
    fn validate_vk() -> Weight;
    fn compute_statement_hash() -> Weight;
    fn register_vk() -> Weight;
    fn unregister_vk() -> Weight;
}

// For backwards compatibility and tests.
impl WeightInfo for () {
    /// Storage: `SettlementPlonky2Pallet::Vks` (r:1 w:0)
    /// Proof: `SettlementPlonky2Pallet::Vks` (`max_values`: None, `max_size`: Some(50045), added: 52520, mode: `MaxEncodedLen`)
    fn get_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1672`
        //  Estimated: `53510`
        // Minimum execution time: 6_230_000 picoseconds.
        Weight::from_parts(6_557_000, 53510)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
    }
    fn validate_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 7_942_000 picoseconds.
        Weight::from_parts(8_111_000, 0)
    }
    fn compute_statement_hash() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 7_283_000 picoseconds.
        Weight::from_parts(7_499_000, 0)
    }
    /// Storage: `SettlementPlonky2Pallet::Disabled` (r:1 w:0)
    /// Proof: `SettlementPlonky2Pallet::Disabled` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
    /// Storage: `SettlementPlonky2Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementPlonky2Pallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
    /// Storage: `SettlementPlonky2Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementPlonky2Pallet::Vks` (`max_values`: None, `max_size`: Some(50045), added: 52520, mode: `MaxEncodedLen`)
    fn register_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `4`
        //  Estimated: `53510`
        // Minimum execution time: 70_306_000 picoseconds.
        Weight::from_parts(72_131_000, 53510)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
    /// Storage: `SettlementPlonky2Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementPlonky2Pallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
    /// Storage: `SettlementPlonky2Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementPlonky2Pallet::Vks` (`max_values`: None, `max_size`: Some(50045), added: 52520, mode: `MaxEncodedLen`)
    fn unregister_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1837`
        //  Estimated: `53510`
        // Minimum execution time: 51_301_000 picoseconds.
        Weight::from_parts(53_127_000, 53510)
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
}