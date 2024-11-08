#![cfg(feature = "runtime-benchmarks")]

use crate::Plonky2;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use hp_verifiers::Verifier;
use pallet_verifiers::{VkOrHash, Vks};

pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: pallet_verifiers::Config<Plonky2> {}
impl<T: pallet_verifiers::Config<Plonky2>> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Plonky2>;

include!("resources.rs");

#[benchmarks]
mod benchmarks {

    use super::*;

    #[benchmark]
    fn submit_proof() {
        // setup code
        let caller = whitelisted_caller();

        let TestData { vk, proof, pubs } = get_valid_test_data();

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof.into(),
            pubs.into(),
        );
    }

    #[benchmark]
    fn submit_proof_with_vk_hash() {
        // setup code
        let caller = whitelisted_caller();
        let TestData { vk, proof, pubs } = get_valid_test_data();
        let hash = sp_core::H256::repeat_byte(2);
        Vks::<T, Plonky2>::insert(hash, vk);

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_hash(hash),
            proof.into(),
            pubs.into(),
        );
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller = whitelisted_caller();
        let vk = get_valid_test_data().vk;

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.clone().into());

        // Verify
        assert!(Vks::<T, Plonky2>::get(Plonky2::vk_hash(&vk)).is_some());
    }

    impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
}

#[cfg(test)]
mod mock {
    use frame_support::derive_impl;
    use sp_runtime::{traits::IdentityLookup, BuildStorage};

    // Configure a mock runtime to test the pallet.
    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            VerifierPallet: crate,
        }
    );

    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlockU32<Test>;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    impl pallet_verifiers::Config<crate::Plonky2> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::Plonky2Weight<()>;
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
