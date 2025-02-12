#![cfg(feature = "runtime-benchmarks")]

use crate::Halo2;
use frame_benchmarking::v2::*;
use frame_support::traits::{Consideration, Footprint};
use frame_system::RawOrigin;

use pallet_aggregate::{funded_account, insert_domain};
use pallet_verifiers::{Tickets, VkEntry, VkOrHash, Vks};
use sp_core::U256;

//pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Halo2<T>>;

fn init<T: pallet_aggregate::Config>() -> (T::AccountId, u32) {
    let caller: T::AccountId = funded_account::<T>();
    let domain_id = 1;
    insert_domain::<T>(domain_id, caller.clone(), Some(1));
    (caller, domain_id)
}

#[benchmarks(where T: pallet_verifiers::Config<Halo2<T>> + pallet_aggregate::Config)]
mod benchmarks {
    //use codec::Decode;
    use hp_verifiers::Verifier;

    use crate::ParamsAndVk;

    use super::*;

    #[benchmark]
    fn submit_proof_8() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let pubs_bytes = include_bytes!("resources/VALID_PUBS_8.bin").to_vec();
        let mut pubs = vec![];

        // using reader
        for chunk in pubs_bytes.chunks(32) {
            if chunk.len() == 32 {
                // ensure we have a full chunk
                let fr = U256::from_little_endian(chunk);
                pubs.push(fr);
            }
        }

        let vk = include_bytes!("resources/VALID_VK_8.bin").to_vec();
        let proof = include_bytes!("resources/VALID_PROOF_8.bin").to_vec();

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk.into()),
            proof.into(),
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_16() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let pubs_bytes = include_bytes!("resources/VALID_PUBS_16.bin").to_vec();
        let mut pubs = vec![];

        // using reader
        for chunk in pubs_bytes.chunks(32) {
            if chunk.len() == 32 {
                // ensure we have a full chunk
                let fr = U256::from_little_endian(chunk);
                pubs.push(fr);
            }
        }

        let vk = include_bytes!("resources/VALID_VK_16.bin").to_vec();
        let proof = include_bytes!("resources/VALID_PROOF_16.bin").to_vec();

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk.into()),
            proof.into(),
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_21() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let pubs_bytes = include_bytes!("resources/VALID_PUBS_21.bin").to_vec();
        let mut pubs = vec![];

        // using reader
        for chunk in pubs_bytes.chunks(32) {
            if chunk.len() == 32 {
                // ensure we have a full chunk
                let fr = U256::from_little_endian(chunk);
                pubs.push(fr);
            }
        }

        let vk = include_bytes!("resources/VALID_VK_21.bin").to_vec();
        let proof = include_bytes!("resources/VALID_PROOF_21.bin").to_vec();
        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk.into()),
            proof.into(),
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_with_vk_hash() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let pubs_bytes = include_bytes!("resources/VALID_PUBS_8.bin").to_vec();
        let mut pubs = vec![];

        // using reader
        for chunk in pubs_bytes.chunks(32) {
            if chunk.len() == 32 {
                // ensure we have a full chunk
                let fr = U256::from_little_endian(chunk);
                pubs.push(fr);
            }
        }

        let vk = include_bytes!("resources/VALID_VK_8.bin").to_vec();
        let proof = include_bytes!("resources/VALID_PROOF_8.bin").to_vec();

        let vk_entry = VkEntry::new(vk.into());

        let hash = sp_core::H256::repeat_byte(2);
        Vks::<T, Halo2<T>>::insert(hash, vk_entry);

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_hash(hash),
            proof.into(),
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller: T::AccountId = funded_account::<T>();

        let vk = include_bytes!("resources/VALID_VK_8.bin").to_vec();

        #[extrinsic_call]
        register_vk(
            RawOrigin::Signed(caller),
            ParamsAndVk::from(vk.clone()).into(),
        );

        // Verify
        assert!(Vks::<T, Halo2<T>>::get(Halo2::<T>::vk_hash(&ParamsAndVk::from(vk))).is_some());
    }

    #[benchmark]
    fn unregister_vk() {
        // setup code
        let caller: T::AccountId = funded_account::<T>();
        let hash = sp_core::H256::repeat_byte(2);
        let vk = include_bytes!("resources/VALID_VK_8.bin").to_vec();

        let footprint = Footprint::from_encodable(&vk);
        let vk_entry = VkEntry::new(vk.into());
        let ticket = T::Ticket::new(&caller, footprint).unwrap();

        Vks::<T, Halo2<T>>::insert(hash, vk_entry);
        Tickets::<T, Halo2<T>>::insert((caller.clone(), hash), ticket);

        #[extrinsic_call]
        unregister_vk(RawOrigin::Signed(caller), hash);
    }

    impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
}

#[cfg(test)]
mod mock {
    use frame_support::{
        derive_impl, parameter_types,
        sp_runtime::{traits::IdentityLookup, BuildStorage},
        traits::{fungible::HoldConsideration, EnsureOrigin, LinearStoragePrice},
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
            Aggregate: pallet_aggregate,
        }
    );

    impl crate::Config for Test {
        type VkMaxBytes = ConstU32<866598845>;
        type ProofMaxBytes = ConstU32<866598845>;
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

    impl pallet_verifiers::Config<crate::Halo2<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = Aggregate;
        type WeightInfo = crate::Halo2Weight<()>;
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

    pub struct NoManager;
    impl EnsureOrigin<RuntimeOrigin> for NoManager {
        type Success = ();

        fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
            Err(o)
        }

        fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
            Err(())
        }
    }

    impl pallet_aggregate::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type AggregationSize = ConstU32<32>;
        type MaxPendingPublishQueueSize = ConstU32<16>;
        type ManagerOrigin = NoManager;
        type Hold = Balances;
        type Consideration = ();
        type EstimateCallFee = ConstU32<1_000_000>;
        type ComputePublisherTip = ();
        type WeightInfo = ();
        const AGGREGATION_SIZE: u32 = 32;
        type Currency = Balances;
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
