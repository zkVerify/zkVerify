// Copyright 2024, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.
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

//! Default weights for the Babe Pallet
//! This file was not auto-generated.

use core::marker::PhantomData;
use frame_support::weights::{
    constants::{RocksDbWeight as DbWeight, WEIGHT_REF_TIME_PER_MICROS, WEIGHT_REF_TIME_PER_NANOS},
    Weight,
};

/// Weights for `pallet_babe` using the ZKVerify node and recommended hardware.
/// Weight for report_equivocation is manually computed as the sum of:
/// - pallet_session::check_membership_proof
/// - pallet_babe::check_equivocation_report
/// - pallet_offences::report_offence_babe
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_babe::WeightInfo for ZKVWeight<T> {
    fn plan_config_change() -> Weight {
        DbWeight::get().writes(1)
    }

    fn report_equivocation(validator_count: u32, max_nominators_per_validator: u32) -> Weight {
        // we take the validator set count from the membership proof to
        // calculate the weight but we set a floor of 32 validators.
        let validator_count = validator_count.max(32) as u64;

        // checking membership proof
        Weight::from_parts(25u64 * WEIGHT_REF_TIME_PER_MICROS, 0)
            .saturating_add(
                Weight::from_parts(155u64 * WEIGHT_REF_TIME_PER_NANOS, 0)
                    .saturating_mul(validator_count),
            )
            .saturating_add(DbWeight::get().reads(5))
            // check equivocation proof
            .saturating_add(Weight::from_parts(120u64 * WEIGHT_REF_TIME_PER_MICROS, 0))
            // report offence
            .saturating_add(Weight::from_parts(110u64 * WEIGHT_REF_TIME_PER_MICROS, 0))
            .saturating_add(Weight::from_parts(
                45u64 * WEIGHT_REF_TIME_PER_MICROS * max_nominators_per_validator as u64,
                0,
            ))
            .saturating_add(DbWeight::get().reads(20 + 8 * max_nominators_per_validator as u64))
            .saturating_add(DbWeight::get().writes(12 + 6 * max_nominators_per_validator as u64))
    }
}
