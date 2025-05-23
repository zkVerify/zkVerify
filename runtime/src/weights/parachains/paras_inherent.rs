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

//! Autogenerated weights for `crate::parachains::paras_inherent`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 43.0.0
//! DATE: 2025-02-28, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `9a35bf1200e4`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// /usr/local/bin/zkv-relay
// benchmark
// pallet
// --chain
// dev
// --pallet
// crate::parachains::paras-inherent
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
// /data/benchmark/runtime/src/weights/parachains/paras_inherent.rs
// --template
// /data/benchmark/relay-node/benchmarks/zkv-deploy-weight-template.hbs
// --base-path=/tmp/tmp.1rbLszjw13

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weights for `crate::parachains::paras_inherent` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> crate::parachains::paras_inherent::WeightInfo for ZKVWeight<T> {
    /// Storage: `ParaInherent::Included` (r:1 w:1)
    /// Proof: `ParaInherent::Included` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `System::ParentHash` (r:1 w:0)
    /// Proof: `System::ParentHash` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
    /// Storage: `ParasShared::AllowedRelayParents` (r:1 w:1)
    /// Proof: `ParasShared::AllowedRelayParents` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::CurrentSessionIndex` (r:1 w:0)
    /// Proof: `ParasShared::CurrentSessionIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::AvailabilityCores` (r:1 w:1)
    /// Proof: `ParaScheduler::AvailabilityCores` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::ActiveValidatorKeys` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorKeys` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Babe::AuthorVrfRandomness` (r:1 w:0)
    /// Proof: `Babe::AuthorVrfRandomness` (`max_values`: Some(1), `max_size`: Some(33), added: 528, mode: `MaxEncodedLen`)
    /// Storage: `ParaInherent::OnChainVotes` (r:1 w:1)
    /// Proof: `ParaInherent::OnChainVotes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Frozen` (r:1 w:0)
    /// Proof: `ParasDisputes::Frozen` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaInclusion::V1` (r:1 w:0)
    /// Proof: `ParaInclusion::V1` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::SessionStartBlock` (r:1 w:0)
    /// Proof: `ParaScheduler::SessionStartBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ValidatorGroups` (r:1 w:0)
    /// Proof: `ParaScheduler::ValidatorGroups` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ClaimQueue` (r:1 w:1)
    /// Proof: `ParaScheduler::ClaimQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::ActiveValidatorIndices` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Session::DisabledValidators` (r:1 w:0)
    /// Proof: `Session::DisabledValidators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    fn enter_empty() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `37588`
        //  Estimated: `41053`
        // Minimum execution time: 149_130_000 picoseconds.
        Weight::from_parts(152_126_000, 41053)
            .saturating_add(T::DbWeight::get().reads(15_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    /// Storage: `ParaInherent::Included` (r:1 w:1)
    /// Proof: `ParaInherent::Included` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `System::ParentHash` (r:1 w:0)
    /// Proof: `System::ParentHash` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
    /// Storage: `ParasShared::AllowedRelayParents` (r:1 w:1)
    /// Proof: `ParasShared::AllowedRelayParents` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::CurrentSessionIndex` (r:1 w:0)
    /// Proof: `ParasShared::CurrentSessionIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::AvailabilityCores` (r:1 w:1)
    /// Proof: `ParaScheduler::AvailabilityCores` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::ActiveValidatorKeys` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorKeys` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Babe::AuthorVrfRandomness` (r:1 w:0)
    /// Proof: `Babe::AuthorVrfRandomness` (`max_values`: Some(1), `max_size`: Some(33), added: 528, mode: `MaxEncodedLen`)
    /// Storage: `ParaSessionInfo::Sessions` (r:1 w:0)
    /// Proof: `ParaSessionInfo::Sessions` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Disputes` (r:1 w:1)
    /// Proof: `ParasDisputes::Disputes` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::BackersOnDisputes` (r:1 w:1)
    /// Proof: `ParasDisputes::BackersOnDisputes` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Included` (r:1 w:1)
    /// Proof: `ParasDisputes::Included` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaSessionInfo::AccountKeys` (r:1 w:0)
    /// Proof: `ParaSessionInfo::AccountKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Session::Validators` (r:1 w:0)
    /// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Staking::ActiveEra` (r:1 w:0)
    /// Proof: `Staking::ActiveEra` (`max_values`: Some(1), `max_size`: Some(13), added: 508, mode: `MaxEncodedLen`)
    /// Storage: `Staking::ErasRewardPoints` (r:1 w:1)
    /// Proof: `Staking::ErasRewardPoints` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaInherent::OnChainVotes` (r:1 w:1)
    /// Proof: `ParaInherent::OnChainVotes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Frozen` (r:1 w:0)
    /// Proof: `ParasDisputes::Frozen` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaInclusion::V1` (r:2 w:1)
    /// Proof: `ParaInclusion::V1` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Dmp::DownwardMessageQueues` (r:1 w:1)
    /// Proof: `Dmp::DownwardMessageQueues` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Dmp::DeliveryFeeFactor` (r:1 w:1)
    /// Proof: `Dmp::DeliveryFeeFactor` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Hrmp::HrmpChannelDigests` (r:1 w:1)
    /// Proof: `Hrmp::HrmpChannelDigests` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::FutureCodeUpgrades` (r:1 w:0)
    /// Proof: `Paras::FutureCodeUpgrades` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::SessionStartBlock` (r:1 w:0)
    /// Proof: `ParaScheduler::SessionStartBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ValidatorGroups` (r:1 w:0)
    /// Proof: `ParaScheduler::ValidatorGroups` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::Parachains` (r:1 w:0)
    /// Proof: `Paras::Parachains` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ClaimQueue` (r:1 w:1)
    /// Proof: `ParaScheduler::ClaimQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::ActiveValidatorIndices` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Session::DisabledValidators` (r:1 w:0)
    /// Proof: `Session::DisabledValidators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Hrmp::HrmpWatermarks` (r:0 w:1)
    /// Proof: `Hrmp::HrmpWatermarks` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::Heads` (r:0 w:1)
    /// Proof: `Paras::Heads` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::UpgradeGoAheadSignal` (r:0 w:1)
    /// Proof: `Paras::UpgradeGoAheadSignal` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::MostRecentContext` (r:0 w:1)
    /// Proof: `Paras::MostRecentContext` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `v` is `[10, 1024]`.
    fn enter_variable_disputes(v: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `117374`
        //  Estimated: `123314 + v * (5 ±0)`
        // Minimum execution time: 882_216_000 picoseconds.
        Weight::from_parts(416_437_625, 123314)
            // Standard Error: 6_517
            .saturating_add(Weight::from_parts(43_055_359, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(28_u64))
            .saturating_add(T::DbWeight::get().writes(15_u64))
            .saturating_add(Weight::from_parts(0, 5).saturating_mul(v.into()))
    }
    /// Storage: `ParaInherent::Included` (r:1 w:1)
    /// Proof: `ParaInherent::Included` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `System::ParentHash` (r:1 w:0)
    /// Proof: `System::ParentHash` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
    /// Storage: `ParasShared::AllowedRelayParents` (r:1 w:1)
    /// Proof: `ParasShared::AllowedRelayParents` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::CurrentSessionIndex` (r:1 w:0)
    /// Proof: `ParasShared::CurrentSessionIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::AvailabilityCores` (r:1 w:1)
    /// Proof: `ParaScheduler::AvailabilityCores` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::ActiveValidatorKeys` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorKeys` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Babe::AuthorVrfRandomness` (r:1 w:0)
    /// Proof: `Babe::AuthorVrfRandomness` (`max_values`: Some(1), `max_size`: Some(33), added: 528, mode: `MaxEncodedLen`)
    /// Storage: `ParaInherent::OnChainVotes` (r:1 w:1)
    /// Proof: `ParaInherent::OnChainVotes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Frozen` (r:1 w:0)
    /// Proof: `ParasDisputes::Frozen` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaInclusion::V1` (r:2 w:1)
    /// Proof: `ParaInclusion::V1` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::SessionStartBlock` (r:1 w:0)
    /// Proof: `ParaScheduler::SessionStartBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ValidatorGroups` (r:1 w:0)
    /// Proof: `ParaScheduler::ValidatorGroups` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::Parachains` (r:1 w:0)
    /// Proof: `Paras::Parachains` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ClaimQueue` (r:1 w:1)
    /// Proof: `ParaScheduler::ClaimQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::ActiveValidatorIndices` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Session::DisabledValidators` (r:1 w:0)
    /// Proof: `Session::DisabledValidators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    fn enter_bitfields() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `75302`
        //  Estimated: `81242`
        // Minimum execution time: 437_311_000 picoseconds.
        Weight::from_parts(447_661_000, 81242)
            .saturating_add(T::DbWeight::get().reads(17_u64))
            .saturating_add(T::DbWeight::get().writes(6_u64))
    }
    /// Storage: `ParaInherent::Included` (r:1 w:1)
    /// Proof: `ParaInherent::Included` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `System::ParentHash` (r:1 w:0)
    /// Proof: `System::ParentHash` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
    /// Storage: `ParasShared::AllowedRelayParents` (r:1 w:1)
    /// Proof: `ParasShared::AllowedRelayParents` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::CurrentSessionIndex` (r:1 w:0)
    /// Proof: `ParasShared::CurrentSessionIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::AvailabilityCores` (r:1 w:1)
    /// Proof: `ParaScheduler::AvailabilityCores` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::ActiveValidatorKeys` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorKeys` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Babe::AuthorVrfRandomness` (r:1 w:0)
    /// Proof: `Babe::AuthorVrfRandomness` (`max_values`: Some(1), `max_size`: Some(33), added: 528, mode: `MaxEncodedLen`)
    /// Storage: `ParaInherent::OnChainVotes` (r:1 w:1)
    /// Proof: `ParaInherent::OnChainVotes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Frozen` (r:1 w:0)
    /// Proof: `ParasDisputes::Frozen` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaInclusion::V1` (r:2 w:1)
    /// Proof: `ParaInclusion::V1` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaSessionInfo::AccountKeys` (r:1 w:0)
    /// Proof: `ParaSessionInfo::AccountKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Session::Validators` (r:1 w:0)
    /// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Staking::ActiveEra` (r:1 w:0)
    /// Proof: `Staking::ActiveEra` (`max_values`: Some(1), `max_size`: Some(13), added: 508, mode: `MaxEncodedLen`)
    /// Storage: `Staking::ErasRewardPoints` (r:1 w:1)
    /// Proof: `Staking::ErasRewardPoints` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Dmp::DownwardMessageQueues` (r:1 w:1)
    /// Proof: `Dmp::DownwardMessageQueues` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Dmp::DeliveryFeeFactor` (r:1 w:1)
    /// Proof: `Dmp::DeliveryFeeFactor` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Hrmp::HrmpChannelDigests` (r:1 w:1)
    /// Proof: `Hrmp::HrmpChannelDigests` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::FutureCodeUpgrades` (r:1 w:0)
    /// Proof: `Paras::FutureCodeUpgrades` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Disputes` (r:1 w:0)
    /// Proof: `ParasDisputes::Disputes` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::SessionStartBlock` (r:1 w:0)
    /// Proof: `ParaScheduler::SessionStartBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ValidatorGroups` (r:1 w:0)
    /// Proof: `ParaScheduler::ValidatorGroups` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::Parachains` (r:1 w:0)
    /// Proof: `Paras::Parachains` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ClaimQueue` (r:1 w:1)
    /// Proof: `ParaScheduler::ClaimQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::CurrentCodeHash` (r:1 w:0)
    /// Proof: `Paras::CurrentCodeHash` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::ParaLifecycles` (r:1 w:0)
    /// Proof: `Paras::ParaLifecycles` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `MessageQueue::BookStateFor` (r:1 w:0)
    /// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(55), added: 2530, mode: `MaxEncodedLen`)
    /// Storage: `ParasShared::ActiveValidatorIndices` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Session::DisabledValidators` (r:1 w:0)
    /// Proof: `Session::DisabledValidators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Included` (r:0 w:1)
    /// Proof: `ParasDisputes::Included` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Hrmp::HrmpWatermarks` (r:0 w:1)
    /// Proof: `Hrmp::HrmpWatermarks` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::Heads` (r:0 w:1)
    /// Proof: `Paras::Heads` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::UpgradeGoAheadSignal` (r:0 w:1)
    /// Proof: `Paras::UpgradeGoAheadSignal` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::MostRecentContext` (r:0 w:1)
    /// Proof: `Paras::MostRecentContext` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `v` is `[2, 5]`.
    fn enter_backed_candidates_variable(v: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `76317`
        //  Estimated: `82257`
        // Minimum execution time: 1_228_106_000 picoseconds.
        Weight::from_parts(1_183_261_172, 82257)
            // Standard Error: 307_178
            .saturating_add(Weight::from_parts(39_626_828, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(29_u64))
            .saturating_add(T::DbWeight::get().writes(15_u64))
    }
    /// Storage: `ParaInherent::Included` (r:1 w:1)
    /// Proof: `ParaInherent::Included` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `System::ParentHash` (r:1 w:0)
    /// Proof: `System::ParentHash` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
    /// Storage: `ParasShared::AllowedRelayParents` (r:1 w:1)
    /// Proof: `ParasShared::AllowedRelayParents` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::CurrentSessionIndex` (r:1 w:0)
    /// Proof: `ParasShared::CurrentSessionIndex` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::AvailabilityCores` (r:1 w:1)
    /// Proof: `ParaScheduler::AvailabilityCores` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasShared::ActiveValidatorKeys` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorKeys` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Babe::AuthorVrfRandomness` (r:1 w:0)
    /// Proof: `Babe::AuthorVrfRandomness` (`max_values`: Some(1), `max_size`: Some(33), added: 528, mode: `MaxEncodedLen`)
    /// Storage: `ParaInherent::OnChainVotes` (r:1 w:1)
    /// Proof: `ParaInherent::OnChainVotes` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Frozen` (r:1 w:0)
    /// Proof: `ParasDisputes::Frozen` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaInclusion::V1` (r:2 w:1)
    /// Proof: `ParaInclusion::V1` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaSessionInfo::AccountKeys` (r:1 w:0)
    /// Proof: `ParaSessionInfo::AccountKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Session::Validators` (r:1 w:0)
    /// Proof: `Session::Validators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Staking::ActiveEra` (r:1 w:0)
    /// Proof: `Staking::ActiveEra` (`max_values`: Some(1), `max_size`: Some(13), added: 508, mode: `MaxEncodedLen`)
    /// Storage: `Staking::ErasRewardPoints` (r:1 w:1)
    /// Proof: `Staking::ErasRewardPoints` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Dmp::DownwardMessageQueues` (r:1 w:1)
    /// Proof: `Dmp::DownwardMessageQueues` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Dmp::DeliveryFeeFactor` (r:1 w:1)
    /// Proof: `Dmp::DeliveryFeeFactor` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Hrmp::HrmpChannelDigests` (r:1 w:1)
    /// Proof: `Hrmp::HrmpChannelDigests` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::FutureCodeUpgrades` (r:1 w:0)
    /// Proof: `Paras::FutureCodeUpgrades` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Disputes` (r:1 w:0)
    /// Proof: `ParasDisputes::Disputes` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::SessionStartBlock` (r:1 w:0)
    /// Proof: `ParaScheduler::SessionStartBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ValidatorGroups` (r:1 w:0)
    /// Proof: `ParaScheduler::ValidatorGroups` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::Parachains` (r:1 w:0)
    /// Proof: `Paras::Parachains` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParaScheduler::ClaimQueue` (r:1 w:1)
    /// Proof: `ParaScheduler::ClaimQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::CurrentCodeHash` (r:1 w:0)
    /// Proof: `Paras::CurrentCodeHash` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::FutureCodeHash` (r:1 w:0)
    /// Proof: `Paras::FutureCodeHash` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::UpgradeRestrictionSignal` (r:1 w:0)
    /// Proof: `Paras::UpgradeRestrictionSignal` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::ParaLifecycles` (r:1 w:0)
    /// Proof: `Paras::ParaLifecycles` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `MessageQueue::BookStateFor` (r:1 w:0)
    /// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(55), added: 2530, mode: `MaxEncodedLen`)
    /// Storage: `ParasShared::ActiveValidatorIndices` (r:1 w:0)
    /// Proof: `ParasShared::ActiveValidatorIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `Session::DisabledValidators` (r:1 w:0)
    /// Proof: `Session::DisabledValidators` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `ParasDisputes::Included` (r:0 w:1)
    /// Proof: `ParasDisputes::Included` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Hrmp::HrmpWatermarks` (r:0 w:1)
    /// Proof: `Hrmp::HrmpWatermarks` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::Heads` (r:0 w:1)
    /// Proof: `Paras::Heads` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::UpgradeGoAheadSignal` (r:0 w:1)
    /// Proof: `Paras::UpgradeGoAheadSignal` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `Paras::MostRecentContext` (r:0 w:1)
    /// Proof: `Paras::MostRecentContext` (`max_values`: None, `max_size`: None, mode: `Measured`)
    fn enter_backed_candidate_code_upgrade() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `76344`
        //  Estimated: `82284`
        // Minimum execution time: 64_960_571_000 picoseconds.
        Weight::from_parts(66_354_359_000, 82284)
            .saturating_add(T::DbWeight::get().reads(31_u64))
            .saturating_add(T::DbWeight::get().writes(15_u64))
    }
}
