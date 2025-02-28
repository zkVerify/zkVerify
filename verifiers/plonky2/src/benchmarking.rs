#![cfg(feature = "runtime-benchmarks")]

use crate::Plonky2 as Verifier;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use hp_verifiers::Verifier as _;
use pallet_verifiers::{benchmarking_utils, VkOrHash};

pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

include!("resources.rs");

#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>)]
mod benchmarks {
    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    #[benchmark]
    fn verify_proof() {
        let data = get_valid_test_data();

        let proof = data.proof;
        let pubs = data.pubs;
        let vk = data.vk;

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn get_vk() {
        let vk = get_valid_test_data().vk;
        let hash = sp_core::H256::repeat_byte(2);

        insert_vk_anonymous::<T>(vk, hash);

        let r;
        #[block]
        {
            r = do_get_vk::<T>(&hash)
        };
        assert!(r.is_some());
    }

    #[benchmark]
    fn validate_vk() {
        let vk = get_valid_test_data().vk;

        let r;
        #[block]
        {
            r = do_validate_vk::<T>(&vk)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn compute_statement_hash() {
        let data = get_valid_test_data();

        let proof = data.proof;
        let pubs = data.pubs;
        let vk = data.vk.into();

        let vk = VkOrHash::Vk(vk);

        #[block]
        {
            do_compute_statement_hash::<T>(&vk, &proof, &pubs);
        }
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller = funded_account::<T>();
        let vk = get_valid_test_data().vk;

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.clone().into());

        // Verify
        assert!(do_get_vk::<T>(&do_vk_hash::<T>(&vk)).is_some());
    }

    #[benchmark]
    fn unregister_vk() {
        // setup code
        let caller = funded_account::<T>();
        let hash = sp_core::H256::repeat_byte(2);
        let vk = get_valid_test_data().vk;

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

    impl crate::Config for Test {
        type MaxProofSize = ConstU32<1000000>;
        type MaxPubsSize = ConstU32<1000000>;
        type MaxVkSize = ConstU32<1000000>;
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
        type RuntimeEvent = RuntimeEvent;
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
