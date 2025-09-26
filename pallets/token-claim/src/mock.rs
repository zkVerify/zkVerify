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

#![allow(
    clippy::borrow_interior_mutable_const,
    clippy::declare_interior_mutable_const
)]

use std::{collections::BTreeMap, sync::LazyLock};

use frame_support::{
    derive_impl, parameter_types,
    traits::{EitherOfDiverse, EnsureOrigin},
    BoundedVec, PalletId,
};
use frame_system::{EnsureRoot, RawOrigin};
use sp_core::ConstU128;
use sp_runtime::{
    testing::{TestSignature, UintAuthorityId},
    traits::IdentityLookup,
    BuildStorage, RuntimeAppPublic,
};

use crate::beneficiary::{AccountIdToBytesLiteral, Beneficiary};
use crate::utils::{get_beneficiaries_map, secp_utils::*};
use crate::{EthereumAddress, EthereumSignature};

pub type Balance = u128;
pub type AccountId = u64;
pub type Origin = RawOrigin<AccountId>;

pub const EXISTENTIAL_DEPOSIT: Balance = 1;

pub const USER_1_RAW: AccountId = 42;
pub const USER_1: Beneficiary<Test> = Beneficiary::<Test>::Substrate(USER_1_RAW);
pub const USER_1_AMOUNT: Balance = 42_000_000_000;
pub const USER_2: Beneficiary<Test> = Beneficiary::<Test>::Substrate(24);
pub const USER_2_AMOUNT: Balance = 24_000_000_000;
pub const USER_3_RAW: EthereumAddress = EthereumAddress(hex_literal::hex!(
    "CFb405552868d9906DeDCAbe2F387a37E35e9610"
));
pub const USER_3: Beneficiary<Test> = Beneficiary::<Test>::Ethereum(USER_3_RAW);
pub const USER_3_AMOUNT: Balance = 50_000_000_000;
pub const USER_4: Beneficiary<Test> = Beneficiary::<Test>::Substrate(24_000);
pub const USER_4_AMOUNT: Balance = 200_000_000_000;
pub const USER_5: Beneficiary<Test> = Beneficiary::<Test>::Substrate(99_000);
pub const USER_5_AMOUNT: Balance = 300_000_000;
pub const USER_6_RAW: EthereumAddress = EthereumAddress(hex_literal::hex!(
    "308046c262264a11445865f727f94fb699b3a1b8"
));
pub const USER_6: Beneficiary<Test> = Beneficiary::<Test>::Ethereum(USER_6_RAW);
pub const USER_6_AMOUNT: Balance = 100_000_000_000;
pub const NON_BENEFICIARY_RAW: AccountId = 6;
pub const NON_BENEFICIARY: Beneficiary<Test> = Beneficiary::<Test>::Substrate(NON_BENEFICIARY_RAW);

impl From<Beneficiary<Test>> for UintAuthorityId {
    fn from(b: Beneficiary<Test>) -> UintAuthorityId {
        match b {
            Beneficiary::<Test>::Substrate(address) => UintAuthorityId::from(address),
            Beneficiary::<Test>::Ethereum(_) => unreachable!(),
        }
    }
}

pub static USER_1_SIGN: LazyLock<(UintAuthorityId, TestSignature, TestSignature)> =
    LazyLock::new(|| {
        let user_signer = UintAuthorityId::from(USER_1);
        let claim_message = INIT_CLAIM_MESSAGE.clone();
        let wrapped_message = [
            crate::beneficiary::MSG_PREFIX,
            claim_message.as_slice(),
            crate::beneficiary::MSG_SUFFIX,
        ]
        .concat();
        let user_signature = user_signer
            .sign(&INIT_CLAIM_MESSAGE.clone().as_slice())
            .unwrap();
        let user_signature_wrapped = user_signer.sign(&wrapped_message.as_slice()).unwrap();
        (user_signer, user_signature, user_signature_wrapped)
    });

pub static USER_3_SIGN_USER_1_DEST: LazyLock<(EthereumAddress, EthereumSignature)> =
    LazyLock::new(|| {
        // Create beneficiary
        let secret_bytes =
            hex_literal::hex!("7b2d076abcc1215ef9c5a37da07f50c92de1048b2e1e7a27b74c0ce154f9cbae");
        let secret = parse_secret(&secret_bytes[..]);
        let address = eth(&secret);

        // Prepare signature
        let msg_to_sign = [
            INIT_CLAIM_MESSAGE.clone().as_slice(),
            crate::beneficiary::ETH_MSG_SEPARATOR,
            MockAccountIdToBytesConversion::to_bytes_literal(&USER_1_RAW).as_slice(),
        ]
        .concat();
        let eth_signature = sig(&secret, msg_to_sign.as_slice());
        (address, eth_signature)
    });

pub const MANAGER_USER: AccountId = 666;

