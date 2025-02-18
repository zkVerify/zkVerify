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

use frame_support::{derive_impl, parameter_types};
use frame_system as system;
use hp_poe::MaxStorageAttestations;
use sp_core::{ConstU32, ConstU64};
use sp_runtime::{traits::IdentityLookup, BuildStorage};

// Timestamp
impl pallet_timestamp::Config for Test {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

// Poe
pub const MILLISECS_PER_PROOF_ROOT_PUBLISHING: u64 = 6000;
pub const MIN_PROOFS_FOR_ROOT_PUBLISHING: u32 = 2;
pub const MAX_STORAGE_ATTESTATIONS: u64 = 10;

parameter_types! {
    pub MaxAttestations: MaxStorageAttestations = MaxStorageAttestations(MAX_STORAGE_ATTESTATIONS);
}

pub struct MockWeightInfo;

impl MockWeightInfo {
    pub const REF_TIME: u64 = 42;
    pub const PROOF_SIZE: u64 = 24;
}

impl crate::weight::WeightInfo for MockWeightInfo {
    fn publish_attestation() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(Self::REF_TIME, Self::PROOF_SIZE)
    }

    fn on_proof_verified() -> frame_support::weights::Weight {
        frame_support::weights::Weight::default()
    }
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MinProofsForPublishing = ConstU32<MIN_PROOFS_FOR_ROOT_PUBLISHING>;
    type MaxElapsedTimeMs = ConstU64<MILLISECS_PER_PROOF_ROOT_PUBLISHING>;
    type MaxStorageAttestations = MaxAttestations;
    type WeightInfo = MockWeightInfo;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Poe: crate::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call},
    }
);

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl system::Config for Test {
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = frame_system::mocking::MockBlockU32<Test>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = sp_io::TestExternalities::from(
        frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap(),
    );
    ext.execute_with(|| System::set_block_number(1));
    ext
}
