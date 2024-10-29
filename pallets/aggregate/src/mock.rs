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

use core::cell::RefCell;
use std::collections::VecDeque;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{Consideration, EnsureOrigin};
use frame_support::weights::RuntimeDbWeight;
use frame_support::{derive_impl, parameter_types};
use frame_system::RawOrigin;
use scale_info::TypeInfo;
use sp_core::{ConstU128, ConstU32};
use sp_runtime::{traits::IdentityLookup, BuildStorage, Perbill};

use crate::{ComputeFeeFor, Domains};

parameter_types! {
    pub const AttestationSize: u32 = 32;
    pub const MaxPendingPublishQueueSize: u32 = 16;
}

pub const FEE_PER_STATEMENT: u32 = 100;
pub const FEE_PERCENT_CORRECTION: u32 = 10;
pub const FEE_PER_STATEMENT_CORRECTED: u32 =
    (FEE_PER_STATEMENT * (100 + FEE_PERCENT_CORRECTION)) / 100;
pub const ESTIMATED_FEE: u32 = FEE_PER_STATEMENT * AttestationSize::get();
pub const ESTIMATED_FEE_CORRECTED: u32 = FEE_PER_STATEMENT_CORRECTED * AttestationSize::get();

pub type Balance = u128;
pub type AccountId = u64;
pub type Origin = RawOrigin<AccountId>;

pub const DOMAIN_ID: u32 = 51;
pub const DOMAIN: Option<u32> = Some(DOMAIN_ID);
pub const NOT_REGISTERED_DOMAIN_ID: u32 = 911;
pub const NOT_REGISTERED_DOMAIN: Option<u32> = Some(NOT_REGISTERED_DOMAIN_ID);
pub const NUM_TEST_ACCOUNTS: usize = 6;
pub const NO_FOUND_USER: AccountId = 999;
pub const PUBLISHER_USER: AccountId = 100;
pub const USER_1: AccountId = 42;
pub const USER_2: AccountId = 24;
pub const USER_DOMAIN_1: AccountId = 42_000;
pub const USER_DOMAIN_2: AccountId = 24_000;
pub const USER_DOMAIN_ERROR_NEW: AccountId = 99_000;
pub const USER_DOMAIN_ERROR_DROP: AccountId = 100_000;
pub const ROOT_USER: AccountId = 666;

pub static USERS: [(AccountId, Balance); NUM_TEST_ACCOUNTS] = [
    (USER_1, 42_000_000_000),
    (USER_2, 24_000_000_000),
    (USER_DOMAIN_1, 100_000_000_000),
    (USER_DOMAIN_2, 200_000_000_000),
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

    fn register_domain() -> frame_support::weights::Weight {
        todo!()
    }

    fn unregister_domain() -> frame_support::weights::Weight {
        todo!()
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

pub struct MockManager;
impl<O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>> EnsureOrigin<O>
    for MockManager
{
    type Success = ();

    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Signed(ROOT_USER) => Ok(()),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(RawOrigin::Signed(ROOT_USER)))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct MockConsideration {
    pub who: AccountId,
    pub count: u64,
    pub size: u64,
}

impl MockConsideration {
    thread_local! {
        pub static QUEUE: RefCell<VecDeque<(AccountId, MockConsideration)>> = RefCell::new(Default::default());
    }

    fn push(self, id: AccountId) {
        Self::QUEUE.with_borrow_mut(|q| q.push_back((id, self)));
    }

    pub fn pop() -> Option<(AccountId, Self)> {
        Self::QUEUE.with_borrow_mut(|q| q.pop_front())
    }
}

impl Consideration<AccountId> for MockConsideration {
    fn new(
        who: &AccountId,
        new: frame_support::traits::Footprint,
    ) -> Result<Self, sp_runtime::DispatchError> {
        if who == &USER_DOMAIN_ERROR_NEW {
            Err(sp_runtime::DispatchError::from("User Domain Error New"))?
        }
        Ok(Self {
            who: *who,
            count: new.count,
            size: new.size,
        })
    }

    fn update(
        self,
        _who: &AccountId,
        _new: frame_support::traits::Footprint,
    ) -> Result<Self, sp_runtime::DispatchError> {
        unimplemented!("We don't support it by now")
    }

    fn drop(self, who: &AccountId) -> Result<(), sp_runtime::DispatchError> {
        Self::push(self, who.clone());
        if who == &USER_DOMAIN_ERROR_DROP {
            Err(sp_runtime::DispatchError::from("User Domain Error Drop"))?
        }
        Ok(())
    }
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;

    type RuntimeHoldReason = RuntimeHoldReason;

    type WeightInfo = MockWeightInfo;

    type AggregationSize = AttestationSize;

    type MaxPendingPublishQueueSize = MaxPendingPublishQueueSize;

    type Hold = Balances;

    type Consideration = MockConsideration;

    type EstimateCallFee = frame_support::traits::ConstU32<ESTIMATED_FEE>;

    type ComputeFeeFor = PercentComputeFeeFor;

    type ManagerOrigin = MockManager;
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
                USER_DOMAIN_1.into(),
                1,
                <Test as crate::Config>::AggregationSize::get(),
                <Test as crate::Config>::MaxPendingPublishQueueSize::get(),
                None,
            ),
        );
    });
    ext
}
