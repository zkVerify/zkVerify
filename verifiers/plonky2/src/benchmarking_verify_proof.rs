#![cfg(feature = "runtime-benchmarks")]

use crate::Plonky2 as Verifier;
use crate::{Plonky2Config, Proof, Vk};
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

    macro_rules! generate_compressed_poseidon_benchmarks {
        ( $( $degree:literal ),* ) => {
            $(
                #[benchmark]
                fn $crate::paste::paste!{[<verify_proof_poseidon_compressed_ $degree>]}() {
                    let vk = Vk::new(
                        Plonky2Config::Poseidon,
                        include_bytes!(concat!(
                        "resources/degree_",
                        stringify!($degree),
                        "/compressed/poseidon/vk.bin"
                    )).to_vec());
                    let proof_bytes = include_bytes!(concat!(
                        "resources/degree_",
                        stringify!($degree),
                        "/compressed/poseidon/proof.bin"
                    )).to_vec();
                    let proof = Proof::new(true, proof_bytes);
                    let pubs = include_bytes!(concat!(
                        "resources/degree_",
                        stringify!($degree),
                        "/compressed/poseidon/pubs.bin"
                    )).to_vec();

                    let r;
                    #[block]
                    {
                        r = do_verify_proof::<T>(&vk, &proof, &pubs)
                    };
                    assert!(r.is_ok());
                }
            )*
        };
    }

    generate_compressed_poseidon_benchmarks!(2, 3, 4);

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_2() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_2/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_2/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_2/compressed/poseidon/pubs.bin").to_vec();

    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_3() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_3/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_3/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_3/compressed/poseidon/pubs.bin").to_vec();

    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_4() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_4/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_4/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_4/compressed/poseidon/pubs.bin").to_vec();

    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

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
        type MaxNSegment = ConstU32<4>;
        type Segment20MaxSize = ConstU32<350_000>;
        type MaxPubsSize = ConstU32<2060>;
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
