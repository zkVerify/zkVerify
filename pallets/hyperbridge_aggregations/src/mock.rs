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

use super::*;
use frame_support::pallet_prelude::ConstU32;
use frame_support::traits::ConstU128;
use frame_support::weights::RuntimeDbWeight;
use frame_support::{construct_runtime, derive_impl, parameter_types, traits::ConstU64};
use ismp::host::StateMachine;
use ismp::router::IsmpRouter;
use sp_runtime::{traits::IdentityLookup, BuildStorage};

pub type Balance = u128;
pub type AccountId = u64;

parameter_types! {
    pub const MockDbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 4_200_000,
        write: 2_400_000,
    };
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = frame_system::mocking::MockBlockU32<Test>;
    type AccountData = pallet_balances::AccountData<Balance>;
    type DbWeight = MockDbWeight;
}

impl pallet_timestamp::Config for Test {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
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

pub struct FakeWeightInfo;

impl ismp_grandpa::WeightInfo for FakeWeightInfo {
    fn add_state_machines(_n: u32) -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(42, 24)
    }

    fn remove_state_machines(_n: u32) -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(4242, 2424)
    }
}

impl ismp_grandpa::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type IsmpHost = Ismp;
    type WeightInfo = FakeWeightInfo;
}

parameter_types! {
    pub const Coprocessor: Option<StateMachine> = Some(StateMachine::Kusama(4009));
    pub const HostStateMachine: StateMachine = StateMachine::Substrate(*b"zkv_");
}

impl pallet_ismp::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type HostStateMachine = HostStateMachine;
    type TimestampProvider = Timestamp;
    type Currency = Balances;
    type Balance = Balance;
    type Router = ModuleRouter;
    type Coprocessor = Coprocessor;
    type ConsensusClients = (ismp_grandpa::consensus::GrandpaConsensusClient<Test>,);
    type WeightProvider = ();
    type OffchainDB = ();
}

#[derive(Default)]
pub struct ModuleRouter;
impl IsmpRouter for ModuleRouter {
    fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
        match id.as_slice() {
            id if id == ZKV_MODULE_ID.to_bytes().as_slice() => {
                Ok(Box::new(crate::Pallet::<Test>::default()))
            }
            _ => Err(ismp::Error::ModuleNotFound(id))?,
        }
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type IsmpDispatcher = pallet_ismp::Pallet<Test>;
    type WeightInfo = ();
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        HyperbridgeAggregations: crate,
        Timestamp: pallet_timestamp,
        Ismp:pallet_ismp::{Pallet, Call, Storage, Event<T>} = 10,
        Balances: pallet_balances,
        IsmpGrandpa: ismp_grandpa,
    }
);

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1000);
    });
    ext
}
