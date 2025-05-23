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

//! Autogenerated weights for `pallet_groth16_verifier`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 43.0.0
//! DATE: 2025-05-23, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `b367aba62c4b`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --runtime
// /app/zkv_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --pallet
// pallet-groth16-verifier
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
// /data/benchmark/runtime/src/weights/pallet_groth16_verifier.rs
// --template
// /data/benchmark/relay-node/benchmarks/zkv-deploy-weight-template.hbs
// --base-path=/tmp/tmp.7uWIhySBJt

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weights for `pallet_groth16_verifier` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_groth16_verifier::WeightInfo for ZKVWeight<T> {
    /// The range of component `n` is `[0, 64]`.
    fn verify_proof_bn254(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_864_298_000 picoseconds.
        Weight::from_parts(2_878_613_619, 0)
            // Standard Error: 17_187
            .saturating_add(Weight::from_parts(102_306_758, 0).saturating_mul(n.into()))
    }
    /// The range of component `n` is `[0, 64]`.
    fn verify_proof_bls12_381(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_630_005_000 picoseconds.
        Weight::from_parts(3_652_175_451, 0)
            // Standard Error: 31_403
            .saturating_add(Weight::from_parts(186_604_384, 0).saturating_mul(n.into()))
    }
    /// Storage: `SettlementGroth16Pallet::Vks` (r:1 w:0)
    /// Proof: `SettlementGroth16Pallet::Vks` (`max_values`: None, `max_size`: Some(7093), added: 9568, mode: `MaxEncodedLen`)
    fn get_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `7209`
        //  Estimated: `10558`
        // Minimum execution time: 13_185_000 picoseconds.
        Weight::from_parts(13_415_000, 10558)
            .saturating_add(T::DbWeight::get().reads(1_u64))
    }
    /// The range of component `n` is `[0, 64]`.
    fn validate_vk_bn254(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_030_140_000 picoseconds.
        Weight::from_parts(1_035_882_075, 0)
            // Standard Error: 6_571
            .saturating_add(Weight::from_parts(93_703_989, 0).saturating_mul(n.into()))
    }
    /// The range of component `n` is `[0, 64]`.
    fn validate_vk_bls12_381(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 401_529_000 picoseconds.
        Weight::from_parts(406_071_056, 0)
            // Standard Error: 4_122
            .saturating_add(Weight::from_parts(61_788_610, 0).saturating_mul(n.into()))
    }
    /// The range of component `n` is `[0, 64]`.
    fn compute_statement_hash(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 5_811_000 picoseconds.
        Weight::from_parts(6_516_432, 0)
            // Standard Error: 636
            .saturating_add(Weight::from_parts(459_205, 0).saturating_mul(n.into()))
    }
    /// Storage: `SettlementGroth16Pallet::Disabled` (r:1 w:0)
    /// Proof: `SettlementGroth16Pallet::Disabled` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
    /// Storage: `SettlementGroth16Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
    /// Storage: `SettlementGroth16Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Vks` (`max_values`: None, `max_size`: Some(7093), added: 9568, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[0, 64]`.
    fn register_vk_bn254(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `76`
        //  Estimated: `10558`
        // Minimum execution time: 1_090_715_000 picoseconds.
        Weight::from_parts(1_098_068_241, 10558)
            // Standard Error: 6_187
            .saturating_add(Weight::from_parts(94_052_318, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `SettlementGroth16Pallet::Disabled` (r:1 w:0)
    /// Proof: `SettlementGroth16Pallet::Disabled` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
    /// Storage: `SettlementGroth16Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
    /// Storage: `SettlementGroth16Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Vks` (`max_values`: None, `max_size`: Some(7093), added: 9568, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[0, 64]`.
    fn register_vk_bls12_381(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `76`
        //  Estimated: `10558`
        // Minimum execution time: 462_093_000 picoseconds.
        Weight::from_parts(468_424_057, 10558)
            // Standard Error: 4_392
            .saturating_add(Weight::from_parts(62_285_144, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `SettlementGroth16Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Tickets` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(139), added: 2614, mode: `MaxEncodedLen`)
    /// Storage: `SettlementGroth16Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Vks` (`max_values`: None, `max_size`: Some(7093), added: 9568, mode: `MaxEncodedLen`)
    fn unregister_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `845`
        //  Estimated: `10558`
        // Minimum execution time: 51_385_000 picoseconds.
        Weight::from_parts(52_027_000, 10558)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
}
