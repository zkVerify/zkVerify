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

use crate::{Proof, ProofType, Ultrahonk as Verifier};
use alloc::vec::Vec;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use hp_verifiers::Verifier as _;
use pallet_verifiers::{benchmarking_utils, VkOrHash};
use resources::*;
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
    fn verify_zk_proof_log_7() {
        let test_params = TestParams::new(7, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_7() {
        let test_params = TestParams::new(7, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_8() {
        let test_params = TestParams::new(8, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_8() {
        let test_params = TestParams::new(8, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_9() {
        let test_params = TestParams::new(9, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_9() {
        let test_params = TestParams::new(9, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_10() {
        let test_params = TestParams::new(10, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_10() {
        let test_params = TestParams::new(10, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_11() {
        let test_params = TestParams::new(11, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_11() {
        let test_params = TestParams::new(11, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_12() {
        let test_params = TestParams::new(12, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_12() {
        let test_params = TestParams::new(12, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_13() {
        let test_params = TestParams::new(13, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_13() {
        let test_params = TestParams::new(13, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_14() {
        let test_params = TestParams::new(14, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_14() {
        let test_params = TestParams::new(14, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_15() {
        let test_params = TestParams::new(15, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_15() {
        let test_params = TestParams::new(14, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_16() {
        let test_params = TestParams::new(16, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_16() {
        let test_params = TestParams::new(15, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_17() {
        let test_params = TestParams::new(17, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_17() {
        let test_params = TestParams::new(17, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_18() {
        let test_params = TestParams::new(18, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_18() {
        let test_params = TestParams::new(18, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_19() {
        let test_params = TestParams::new(19, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_19() {
        let test_params = TestParams::new(19, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_20() {
        let test_params = TestParams::new(20, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_20() {
        let test_params = TestParams::new(20, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_21() {
        let test_params = TestParams::new(21, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_21() {
        let test_params = TestParams::new(21, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_22() {
        let test_params = TestParams::new(22, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_22() {
        let test_params = TestParams::new(22, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_23() {
        let test_params = TestParams::new(23, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_23() {
        let test_params = TestParams::new(23, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_24() {
        let test_params = TestParams::new(24, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_24() {
        let test_params = TestParams::new(24, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_zk_proof_log_25() {
        let test_params = TestParams::new(25, ProofType::ZK);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_plain_proof_log_25() {
        let test_params = TestParams::new(25, ProofType::Plain);
        let TestData { vk, proof, pubs } = get_parameterized_test_data(test_params);
        let vproof = VersionedProof::V3_0(proof);
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &vproof, &pubs)
        };
        assert!(r.is_ok());
    }
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
