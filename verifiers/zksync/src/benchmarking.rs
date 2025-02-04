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

use crate::Zksync;
use frame_benchmarking::v2::*;
use frame_support::traits::{Consideration, Footprint};
use frame_system::RawOrigin;
use hp_verifiers::Verifier;
use pallet_aggregate::{funded_account, insert_domain};
use pallet_verifiers::{Tickets, VkEntry, VkOrHash, Vks};

pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: pallet_verifiers::Config<Zksync> {}
impl<T: pallet_verifiers::Config<Zksync>> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Zksync>;

include!("resources.rs");

fn init<T: pallet_aggregate::Config>() -> (T::AccountId, u32) {
    let caller: T::AccountId = funded_account::<T>();
    let domain_id = 1;
    insert_domain::<T>(domain_id, caller.clone(), Some(1));
    (caller, domain_id)
}

#[benchmarks(where T: pallet_aggregate::Config)]
mod benchmarks {

    use super::*;

    #[benchmark]
    fn submit_proof() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let vk = VkOrHash::from_vk(());
        let proof = PROOF.into();
        let pubs = PUBS.into();

        #[extrinsic_call]
        submit_proof(RawOrigin::Signed(caller), vk, proof, pubs, Some(domain_id));
    }

    #[benchmark]
    fn submit_proof_with_vk_hash() {
        // setup code
        let (caller, domain_id) = init::<T>();

        let proof = PROOF.into();
        let pubs = PUBS.into();
        let hash = sp_core::H256::repeat_byte(2);
        let vk_entry = VkEntry::new(());
        Vks::<T, Zksync>::insert(hash, vk_entry);
        let vk_or_hash = VkOrHash::from_hash(hash);

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            vk_or_hash,
            proof,
            pubs,
            Some(domain_id),
        );
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller: T::AccountId = funded_account::<T>();
        let vk = ().into();

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk);

        // Verify
        assert!(Vks::<T, Zksync>::get(Zksync::vk_hash(&())).is_some());
    }

    #[benchmark]
    fn unregister_vk() {
        // setup code
        let caller: T::AccountId = funded_account::<T>();
        let hash = sp_core::H256::repeat_byte(2);
        let vk = ().into();
        let vk_entry = VkEntry::new(vk);
        let footprint = Footprint::from_encodable(&vk_entry);
        let ticket = T::Ticket::new(&caller, footprint).unwrap();

        Vks::<T, Zksync>::insert(hash, vk_entry);
        Tickets::<T, Zksync>::insert((caller.clone(), hash), ticket);

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
        type OnProofVerified = Aggregate;
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
        type DispatchAggregation = ();
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
