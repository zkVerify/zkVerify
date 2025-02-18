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

#![cfg(feature = "runtime-benchmarks")]

use crate::Zksync as Verifier;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use hp_verifiers::Verifier as _;
use pallet_verifiers::{benchmarking_utils, VkOrHash};

pub struct Pallet<T: Config>(crate::Pallet<T>);

pub trait Config: pallet_verifiers::Config<Verifier> {}
impl<T: pallet_verifiers::Config<Verifier>> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier>;

include!("resources.rs");

#[benchmarks]
mod benchmarks {

    use super::*;

    benchmarking_utils!(Verifier);

    #[benchmark]
    fn verify_proof() {
        let proof = PROOF.into();
        let pubs = PUBS.into();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&(), &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn get_vk() {
        let hash = sp_core::H256::repeat_byte(2);

        insert_vk_anonymous::<T>((), hash);

        let r;
        #[block]
        {
            r = do_get_vk::<T>(&hash)
        };
        assert!(r.is_some());
    }

    #[benchmark]
    fn validate_vk() {
        let r;
        #[block]
        {
            r = do_validate_vk::<T>(&())
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn compute_statement_hash() {
        let proof = PROOF.into();
        let pubs = PUBS.into();

        let vk = VkOrHash::Vk(().into());

        #[block]
        {
            do_compute_statement_hash::<T>(&vk, &proof, &pubs);
        }
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller = funded_account::<T>();
        let vk = ();

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.into());

        // Verify
        assert!(do_get_vk::<T>(&do_vk_hash::<T>(&vk)).is_some());
    }

    #[benchmark]
    fn unregister_vk() {
        // setup code
        let caller: T::AccountId = funded_account::<T>();
        let hash = sp_core::H256::repeat_byte(2);
        let vk = ();

        insert_vk::<T>(caller.clone(), vk, hash);

        #[extrinsic_call]
        unregister_vk(RawOrigin::Signed(caller), hash);

        // Verify
        assert!(do_get_vk::<T>(&hash).is_none());
    }
    impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
}

#[cfg(test)]
mod mock {
    use frame_support::{
        derive_impl, parameter_types,
        sp_runtime::{traits::IdentityLookup, BuildStorage},
        traits::{fungible::HoldConsideration, LinearStoragePrice},
    };
    use sp_core::{ConstU128, ConstU32};

    type Balance = u128;
    type AccountId = u64;

    // Configure a mock runtime to test the pallet.
    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            Balances: pallet_balances,
            CommonVerifiersPallet: pallet_verifiers::common,
            VerifierPallet: crate,
        }
    );

    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlockU32<Test>;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type AccountData = pallet_balances::AccountData<Balance>;
    }

    parameter_types! {
            pub const BaseDeposit: Balance = 1;
            pub const PerByteDeposit: Balance = 2;
            pub const HoldReasonVkRegistration: RuntimeHoldReason = RuntimeHoldReason::CommonVerifiersPallet(pallet_verifiers::common::HoldReason::VkRegistration);
    }

    impl pallet_verifiers::Config<crate::Zksync> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::ZksyncWeight<()>;
        type Ticket = HoldConsideration<
            AccountId,
            Balances,
            HoldReasonVkRegistration,
            LinearStoragePrice<BaseDeposit, PerByteDeposit, Balance>,
        >;
        type Currency = Balances;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type WeightInfo = ();
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxLocks = ConstU32<10>;
        type MaxReserves = ConstU32<10>;
        type MaxFreezes = ConstU32<10>;
    }

    impl pallet_verifiers::common::Config for Test {
        type CommonWeightInfo = Test;
    }

    /// Build genesis storage according to the mock runtime.
    pub fn test_ext() -> sp_io::TestExternalities {
        let mut ext = sp_io::TestExternalities::from(
            frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .unwrap(),
        );
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
