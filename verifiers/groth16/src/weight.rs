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
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.0
//! DATE: 2025-02-12, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `miklap`, CPU: `11th Gen Intel(R) Core(TM) i7-11850H @ 2.50GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/zkv-node
// benchmark
// pallet
// --chain
// dev
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
// /home/mdamico/devel/zkVerify/HEADER-APACHE2
// --output
// verifiers/groth16/src/weight.rs
// --template
// /home/mdamico/devel/zkVerify/node/zkv-pallets-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_groth16_verifier`.
pub trait WeightInfo {
    fn verify_proof_bn254(n: u32, ) -> Weight;
    fn verify_proof_bls12_381(n: u32, ) -> Weight;
    fn get_vk() -> Weight;
    fn validate_vk_bn254(n: u32, ) -> Weight;
    fn validate_vk_bls12_381(n: u32, ) -> Weight;
    fn compute_statement_hash(n: u32, ) -> Weight;
    fn register_vk_bn254(n: u32, ) -> Weight;
    fn register_vk_bls12_381(n: u32, ) -> Weight;
    fn unregister_vk() -> Weight;
}

// For backwards compatibility and tests.
impl WeightInfo for () {
    /// The range of component `n` is `[0, 16]`.
    fn verify_proof_bn254(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_454_192_000 picoseconds.
        Weight::from_parts(2_473_256_053, 0)
            // Standard Error: 811_952
            .saturating_add(Weight::from_parts(114_919_711, 0).saturating_mul(n.into()))
    }
    /// The range of component `n` is `[0, 16]`.
    fn verify_proof_bls12_381(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_441_301_000 picoseconds.
        Weight::from_parts(3_674_034_551, 0)
            // Standard Error: 1_406_009
            .saturating_add(Weight::from_parts(204_571_428, 0).saturating_mul(n.into()))
    }
    /// Storage: `SettlementGroth16Pallet::Vks` (r:1 w:0)
    /// Proof: `SettlementGroth16Pallet::Vks` (`max_values`: None, `max_size`: Some(3956), added: 6431, mode: `MaxEncodedLen`)
    fn get_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2421`
        //  Estimated: `7421`
        // Minimum execution time: 12_949_000 picoseconds.
        Weight::from_parts(14_898_000, 7421)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
    }
    /// The range of component `n` is `[0, 16]`.
    fn validate_vk_bn254(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_032_485_000 picoseconds.
        Weight::from_parts(1_050_182_017, 0)
            // Standard Error: 682_670
            .saturating_add(Weight::from_parts(103_251_576, 0).saturating_mul(n.into()))
    }
    /// The range of component `n` is `[0, 16]`.
    fn validate_vk_bls12_381(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 398_352_000 picoseconds.
        Weight::from_parts(413_390_373, 0)
            // Standard Error: 127_061
            .saturating_add(Weight::from_parts(63_724_450, 0).saturating_mul(n.into()))
    }
    /// The range of component `n` is `[0, 16]`.
    fn compute_statement_hash(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 7_734_000 picoseconds.
        Weight::from_parts(9_716_206, 0)
            // Standard Error: 12_342
            .saturating_add(Weight::from_parts(882_153, 0).saturating_mul(n.into()))
    }
    fn register_vk_bn254(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `6`
        //  Estimated: `7421`
        // Minimum execution time: 1_016_480_000 picoseconds.
        Weight::from_parts(1_088_139_888, 7421)
            // Standard Error: 362_999
            .saturating_add(Weight::from_parts(86_573_273, 0).saturating_mul(n.into()))
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
    /// Storage: `SettlementGroth16Pallet::Disabled` (r:1 w:0)
    /// Proof: `SettlementGroth16Pallet::Disabled` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
    /// Storage: `SettlementGroth16Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Tickets` (`max_values`: None, `max_size`: Some(97), added: 2572, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(121), added: 2596, mode: `MaxEncodedLen`)
    /// Storage: `SettlementGroth16Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Vks` (`max_values`: None, `max_size`: Some(3956), added: 6431, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[0, 16]`.
    fn register_vk_bls12_381(n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `6`
        //  Estimated: `7421`
        // Minimum execution time: 421_741_000 picoseconds.
        Weight::from_parts(494_500_781, 7421)
            // Standard Error: 170_598
            .saturating_add(Weight::from_parts(58_181_393, 0).saturating_mul(n.into()))
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
    /// Storage: `SettlementGroth16Pallet::Tickets` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Tickets` (`max_values`: None, `max_size`: Some(97), added: 2572, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Holds` (r:1 w:1)
    /// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(121), added: 2596, mode: `MaxEncodedLen`)
    /// Storage: `SettlementGroth16Pallet::Vks` (r:1 w:1)
    /// Proof: `SettlementGroth16Pallet::Vks` (`max_values`: None, `max_size`: Some(3956), added: 6431, mode: `MaxEncodedLen`)
    fn unregister_vk() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `775`
        //  Estimated: `7421`
        // Minimum execution time: 68_718_000 picoseconds.
        Weight::from_parts(79_262_000, 7421)
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
}