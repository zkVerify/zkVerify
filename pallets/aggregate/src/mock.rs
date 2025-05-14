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

use crate::{
    data::{Delivery, Reserve},
    AggregationSize, BalanceOf, CallOf, ComputePublisherTip, Domains,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    derive_impl,
    dispatch::PostDispatchInfo,
    parameter_types,
    traits::{Consideration, EnsureOrigin, EstimateCallFee, Footprint},
    weights::{RuntimeDbWeight, Weight},
};
use frame_system::RawOrigin;
use hp_dispatch::{
    BoundedStateMachine, Destination, DispatchAggregation, HyperbridgeDispatchParameters,
};
use scale_info::TypeInfo;
use sp_core::{ConstU128, ConstU32, H160, H256};
use sp_runtime::{traits::IdentityLookup, BuildStorage, DispatchResult, Perbill};

parameter_types! {
    pub const MaxAggregationSize: AggregationSize = 64;
    pub const MaxPendingPublishQueueSize: u32 = 16;
}

pub const ESTIMATED_FEE: u32 = 6400;
pub const FEE_PERCENT_CORRECTION: u32 = 10;
pub const ESTIMATED_FEE_CORRECTED: u32 = (ESTIMATED_FEE * (100 + FEE_PERCENT_CORRECTION)) / 100;

pub type Balance = u128;
pub type AccountId = u64;
pub type Origin = RawOrigin<AccountId>;

pub const DOMAIN_ID: u32 = 51;
pub const DOMAIN: Option<u32> = Some(DOMAIN_ID);
pub const DOMAIN_ID_NONE: u32 = 666;
pub const DOMAIN_NONE: Option<u32> = Some(DOMAIN_ID_NONE);
pub const DOMAIN_SIZE: AggregationSize = 32;
pub const DOMAIN_QUEUE_SIZE: u32 = 16;
pub const DOMAIN_FEE: Balance = (ESTIMATED_FEE_CORRECTED / DOMAIN_SIZE as u32) as Balance;
pub const NOT_REGISTERED_DOMAIN_ID: u32 = 911;
pub const NOT_REGISTERED_DOMAIN: Option<u32> = Some(NOT_REGISTERED_DOMAIN_ID);
pub const NO_DOMAIN_FEE_FUND_USER: AccountId = 999;
pub const NO_DELIVERY_FUND_USER: AccountId = 888;
pub const NO_FUND_USER: AccountId = 777;
pub const PUBLISHER_USER: AccountId = 100;
pub const USER_1: AccountId = 42;
pub const USER_2: AccountId = 24;
pub const USER_DOMAIN_1: AccountId = 42_000;
pub const USER_DOMAIN_2: AccountId = 24_000;
pub const USER_DOMAIN_ERROR_NEW: AccountId = 99_000;
pub const USER_DOMAIN_ERROR_DROP: AccountId = 100_000;
pub const USER_DELIVERY_OWNER: AccountId = 33_333;
pub const ROOT_USER: AccountId = 666;

pub static USERS: &[(AccountId, Balance)] = &[
    (USER_1, 42_000_000_000),
    (USER_2, 24_000_000_000),
    (USER_DOMAIN_1, 100_000_000_000),
    (USER_DOMAIN_2, 200_000_000_000),
    (PUBLISHER_USER, 1_000_000_000),
    (NO_DOMAIN_FEE_FUND_USER, (DOMAIN_FEE / 2) as u128),
    (NO_DELIVERY_FUND_USER, DOMAIN_FEE + 1 as u128),
    (NO_FUND_USER, 1_u128),
];

pub struct MockWeightInfo;

impl MockWeightInfo {
    pub const OPV_REF_TIME: u64 = 1_000_000_042;
    pub const OPV_PROOF_SIZE: u64 = 1_000_000_024;
    pub const AGG_REF_TIME: u64 = 42;
    pub const AGG_PROOF_SIZE: u64 = 24;
    pub const REG_REF_TIME: u64 = 142;
    pub const REG_PROOF_SIZE: u64 = 124;
    pub const UNR_REF_TIME: u64 = 242;
    pub const UNR_PROOF_SIZE: u64 = 224;
    pub const HOLD_REF_TIME: u64 = 342;
    pub const HOLD_PROOF_SIZE: u64 = 324;
    pub const SET_PRICE_REF_TIME: u64 = 442;
    pub const SET_PRICE_PROOF_SIZE: u64 = 424;
    pub const AGG_NO_DOMAIN_REF_TIME: u64 = 1_000_042;
    pub const AGG_NO_DOMAIN_PROOF_SIZE: u64 = 1_000_024;
    pub const AGG_NO_ID_REF_TIME: u64 = 1_001_042;
    pub const AGG_NO_ID_PROOF_SIZE: u64 = 1_001_024;
}

