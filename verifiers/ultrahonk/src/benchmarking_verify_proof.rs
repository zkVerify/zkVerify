// Copyright 2025, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]

use crate::resources::*;
use crate::{ProofType, Ultrahonk as Verifier};
use frame_benchmarking::v2::*;
use hp_verifiers::Verifier as _;
use pallet_verifiers::benchmarking_utils;
pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>)]
pub mod benchmarks {

    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    #[benchmark]
    fn verify_zk_proof_v3_0(
        log_n: Linear<MIN_BENCHMARKED_LOG_CIRCUIT_SIZE, MAX_BENCHMARKED_LOG_CIRCUIT_SIZE>,
    ) {
        let test_params = TestParams::new(log_n, ProofType::ZK, ProtocolVersion::V3_0);
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = get_parameterized_test_data(test_params).unwrap();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&versioned_vk, &versioned_proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_v3_0(
        log_n: Linear<MIN_BENCHMARKED_LOG_CIRCUIT_SIZE, MAX_BENCHMARKED_LOG_CIRCUIT_SIZE>,
    ) {
        let test_params = TestParams::new(log_n, ProofType::Plain, ProtocolVersion::V3_0);
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = get_parameterized_test_data(test_params).unwrap();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&versioned_vk, &versioned_proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_v0_84() {
        let test_params = TestParams::new_v0_84(ProofType::ZK);
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = get_parameterized_test_data(test_params).unwrap();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&versioned_vk, &versioned_proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_v0_84() {
        let test_params = TestParams::new_v0_84(ProofType::Plain);
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = get_parameterized_test_data(test_params).unwrap();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&versioned_vk, &versioned_proof, &pubs)
        };
        assert!(r.is_ok());
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

    impl crate::Config for Test {
        type MaxPubs = ConstU32<2060>; // this is arbitrary right now
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

    impl pallet_verifiers::Config<crate::Ultrahonk<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::UltrahonkWeight<()>;
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
