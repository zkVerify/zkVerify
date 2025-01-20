// Copyright 2024, Horizen Labs, Inc.

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

use crate::Ultraplonk;
use frame_benchmarking::v2::*;
use frame_support::traits::{Consideration, Footprint};
use frame_system::RawOrigin;
use hp_verifiers::Verifier;
use pallet_aggregate::{funded_account, insert_domain};
use pallet_verifiers::{Tickets, VkEntry, VkOrHash, Vks};
use sp_std::{vec, vec::Vec};
pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Ultraplonk<T>>;

include!("resources.rs");

fn init<T: pallet_aggregate::Config>() -> (T::AccountId, u32) {
    let caller: T::AccountId = funded_account::<T>();
    let domain_id = 1;
    insert_domain::<T>(domain_id, caller.clone(), Some(1));
    (caller, domain_id)
}

#[benchmarks(where T: pallet_verifiers::Config<Ultraplonk<T>> + pallet_aggregate::Config)]
pub mod benchmarks {

    use super::*;

    #[benchmark]
    fn submit_proof() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let proof = VALID_PROOF.to_vec().into();
        let pubs = public_input().into();
        let vk = VALID_VK;

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof,
            pubs,
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_1() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let proof = include_bytes!("resources/01_proof").to_vec().into();
        let pubs = vec![*include_bytes!("resources/01_pubs")].into();
        let vk = *include_bytes!("resources/01_vk");

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof,
            pubs,
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_8() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let proof = include_bytes!("resources/08_proof").to_vec().into();
        let pubs: Vec<_> = include_bytes!("resources/08_pubs")
            .chunks_exact(crate::PUBS_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        let vk = *include_bytes!("resources/08_vk");

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof,
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_16() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let proof = include_bytes!("resources/16_proof").to_vec().into();
        let pubs: Vec<_> = include_bytes!("resources/16_pubs")
            .chunks_exact(crate::PUBS_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        let vk = *include_bytes!("resources/16_vk");

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof,
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_32() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let proof = include_bytes!("resources/32_proof").to_vec().into();
        let pubs: Vec<_> = include_bytes!("resources/32_pubs")
            .chunks_exact(crate::PUBS_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        let vk = *include_bytes!("resources/32_vk");

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof,
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_with_vk_hash() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let proof = VALID_PROOF.to_vec().into();
        let pubs = public_input();
        let vk = VALID_VK;
        let hash = sp_core::H256::repeat_byte(2);
        let vk_entry = VkEntry::new(vk);
        Vks::<T, Ultraplonk<T>>::insert(hash, vk_entry);

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_hash(hash),
            proof,
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn submit_proof_32_with_vk_hash() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let proof = include_bytes!("resources/32_proof").to_vec().into();
        let pubs: Vec<_> = include_bytes!("resources/32_pubs")
            .chunks_exact(crate::PUBS_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        let vk = *include_bytes!("resources/32_vk");
        let hash = sp_core::H256::repeat_byte(2);
        let vk_entry = VkEntry::new(vk);
        Vks::<T, Ultraplonk<T>>::insert(hash, vk_entry);

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_hash(hash),
            proof,
            pubs.into(),
            Some(domain_id),
        );
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller: T::AccountId = funded_account::<T>();
        let vk = VALID_VK;

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.clone().into());

        // Verify
        assert!(Vks::<T, Ultraplonk<T>>::get(Ultraplonk::<T>::vk_hash(&vk)).is_some());
    }

    #[benchmark]
    fn unregister_vk() {
        // setup code
        let caller: T::AccountId = funded_account::<T>();
        let hash = sp_core::H256::repeat_byte(2);
        let vk = VALID_VK;
        let vk_entry = VkEntry::new(vk);
        let footprint = Footprint::from_encodable(&vk_entry);
        let ticket = T::Ticket::new(&caller, footprint).unwrap();

        Vks::<T, Ultraplonk<T>>::insert(hash, vk_entry);
        Tickets::<T, Ultraplonk<T>>::insert((caller.clone(), hash), ticket);

        #[extrinsic_call]
        unregister_vk(RawOrigin::Signed(caller), hash);
    }

    // We cannot implement testing benchmarck for ultraplonk verifier due there is no way to make them
    // thread safe.
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
        type MaxPubs = ConstU32<2060>; // this is arbitrary right now
    }

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

    impl pallet_verifiers::Config<crate::Ultraplonk<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = Aggregate;
        type WeightInfo = crate::UltraplonkWeight<()>;
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