impl crate::WeightInfo for MockWeightInfo {
    fn on_proof_verified() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(Self::OPV_REF_TIME, Self::OPV_REF_TIME)
    }

    fn aggregate(n: u32) -> Weight {
        let variable = 1000 * n as u64;
        Weight::from_parts(
            Self::AGG_REF_TIME + variable,
            Self::AGG_PROOF_SIZE + variable,
        )
    }

    fn aggregate_on_invalid_domain() -> Weight {
        Weight::from_parts(Self::AGG_NO_DOMAIN_REF_TIME, Self::AGG_NO_DOMAIN_PROOF_SIZE)
    }

    fn aggregate_on_invalid_id() -> Weight {
        Weight::from_parts(Self::AGG_NO_ID_REF_TIME, Self::AGG_NO_ID_PROOF_SIZE)
    }

    fn register_domain() -> Weight {
        Weight::from_parts(Self::REG_REF_TIME, Self::REG_PROOF_SIZE)
    }

    fn unregister_domain() -> Weight {
        Weight::from_parts(Self::UNR_REF_TIME, Self::UNR_PROOF_SIZE)
    }

    fn hold_domain() -> Weight {
        Weight::from_parts(Self::HOLD_REF_TIME, Self::HOLD_PROOF_SIZE)
    }

    fn set_total_delivery_fee() -> Weight {
        Weight::from_parts(Self::SET_PRICE_REF_TIME, Self::SET_PRICE_PROOF_SIZE)
    }
}

parameter_types! {
    pub const MockDbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 4_200_000,
       write: 2_400_000,
    };
}

pub struct PercentComputeFeeFor;

impl ComputePublisherTip<Balance> for PercentComputeFeeFor {
    fn compute_tip(estimated: Balance) -> Option<Balance> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockDispatchAggregation {
    pub domain_id: u32,
    pub aggregation_id: u64,
    pub aggregation: H256,
    pub destination: Destination,
    pub fee: u128,
    pub delivery_owner: AccountId,
}

impl MockDispatchAggregation {
    pub const NONE_REF_TIME: u64 = 42;
    pub const NONE_PROOF_SIZE: u64 = 24;
    pub const HB_REF_TIME: u64 = 4242;
    pub const HB_PROOF_SIZE: u64 = 2424;

    thread_local! {
        pub static CALLS: RefCell<VecDeque<MockDispatchAggregation>> = RefCell::new(Default::default());
    }

    thread_local! {
        pub static RETURN: RefCell<DispatchResult> = const { RefCell::new(Ok(())) };
    }

    pub fn pop() -> Option<Self> {
        Self::CALLS.with_borrow_mut(|calls| calls.pop_front())
    }

    fn push(self) {
        Self::CALLS.with_borrow_mut(|calls| calls.push_back(self))
    }

    pub fn set_return(dispatch_result: DispatchResult) {
        let _ = Self::RETURN.replace(dispatch_result);
    }

    fn return_value() -> DispatchResult {
        Self::RETURN.with_borrow(|r| *r)
    }

    pub fn none_weight() -> Weight {
        Weight::from_parts(Self::NONE_REF_TIME, Self::NONE_PROOF_SIZE)
    }

    pub fn hyperbridge_weight() -> Weight {
        Weight::from_parts(Self::HB_REF_TIME, Self::HB_PROOF_SIZE)
    }

    pub fn max_weight() -> Weight {
        Self::hyperbridge_weight()
    }
}

impl DispatchAggregation<Balance, AccountId> for MockDispatchAggregation {
    fn dispatch_aggregation(
        domain_id: u32,
        aggregation_id: u64,
        aggregation: H256,
        destination: Destination,
        fee: Balance,
        delivery_owner: AccountId,
    ) -> DispatchResult {
        MockDispatchAggregation {
            domain_id,
            aggregation_id,
            aggregation,
            destination,
            fee,
            delivery_owner,
        }
        .push();
        MockDispatchAggregation::return_value()
    }

    fn max_weight() -> Weight {
        Self::max_weight()
    }

