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

//! Autogenerated weights for `pallet_bounties`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.2.0
//! DATE: 2025-06-05, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `4694c7c49f7d`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --runtime
// /app/zkv_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --pallet
// pallet-bounties
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
// /data/benchmark/runtime/src/weights/pallet_bounties.rs
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

/// Weights for `pallet_bounties` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_bounties::WeightInfo for ZKVWeight<T> {
    /// Storage: `Bounties::BountyCount` (r:1 w:1)
    /// Proof: `Bounties::BountyCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::BountyDescriptions` (r:0 w:1)
    /// Proof: `Bounties::BountyDescriptions` (`max_values`: None, `max_size`: Some(16400), added: 18875, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::Bounties` (r:0 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// The range of component `d` is `[0, 16384]`.
    fn propose_bounty(d: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `107`
        //  Estimated: `3593`
        // Minimum execution time: 24_356_000 picoseconds.
        Weight::from_parts(25_229_520, 3593)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(482, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::BountyApprovals` (r:1 w:1)
    /// Proof: `Bounties::BountyApprovals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
    fn approve_bounty() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `198`
        //  Estimated: `3642`
        // Minimum execution time: 13_235_000 picoseconds.
        Weight::from_parts(13_595_000, 3642)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    fn propose_curator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `218`
        //  Estimated: `3642`
        // Minimum execution time: 9_818_000 picoseconds.
        Weight::from_parts(10_229_000, 3642)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::BountyApprovals` (r:1 w:1)
    /// Proof: `Bounties::BountyApprovals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
    fn approve_bounty_with_curator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `198`
        //  Estimated: `3642`
        // Minimum execution time: 12_153_000 picoseconds.
        Weight::from_parts(12_574_000, 3642)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:2 w:2)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn unassign_curator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `460`
        //  Estimated: `6196`
        // Minimum execution time: 33_753_000 picoseconds.
        Weight::from_parts(34_634_000, 6196)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    fn accept_curator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `353`
        //  Estimated: `3642`
        // Minimum execution time: 23_774_000 picoseconds.
        Weight::from_parts(24_245_000, 3642)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `ChildBounties::ParentChildBounties` (r:1 w:0)
    /// Proof: `ChildBounties::ParentChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
    fn award_bounty() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `296`
        //  Estimated: `3642`
        // Minimum execution time: 13_535_000 picoseconds.
        Weight::from_parts(13_976_000, 3642)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:3 w:3)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// Storage: `ChildBounties::ChildrenCuratorFees` (r:1 w:1)
    /// Proof: `ChildBounties::ChildrenCuratorFees` (`max_values`: None, `max_size`: Some(28), added: 2503, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::BountyDescriptions` (r:0 w:1)
    /// Proof: `Bounties::BountyDescriptions` (`max_values`: None, `max_size`: Some(16400), added: 18875, mode: `MaxEncodedLen`)
    /// Storage: `ChildBounties::ParentTotalChildBounties` (r:0 w:1)
    /// Proof: `ChildBounties::ParentTotalChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
    /// Storage: `ChildBounties::ParentChildBounties` (r:0 w:1)
    /// Proof: `ChildBounties::ParentChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
    fn claim_bounty() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `623`
        //  Estimated: `8799`
        // Minimum execution time: 98_944_000 picoseconds.
        Weight::from_parts(100_197_000, 8799)
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(8_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `ChildBounties::ParentChildBounties` (r:1 w:0)
    /// Proof: `ChildBounties::ParentChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:2 w:2)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::BountyDescriptions` (r:0 w:1)
    /// Proof: `Bounties::BountyDescriptions` (`max_values`: None, `max_size`: Some(16400), added: 18875, mode: `MaxEncodedLen`)
    fn close_bounty_proposed() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `443`
        //  Estimated: `6196`
        // Minimum execution time: 40_054_000 picoseconds.
        Weight::from_parts(40_586_000, 6196)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `ChildBounties::ParentChildBounties` (r:1 w:1)
    /// Proof: `ChildBounties::ParentChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:3 w:3)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::BountyDescriptions` (r:0 w:1)
    /// Proof: `Bounties::BountyDescriptions` (`max_values`: None, `max_size`: Some(16400), added: 18875, mode: `MaxEncodedLen`)
    /// Storage: `ChildBounties::ParentTotalChildBounties` (r:0 w:1)
    /// Proof: `ChildBounties::ParentTotalChildBounties` (`max_values`: None, `max_size`: Some(16), added: 2491, mode: `MaxEncodedLen`)
    fn close_bounty_active() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `642`
        //  Estimated: `8799`
        // Minimum execution time: 69_840_000 picoseconds.
        Weight::from_parts(70_732_000, 8799)
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(7_u64))
    }
    /// Storage: `Bounties::Bounties` (r:1 w:1)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    fn extend_bounty_expiry() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `254`
        //  Estimated: `3642`
        // Minimum execution time: 10_469_000 picoseconds.
        Weight::from_parts(10_720_000, 3642)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Bounties::BountyApprovals` (r:1 w:1)
    /// Proof: `Bounties::BountyApprovals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::Bounties` (r:100 w:100)
    /// Proof: `Bounties::Bounties` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:200 w:200)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// The range of component `b` is `[0, 100]`.
    fn spend_funds(b: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + b * (292 ±0)`
        //  Estimated: `1887 + b * (5206 ±0)`
        // Minimum execution time: 4_979_000 picoseconds.
        Weight::from_parts(5_069_000, 1887)
            // Standard Error: 10_082
            .saturating_add(Weight::from_parts(32_641_280, 0).saturating_mul(b.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(b.into())))
            .saturating_add(T::DbWeight::get().writes(1_u64))
            .saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(b.into())))
            .saturating_add(Weight::from_parts(0, 5206).saturating_mul(b.into()))
    }
}
