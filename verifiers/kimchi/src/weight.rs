// Copyright 2026, Horizen Labs, Inc.
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

use frame_support::weights::{constants::RocksDbWeight, Weight};

/// Weight functions needed for `pallet_kimchi_verifier`.
pub trait WeightInfo {
    fn verify_proof() -> Weight;
    fn get_vk() -> Weight;
    fn validate_vk() -> Weight;
    fn compute_statement_hash() -> Weight;
    fn register_vk() -> Weight;
    fn unregister_vk() -> Weight;
}

impl WeightInfo for () {
    fn verify_proof() -> Weight {
        // Static upper bound for the Kimchi verification formula:
        // 176ms four-chunk base + 1024 public inputs * 50us.
        Weight::from_parts(227_200_000_000, 0)
    }

    fn get_vk() -> Weight {
        Weight::from_parts(5_000_000, 69_046).saturating_add(RocksDbWeight::get().reads(1_u64))
    }

    fn validate_vk() -> Weight {
        Weight::from_parts(57_302_000_000, 0)
    }

    fn compute_statement_hash() -> Weight {
        Weight::from_parts(117_000_000, 0)
    }

    fn register_vk() -> Weight {
        Weight::from_parts(57_316_000_000, 69_046)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }

    fn unregister_vk() -> Weight {
        Weight::from_parts(34_000_000, 69_046)
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
}
