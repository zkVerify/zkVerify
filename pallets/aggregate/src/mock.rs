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
#![cfg(test)]

use frame_support::weights::RuntimeDbWeight;
use frame_support::{derive_impl, parameter_types};
use sp_core::{ConstU128, ConstU32, Get, H256};
use sp_runtime::{traits::IdentityLookup, BuildStorage, Perbill};

use crate::{AccountOf, Aggregation, BalanceOf, ComputeFeeFor, Domains, StatementEntry};

parameter_types! {
    pub const AttestationSize: u32 = 4;
    pub const MaxPublishedPerBlock: u32 = 3;
    pub const MaxPendingPublishQueueSize: u32 = 6;
}

pub const FEE_PER_STATEMENT: u32 = 100;
pub const FEE_PERCENT_CORRECTION: u32 = 10;
pub const FEE_PER_STATEMENT_CORRECTED: u32 =
    (FEE_PER_STATEMENT * (100 + FEE_PERCENT_CORRECTION)) / 100;
pub const ESTIMATED_FEE: u32 = FEE_PER_STATEMENT * AttestationSize::get();
pub const ESTIMATED_FEE_CORRECTED: u32 = FEE_PER_STATEMENT_CORRECTED * AttestationSize::get();

pub type Balance = u128;
pub type AccountId = u64;

pub const DOMAIN_ID: u32 = 51;
pub const DOMAIN: Option<u32> = Some(DOMAIN_ID);
pub const NOT_REGISTERED_DOMAIN_ID: u32 = 911;
pub const NOT_REGISTERED_DOMAIN: Option<u32> = Some(NOT_REGISTERED_DOMAIN_ID);
pub const NUM_TEST_ACCOUNTS: usize = 4;
pub const NO_FOUND_USER: AccountId = 999;
pub const PUBLISHER_USER: AccountId = 100;
pub const USER_1: AccountId = 42;
pub const USER_2: AccountId = 24;

pub static USERS: [(AccountId, Balance); NUM_TEST_ACCOUNTS] = [
    (USER_1, 42_000_000_000),
    (USER_2, 24_000_000_000),
    (PUBLISHER_USER, 1_000_000_000),
    (NO_FOUND_USER, (FEE_PER_STATEMENT / 2) as u128),
];

pub struct MockWeightInfo;

impl MockWeightInfo {
    pub const REF_TIME: u64 = 42;
    pub const PROOF_SIZE: u64 = 24;
}

impl crate::WeightInfo for MockWeightInfo {
    fn aggregate() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(Self::REF_TIME, Self::PROOF_SIZE)
    }
}

parameter_types! {
    pub const MockDbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 4_200_000,
       write: 2_400_000,
    };
}

pub struct PercentComputeFeeFor;

impl ComputeFeeFor<Balance> for PercentComputeFeeFor {
    fn compute_fee(estimated: Balance) -> Option<Balance> {
        Some(Perbill::from_percent(FEE_PERCENT_CORRECTION) * estimated)
    }
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = MockWeightInfo;

    type AggregationSize = AttestationSize;

    type MaxPendingPublishQueueSize = MaxPendingPublishQueueSize;

    type Currency = Balances;

    type EstimateCallFee = frame_support::traits::ConstU32<ESTIMATED_FEE>;

    type ComputeFeeFor = PercentComputeFeeFor;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Aggregate: crate,
    }
);

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = frame_system::mocking::MockBlockU32<Test>;
    type AccountData = pallet_balances::AccountData<Balance>;
    type DbWeight = MockDbWeight;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = ();
}

impl Aggregation<Test> {
    pub(crate) fn add_statement(
        &mut self,
        account: AccountOf<Test>,
        reserve: BalanceOf<Test>,
        statement: H256,
    ) {
        self.statements
            .try_push(StatementEntry::new(account, reserve, statement))
            .unwrap();
    }
}

pub fn db_weights() -> RuntimeDbWeight {
    <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get()
}

// Build genesis storage according to the mock runtime.
pub fn test() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: USERS.to_vec(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::from(t);

    ext.execute_with(|| {
        System::set_block_number(1);
        Domains::<Test>::insert(
            DOMAIN_ID,
            crate::Domain::<Test>::create(
                DOMAIN_ID,
                1,
                <Test as crate::Config>::AggregationSize::get(),
                <Test as crate::Config>::MaxPendingPublishQueueSize::get(),
            ),
        );
    });
    ext
}
