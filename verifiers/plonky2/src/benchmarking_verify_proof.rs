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

use crate::Plonky2 as Verifier;
use crate::{resources::*, Plonky2Config};
use frame_benchmarking::v2::*;
use hp_verifiers::Verifier as _;
use pallet_verifiers::benchmarking_utils;

pub struct Pallet<T: Config>(crate::Pallet<T>);

pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>)]
mod benchmarks {
    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_2() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(2, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_2() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(2, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_3() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(3, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_3() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(3, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_4() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(4, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_4() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(4, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_5() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(5, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_5() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(5, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_6() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(6, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_6() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(6, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_7() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(7, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_7() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(7, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_8() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(8, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_8() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(8, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_9() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(9, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_9() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(9, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_10() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(10, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_10() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(10, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_11() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(11, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_11() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(11, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_12() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(12, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_12() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(12, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_13() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(13, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_13() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(13, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_14() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(14, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_14() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(14, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_15() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(15, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_15() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(15, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_16() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(16, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_16() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(16, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_17() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(17, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_17() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(17, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_18() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(18, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_18() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(18, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_19() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(19, Plonky2Config::Poseidon);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_19() {
        let TestData { vk, proof, pubs } = get_parameterized_test_data(19, Plonky2Config::Keccak);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
}

#[cfg(test)]
pub mod mock {
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

    impl crate::Config for Test {
        type MaxProofSize = ConstU32<1000000>;
        type MaxPubsSize = ConstU32<1000000>;
        type MaxVkSize = ConstU32<1000000>;
        type WeightInfo = ();
    }

    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlockU32<Test>;
        type AccountId = AccountId;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    parameter_types! {
        pub const BaseDeposit: Balance = 1;
        pub const PerByteDeposit: Balance = 2;
        pub const HoldReasonVkRegistration: RuntimeHoldReason = RuntimeHoldReason::CommonVerifiersPallet(pallet_verifiers::common::HoldReason::VkRegistration);
    }

    impl pallet_verifiers::Config<crate::Plonky2<Test>> for Test {
        type OnProofVerified = ();
        type WeightInfo = crate::Plonky2Weight<()>;
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
        type DoneSlashHandler = ();
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