pub static GENESIS_BENEFICIARIES: [(Beneficiary<Test>, Balance); 3] = [
    (USER_1, USER_1_AMOUNT),
    (USER_2, USER_2_AMOUNT),
    (USER_3, USER_3_AMOUNT),
];

pub static EMPTY_BENEFICIARIES_MAP: LazyLock<BTreeMap<Beneficiary<Test>, Balance>> =
    LazyLock::new(BTreeMap::new);

pub static GENESIS_BENEFICIARIES_MAP: LazyLock<BTreeMap<Beneficiary<Test>, Balance>> =
    LazyLock::new(|| GENESIS_BENEFICIARIES.clone().into_iter().collect());

pub static SUFFICIENT_GENESIS_BALANCE: Balance = USER_1_AMOUNT + USER_2_AMOUNT + USER_3_AMOUNT;
pub const INSUFFICIENT_GENESIS_BALANCE: Balance = USER_5_AMOUNT;

pub static NEW_BENEFICIARIES: [(Beneficiary<Test>, Balance); 3] = [
    (USER_4, USER_4_AMOUNT),
    (USER_5, USER_5_AMOUNT),
    (USER_6, USER_6_AMOUNT),
];

pub static NEW_BENEFICIARIES_MAP: LazyLock<BTreeMap<Beneficiary<Test>, Balance>> =
    LazyLock::new(|| NEW_BENEFICIARIES.clone().into_iter().collect());

pub static NEW_SUFFICIENT_BALANCE: Balance = USER_4_AMOUNT + USER_5_AMOUNT + USER_6_AMOUNT;

pub const INIT_CLAIM_MESSAGE: LazyLock<BoundedVec<u8, MaxClaimMessageLength>> =
    LazyLock::new(|| BoundedVec::try_from(b"TestMessage".to_vec()).unwrap());

pub const EMPTY_CLAIM_MESSAGE: LazyLock<BoundedVec<u8, MaxClaimMessageLength>> =
    LazyLock::new(|| BoundedVec::try_from(vec![]).unwrap());

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

pub struct MockAccountIdToBytesConversion;

impl AccountIdToBytesLiteral<Test> for MockAccountIdToBytesConversion {
    type AccountId = <Test as frame_system::Config>::AccountId;
    fn to_bytes_literal(account: &Self::AccountId) -> Vec<u8> {
        account.to_string().into_bytes()
    }
}

parameter_types! {
    pub const ClaimPalletId: PalletId = PalletId(*b"zkvt/clm");
    pub const MaxBeneficiaries: u32 = 100;
    pub const MaxOpBeneficiaries: u32 = MaxBeneficiaries::get() - 1;
    pub const MaxClaimMessageLength: u32 = 100;
    pub UnclaimedDestinationMockAccount: AccountId = 111;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct MockBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::BenchmarkHelper<TestSignature, UintAuthorityId> for MockBenchmarkHelper {
    fn sign_claim(message: &[u8]) -> (TestSignature, UintAuthorityId) {
        let signer = UintAuthorityId::from(USER_1);
        let signature = signer.sign(&message).unwrap();
        (signature, signer)
    }

    fn sign_claim_ethereum(message: &[u8]) -> (EthereumSignature, EthereumAddress) {
        use crate::utils::secp_utils::*;
        let sk = secret_from_seed(b"//TestBeneficiary");
        (sig(&sk, message), eth(&sk))
    }
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ClaimPalletId;
    type ManagerOrigin = EitherOfDiverse<EnsureRoot<AccountId>, MockManager>;
    type Currency = Balances;
    type UnclaimedDestination = UnclaimedDestinationMockAccount;
    type WeightInfo = ();
    type MaxBeneficiaries = MaxBeneficiaries;
    type MaxClaimMessageLength = MaxClaimMessageLength;
    type Signer = UintAuthorityId;
    type Signature = TestSignature;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = MockBenchmarkHelper;
    const MAX_OP_BENEFICIARIES: u32 = MaxOpBeneficiaries::get();
    type AccountIdBytesToSign = MockAccountIdToBytesConversion;
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
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type AccountStore = System;
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
}

pub enum WithGenesisBeneficiaries {
    Yes,
    No,
}

pub enum GenesisClaimBalance {
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
        claim_message: INIT_CLAIM_MESSAGE.clone(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::from(t);

    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

pub fn test_genesis_with_beneficiaries(n: u32) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(MANAGER_USER, 42_000_000_000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    crate::GenesisConfig::<Test> {
        beneficiaries: get_beneficiaries_map::<Test>(n).0.into_iter().collect(),
        genesis_balance: 42_000_000_000,
        claim_message: INIT_CLAIM_MESSAGE.clone(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::from(t);

    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

pub fn test_genesis_empty_claim_message(n: u32) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(MANAGER_USER, 42_000_000_000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    crate::GenesisConfig::<Test> {
        beneficiaries: get_beneficiaries_map::<Test>(n).0.into_iter().collect(),
        genesis_balance: 42_000_000_000,
        claim_message: EMPTY_CLAIM_MESSAGE.clone(),
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
