// Copyright 2026, Horizen Labs, Inc.
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

use crate::{
    Config as VerifierConfig, Kimchi as Verifier, KimchiProfileId, Proof, Pubs, Vk, PUB_SIZE,
};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet_verifiers::traits::Verifier as _;
use pallet_verifiers::{benchmarking_utils, VkOrHash};

pub trait Config: crate::Config {}
pub struct Pallet<T: Config>(crate::Pallet<T>);
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

const BENCH_PROOF: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/proof.bin");
const BENCH_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/verifier_index.bin");
const BENCH_PUBS: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/pubs.bin");

fn benchmark_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    (
        BENCH_PROOF.to_vec(),
        Vk::new(BENCH_VERIFIER_INDEX.to_vec(), KimchiProfileId::Vesta16),
        decode_pubs(BENCH_PUBS),
    )
}

fn decode_pubs(bytes: &[u8]) -> Pubs {
    assert!(
        bytes.len().is_multiple_of(PUB_SIZE),
        "Kimchi public input fixture must be a sequence of {PUB_SIZE}-byte fields"
    );
    bytes
        .chunks_exact(PUB_SIZE)
        .map(|chunk| {
            chunk
                .try_into()
                .expect("chunks_exact always returns PUB_SIZE bytes")
        })
        .collect()
}

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>)]
mod benchmarks {
    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    #[benchmark]
    fn verify_proof() {
        let (proof, vk, pubs) = benchmark_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn get_vk() {
        let (_, vk, _) = benchmark_data::<T>();
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
        let (_, vk, _) = benchmark_data::<T>();

        let r;
        #[block]
        {
            r = do_validate_vk::<T>(&vk)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn compute_statement_hash() {
        let (proof, vk, pubs) = benchmark_data::<T>();
        let vk = VkOrHash::Vk(vk.into());

        #[block]
        {
            do_compute_statement_hash::<T>(&vk, &proof, &pubs);
        }
    }

    #[benchmark]
    fn register_vk() {
        let caller = funded_account::<T>();
        let (_, vk, _) = benchmark_data::<T>();

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.clone().into());

        assert!(do_get_vk::<T>(&do_vk_hash::<T>(&vk)).is_some());
    }

    #[benchmark]
    fn unregister_vk() {
        let caller: T::AccountId = funded_account::<T>();
        let hash = sp_core::H256::repeat_byte(2);
        let (_, vk, _) = benchmark_data::<T>();

        insert_vk::<T>(caller.clone(), vk, hash);

        #[extrinsic_call]
        unregister_vk(RawOrigin::Signed(caller), hash);

        assert!(do_get_vk::<T>(&hash).is_none());
    }

    #[benchmark]
    fn submit_proof_inline_vk() {
        let caller = funded_account::<T>();
        let (proof, vk, pubs) = benchmark_data::<T>();
        let vk_or_hash = VkOrHash::Vk(vk.into());

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            vk_or_hash,
            proof.into(),
            pubs.into(),
            None,
        );
    }

    #[benchmark]
    fn submit_proof_registered_vk() {
        let caller = funded_account::<T>();
        let (proof, vk, pubs) = benchmark_data::<T>();
        let hash = do_vk_hash::<T>(&vk);
        insert_vk::<T>(caller.clone(), vk, hash);

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::Hash(hash),
            proof.into(),
            pubs.into(),
            None,
        );
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
        type MaxProofSize = ConstU32<262144>;
        type MaxPubs = ConstU32<1024>;
        type MaxVkSize = ConstU32<65536>;
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

    impl pallet_verifiers::Config<crate::Kimchi<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::KimchiWeight<()>;
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

    pub fn test_ext() -> sp_io::TestExternalities {
        let mut ext = sp_io::TestExternalities::from(
            frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .expect("mock genesis storage should build"),
        );
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
