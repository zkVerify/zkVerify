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

//! Autogenerated weights for `pallet_referenda`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.2.0
//! DATE: 2025-06-05, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `eadb542290dd`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --runtime
// /app/zkv_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --pallet
// pallet-referenda
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
// /data/benchmark/runtime/src/weights/pallet_referenda.rs
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

/// Weights for `pallet_referenda` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_referenda::WeightInfo for ZKVWeight<T> {
    /// Storage: `Referenda::ReferendumCount` (r:1 w:1)
    /// Proof: `Referenda::ReferendumCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::ReferendumInfoFor` (r:0 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    fn submit() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `110`
        //  Estimated: `42428`
        // Minimum execution time: 30_898_000 picoseconds.
        Weight::from_parts(31_969_000, 42428)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:2 w:2)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Retries` (r:0 w:1)
    /// Proof: `Scheduler::Retries` (`max_values`: None, `max_size`: Some(30), added: 2505, mode: `MaxEncodedLen`)
    fn place_decision_deposit_preparing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `363`
        //  Estimated: `83866`
        // Minimum execution time: 42_349_000 picoseconds.
        Weight::from_parts(43_361_000, 83866)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:0)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::TrackQueue` (r:1 w:1)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Retries` (r:0 w:1)
    /// Proof: `Scheduler::Retries` (`max_values`: None, `max_size`: Some(30), added: 2505, mode: `MaxEncodedLen`)
    fn place_decision_deposit_queued() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3150`
        //  Estimated: `42428`
        // Minimum execution time: 53_960_000 picoseconds.
        Weight::from_parts(54_943_000, 42428)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:0)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::TrackQueue` (r:1 w:1)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Retries` (r:0 w:1)
    /// Proof: `Scheduler::Retries` (`max_values`: None, `max_size`: Some(30), added: 2505, mode: `MaxEncodedLen`)
    fn place_decision_deposit_not_queued() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3170`
        //  Estimated: `42428`
        // Minimum execution time: 53_249_000 picoseconds.
        Weight::from_parts(54_001_000, 42428)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:1)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:2 w:2)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Retries` (r:0 w:1)
    /// Proof: `Scheduler::Retries` (`max_values`: None, `max_size`: Some(30), added: 2505, mode: `MaxEncodedLen`)
    fn place_decision_deposit_passing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `363`
        //  Estimated: `83866`
        // Minimum execution time: 46_877_000 picoseconds.
        Weight::from_parts(47_689_000, 83866)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:1)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:2 w:2)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Retries` (r:0 w:1)
    /// Proof: `Scheduler::Retries` (`max_values`: None, `max_size`: Some(30), added: 2505, mode: `MaxEncodedLen`)
    fn place_decision_deposit_failing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `363`
        //  Estimated: `83866`
        // Minimum execution time: 48_601_000 picoseconds.
        Weight::from_parts(49_352_000, 83866)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    fn refund_decision_deposit() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `279`
        //  Estimated: `4401`
        // Minimum execution time: 26_410_000 picoseconds.
        Weight::from_parts(27_070_000, 4401)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    fn refund_submission_deposit() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `232`
        //  Estimated: `4401`
        // Minimum execution time: 26_179_000 picoseconds.
        Weight::from_parts(26_660_000, 4401)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:2 w:2)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Retries` (r:0 w:1)
    /// Proof: `Scheduler::Retries` (`max_values`: None, `max_size`: Some(30), added: 2505, mode: `MaxEncodedLen`)
    fn cancel() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `308`
        //  Estimated: `83866`
        // Minimum execution time: 29_716_000 picoseconds.
        Weight::from_parts(30_166_000, 83866)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:2 w:2)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::MetadataOf` (r:1 w:0)
    /// Proof: `Referenda::MetadataOf` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Retries` (r:0 w:1)
    /// Proof: `Scheduler::Retries` (`max_values`: None, `max_size`: Some(30), added: 2505, mode: `MaxEncodedLen`)
    fn kill() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `615`
        //  Estimated: `83866`
        // Minimum execution time: 81_552_000 picoseconds.
        Weight::from_parts(82_513_000, 83866)
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    /// Storage: `Referenda::TrackQueue` (r:1 w:0)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:1)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    fn one_fewer_deciding_queue_empty() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `102`
        //  Estimated: `5477`
        // Minimum execution time: 8_856_000 picoseconds.
        Weight::from_parts(9_167_000, 5477)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Referenda::TrackQueue` (r:1 w:1)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn one_fewer_deciding_failing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3077`
        //  Estimated: `42428`
        // Minimum execution time: 35_356_000 picoseconds.
        Weight::from_parts(36_218_000, 42428)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `Referenda::TrackQueue` (r:1 w:1)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn one_fewer_deciding_passing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3077`
        //  Estimated: `42428`
        // Minimum execution time: 37_430_000 picoseconds.
        Weight::from_parts(38_101_000, 42428)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:0)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::TrackQueue` (r:1 w:1)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    fn nudge_referendum_requeued_insertion() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2939`
        //  Estimated: `5477`
        // Minimum execution time: 17_503_000 picoseconds.
        Weight::from_parts(18_033_000, 5477)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:0)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::TrackQueue` (r:1 w:1)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    fn nudge_referendum_requeued_slide() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2939`
        //  Estimated: `5477`
        // Minimum execution time: 17_212_000 picoseconds.
        Weight::from_parts(17_763_000, 5477)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:0)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::TrackQueue` (r:1 w:1)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    fn nudge_referendum_queued() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2943`
        //  Estimated: `5477`
        // Minimum execution time: 22_232_000 picoseconds.
        Weight::from_parts(22_822_000, 5477)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:0)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::TrackQueue` (r:1 w:1)
    /// Proof: `Referenda::TrackQueue` (`max_values`: None, `max_size`: Some(2012), added: 4487, mode: `MaxEncodedLen`)
    fn nudge_referendum_not_queued() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2963`
        //  Estimated: `5477`
        // Minimum execution time: 21_721_000 picoseconds.
        Weight::from_parts(22_142_000, 5477)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_no_deposit() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `260`
        //  Estimated: `42428`
        // Minimum execution time: 20_298_000 picoseconds.
        Weight::from_parts(20_969_000, 42428)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_preparing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `308`
        //  Estimated: `42428`
        // Minimum execution time: 20_307_000 picoseconds.
        Weight::from_parts(21_009_000, 42428)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    fn nudge_referendum_timed_out() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `206`
        //  Estimated: `4401`
        // Minimum execution time: 13_916_000 picoseconds.
        Weight::from_parts(14_316_000, 4401)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:1)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_begin_deciding_failing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `308`
        //  Estimated: `42428`
        // Minimum execution time: 26_459_000 picoseconds.
        Weight::from_parts(27_261_000, 42428)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::DecidingCount` (r:1 w:1)
    /// Proof: `Referenda::DecidingCount` (`max_values`: None, `max_size`: Some(14), added: 2489, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_begin_deciding_passing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `308`
        //  Estimated: `42428`
        // Minimum execution time: 27_641_000 picoseconds.
        Weight::from_parts(28_263_000, 42428)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_begin_confirming() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `361`
        //  Estimated: `42428`
        // Minimum execution time: 21_079_000 picoseconds.
        Weight::from_parts(21_581_000, 42428)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_end_confirming() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `344`
        //  Estimated: `42428`
        // Minimum execution time: 20_599_000 picoseconds.
        Weight::from_parts(20_999_000, 42428)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_continue_not_confirming() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `361`
        //  Estimated: `42428`
        // Minimum execution time: 20_388_000 picoseconds.
        Weight::from_parts(21_040_000, 42428)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_continue_confirming() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `365`
        //  Estimated: `42428`
        // Minimum execution time: 19_546_000 picoseconds.
        Weight::from_parts(20_097_000, 42428)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:2 w:2)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Lookup` (r:1 w:1)
    /// Proof: `Scheduler::Lookup` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
    fn nudge_referendum_approved() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `365`
        //  Estimated: `83866`
        // Minimum execution time: 29_895_000 picoseconds.
        Weight::from_parts(30_416_000, 83866)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
    fn nudge_referendum_rejected() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `361`
        //  Estimated: `42428`
        // Minimum execution time: 20_839_000 picoseconds.
        Weight::from_parts(21_439_000, 42428)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:0)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Preimage::StatusFor` (r:1 w:0)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// Storage: `Preimage::RequestStatusFor` (r:1 w:0)
    /// Proof: `Preimage::RequestStatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::MetadataOf` (r:0 w:1)
    /// Proof: `Referenda::MetadataOf` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
    fn set_some_metadata() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `350`
        //  Estimated: `4401`
        // Minimum execution time: 20_588_000 picoseconds.
        Weight::from_parts(21_240_000, 4401)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: `Referenda::ReferendumInfoFor` (r:1 w:0)
    /// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(936), added: 3411, mode: `MaxEncodedLen`)
    /// Storage: `Referenda::MetadataOf` (r:1 w:1)
    /// Proof: `Referenda::MetadataOf` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
    fn clear_metadata() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `283`
        //  Estimated: `4401`
        // Minimum execution time: 16_371_000 picoseconds.
        Weight::from_parts(16_831_000, 4401)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}
