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

use std::{collections::BTreeMap, sync::LazyLock};

use frame_support::{
    derive_impl, parameter_types,
    traits::{EitherOfDiverse, EnsureOrigin},
    weights::RuntimeDbWeight,
    PalletId,
};
use frame_system::{EnsureRoot, RawOrigin};
use sp_core::ConstU128;
use sp_runtime::{traits::IdentityLookup, BuildStorage};

pub type Balance = u128;
pub type AccountId = u64;
pub type Origin = RawOrigin<AccountId>;

pub const EXISTENTIAL_DEPOSIT: Balance = 1;

pub const USER_1: AccountId = 42;
pub const USER_1_AMOUNT: Balance = 42_000_000_000;
pub const USER_2: AccountId = 24;
pub const USER_2_AMOUNT: Balance = 24_000_000_000;
pub const USER_3: AccountId = 42_000;
pub const USER_3_AMOUNT: Balance = 100_000_000_000;
pub const USER_4: AccountId = 24_000;
pub const USER_4_AMOUNT: Balance = 200_000_000_000;
pub const USER_5: AccountId = 99_000;
pub const USER_5_AMOUNT: Balance = 300_000_000;
pub const USER_6: AccountId = 33_333;
pub const USER_6_AMOUNT: Balance = 50_000_000_000;
pub const NON_BENEFICIARY: AccountId = 6;

pub const MANAGER_USER: AccountId = 666;

pub static GENESIS_BENEFICIARIES: [(AccountId, Balance); 3] = [
    (USER_1, USER_1_AMOUNT),
    (USER_2, USER_2_AMOUNT),
    (USER_3, USER_3_AMOUNT),
];

pub static EMPTY_BENEFICIARIES_MAP: LazyLock<BTreeMap<AccountId, Balance>> =
    LazyLock::new(|| BTreeMap::new());

pub static GENESIS_BENEFICIARIES_MAP: LazyLock<BTreeMap<AccountId, Balance>> =
    LazyLock::new(|| GENESIS_BENEFICIARIES.into_iter().collect());

pub static SUFFICIENT_GENESIS_BALANCE: Balance = USER_1_AMOUNT + USER_2_AMOUNT + USER_3_AMOUNT;
pub const INSUFFICIENT_GENESIS_BALANCE: Balance = USER_5_AMOUNT;

pub static NEW_BENEFICIARIES: [(AccountId, Balance); 3] = [
    (USER_4, USER_4_AMOUNT),
    (USER_5, USER_5_AMOUNT),
    (USER_6, USER_6_AMOUNT),
];

pub static NEW_BENEFICIARIES_MAP: LazyLock<BTreeMap<AccountId, Balance>> =
    LazyLock::new(|| NEW_BENEFICIARIES.into_iter().collect());

pub static NEW_SUFFICIENT_BALANCE: Balance = USER_4_AMOUNT + USER_5_AMOUNT + USER_6_AMOUNT;

pub struct MockWeightInfo;

impl MockWeightInfo {
    pub const REF_TIME: u64 = 42;
    pub const PROOF_SIZE: u64 = 24;
}

impl crate::WeightInfo for MockWeightInfo {
    fn begin_airdrop(n: u32) -> frame_support::weights::Weight {
        let variable = 1000 * n as u64;
        frame_support::weights::Weight::from_parts(
            Self::REF_TIME + variable,
            Self::PROOF_SIZE + variable,
        )
    }

    fn claim() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(Self::REF_TIME, Self::PROOF_SIZE)
    }

    fn claim_for() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(Self::REF_TIME, Self::PROOF_SIZE)
    }

    fn add_beneficiaries(n: u32) -> frame_support::weights::Weight {
        let variable = 1000 * n as u64;
        frame_support::weights::Weight::from_parts(
            Self::REF_TIME + variable,
            Self::PROOF_SIZE + variable,
        )
    }

    fn end_airdrop(n: u32) -> frame_support::weights::Weight {
        let variable = 1000 * n as u64;
        frame_support::weights::Weight::from_parts(
            Self::REF_TIME + variable,
            Self::PROOF_SIZE + variable,
        )
    }
}

parameter_types! {
    pub const MockDbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 4_200_000,
       write: 2_400_000,
    };
}

pub struct MockManager;
impl<O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>> EnsureOrigin<O>
    for MockManager
{
    type Success = ();

    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Signed(MANAGER_USER) => Ok(()),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(RawOrigin::Signed(MANAGER_USER)))
    }
}

parameter_types! {
    pub const ClaimPalletId: PalletId = PalletId(*b"zkvt/clm");
    pub const MaxBeneficiaries: u32 = 100;
    pub UnclaimedDestinationMockAccount: AccountId = 111;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ClaimPalletId;
    type ManagerOrigin = EitherOfDiverse<EnsureRoot<AccountId>, MockManager>;
    type Currency = Balances;
    type UnclaimedDestination = UnclaimedDestinationMockAccount;
    type WeightInfo = MockWeightInfo;
    #[cfg(feature = "runtime-benchmarks")]
    const MAX_BENEFICIARIES: u32 = MaxBeneficiaries::get();
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Claim: crate,
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

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type AccountStore = System;
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
}

pub(crate) enum WithGenesisBeneficiaries {
    Yes,
    No,
}

pub(crate) enum GenesisClaimBalance {
    Sufficient,
    Insufficient,
    None,
}

// Build genesis storage according to the mock runtime.
pub fn test_with_configs(
    with_genesis_beneficiaries: WithGenesisBeneficiaries,
    genesis_claim_balance: GenesisClaimBalance,
) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(MANAGER_USER, 42_000_000_000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    crate::GenesisConfig::<Test> {
        beneficiaries: match with_genesis_beneficiaries {
            WithGenesisBeneficiaries::Yes => GENESIS_BENEFICIARIES.to_vec(),
            WithGenesisBeneficiaries::No => vec![],
        },
        genesis_balance: match genesis_claim_balance {
            GenesisClaimBalance::Sufficient => SUFFICIENT_GENESIS_BALANCE,
            GenesisClaimBalance::Insufficient => INSUFFICIENT_GENESIS_BALANCE,
            GenesisClaimBalance::None => 0,
        },
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::from(t);

    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

pub fn test() -> sp_io::TestExternalities {
    test_with_configs(WithGenesisBeneficiaries::No, GenesisClaimBalance::None)
}