    fn dispatch_weight(destination: &Destination) -> Weight {
        match destination {
            Destination::None => Self::none_weight(),
            Destination::Hyperbridge(_) => Self::hyperbridge_weight(),
        }
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

impl Consideration<AccountId, Footprint> for MockConsideration {
    fn new(who: &AccountId, new: Footprint) -> Result<Self, sp_runtime::DispatchError> {
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
        Self::push(self, *who);
        if who == &USER_DOMAIN_ERROR_DROP {
            Err(sp_runtime::DispatchError::from("User Domain Error Drop"))?
        }
        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn ensure_successful(_: &u64, _: Footprint) {
        unimplemented!("Not needed by now")
    }
}

pub struct MockEstimateCallFeeImpl<const V: u32>;

#[derive(Debug, Clone)]
pub struct EstimateCallData {
    pub call: CallOf<Test>,
    pub post_info: PostDispatchInfo,
}

impl<const V: u32> MockEstimateCallFeeImpl<V> {
    thread_local! {
        pub static QUEUE: RefCell<VecDeque<EstimateCallData>> = RefCell::new(Default::default());
    }

    fn push(data: EstimateCallData) {
        Self::QUEUE.with_borrow_mut(|q| q.push_back(data));
    }

    pub fn pop() -> Option<EstimateCallData> {
        Self::QUEUE.with_borrow_mut(|q| q.pop_front())
    }
}

impl<const V: u32> EstimateCallFee<CallOf<Test>, BalanceOf<Test>> for MockEstimateCallFeeImpl<V> {
    fn estimate_call_fee(
        call: &CallOf<Test>,
        post_info: frame_support::dispatch::PostDispatchInfo,
    ) -> BalanceOf<Test> {
        Self::push(EstimateCallData {
            call: call.clone(),
            post_info,
        });
        V as BalanceOf<Test>
    }
}

pub type MockEstimateCallFee = MockEstimateCallFeeImpl<ESTIMATED_FEE>;

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;

    type RuntimeHoldReason = RuntimeHoldReason;

    type WeightInfo = MockWeightInfo;

    type AggregationSize = MaxAggregationSize;

    type MaxPendingPublishQueueSize = MaxPendingPublishQueueSize;

    type Hold = Balances;

    type Consideration = MockConsideration;

    type EstimateCallFee = MockEstimateCallFee;

    type ComputePublisherTip = PercentComputeFeeFor;

    type ManagerOrigin = MockManager;

    #[cfg(feature = "runtime-benchmarks")]
    const AGGREGATION_SIZE: u32 = MaxAggregationSize::get() as u32;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
    type DispatchAggregation = MockDispatchAggregation;
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

pub fn hyperbridge_destination() -> Destination {
    Destination::Hyperbridge(HyperbridgeDispatchParameters {
        destination_chain: BoundedStateMachine::Evm(11155111),
        destination_module: H160::default(),
        timeout: 100,
    })
}

pub fn none_destination() -> Destination {
    Destination::None
}

pub fn none_delivering() -> Delivery<Balance> {
    none_destination().into()
}

pub fn priced_none_delivering(delivery_fee: Balance, owner_tip: Balance) -> Delivery<Balance> {
    Delivery::new(none_destination(), delivery_fee, owner_tip)
}

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

impl crate::Domain<Test> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        id: u32,
        owner: crate::data::User<crate::AccountOf<Test>>,
        next_aggregation_id: u64,
        max_aggregation_size: AggregationSize,
        publish_queue_size: u32,
        aggregate_rules: crate::data::AggregateSecurityRules,
        ticket: Option<crate::TicketOf<Test>>,
        delivery: crate::data::DeliveryParams<crate::AccountOf<Test>, crate::BalanceOf<Test>>,
    ) -> Self {
        Self::try_create(
            id,
            owner,
            next_aggregation_id,
            max_aggregation_size,
            publish_queue_size,
            aggregate_rules,
            ticket,
            delivery,
        )
        .unwrap()
    }
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
                DOMAIN_SIZE,
                DOMAIN_QUEUE_SIZE,
                crate::data::AggregateSecurityRules::Untrusted,
                None,
                hyperbridge_destination().into(),
            ),
        );
        Domains::<Test>::insert(
            DOMAIN_ID_NONE,
            crate::Domain::<Test>::create(
                DOMAIN_ID_NONE,
                USER_DOMAIN_1.into(),
                1,
                DOMAIN_SIZE,
                DOMAIN_QUEUE_SIZE,
                crate::data::AggregateSecurityRules::Untrusted,
                None,
                none_delivering().into(),
            ),
        );
    });
    ext
}

impl From<Destination> for crate::data::Delivery<Balance> {
    fn from(destination: Destination) -> Self {
        Delivery::new(destination, 0_u128, 0_u128)
    }
}

impl From<crate::data::Delivery<Balance>> for crate::data::DeliveryParams<AccountId, Balance> {
    fn from(delivery: crate::data::Delivery<Balance>) -> Self {
        Self::new(USER_DELIVERY_OWNER, delivery)
    }
}

impl From<Destination> for crate::data::DeliveryParams<AccountId, Balance> {
    fn from(destination: Destination) -> Self {
        crate::data::Delivery::<Balance>::from(destination).into()
    }
}

impl Default for Reserve<Balance> {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
