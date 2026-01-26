// Copyright 2025, Horizen Labs, Inc.
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

use crate::{Tee as Verifier, Vk};
use alloc::vec;
use frame_benchmarking::v2::*;
use frame_support::sp_runtime::traits::UniqueSaturatedInto;
use frame_system::RawOrigin;
use hp_verifiers::Verifier as _;
use pallet_timestamp::Pallet as Timestamp;
use pallet_verifiers::{benchmarking_utils, VkOrHash};

pub trait Config: crate::Config {}
pub struct Pallet<T: Config>(crate::Pallet<T>);
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

const PRESENT: u64 = 1769092187000; // Thu, 22 Jan 2026 14:29:47 GMT, in ms

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>, T: pallet_timestamp::Config, T: pallet_babe::Config)]
mod benchmarks {

    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    fn set_timestamp<T>(ts: u64)
    where
        T: pallet_babe::Config + pallet_timestamp::Config,
    {
        // We only actually need the timestamp to be in the valid time frame for the TcbInfo.
        // BABE must be aligned to prevent assertion failures.
        pallet_babe::CurrentSlot::<T>::put(sp_consensus_babe::Slot::from(ts / 6000)); // slot time
        let timestamp: T::Moment = ts.unique_saturated_into();
        Timestamp::<T>::set_timestamp(timestamp);
    }

    #[benchmark]
    fn verify_proof() {
        let proof = include_bytes!("resources/intel/valid_quote.dat").to_vec();
        let pubs = vec![];
        let vk = Vk {
            tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json")
                .to_vec()
                .try_into()
                .unwrap(),
            certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem")
                .to_vec()
                .try_into()
                .unwrap(),
        };

        set_timestamp::<T>(PRESENT);

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn get_vk() {
        let vk = Vk {
            tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json")
                .to_vec()
                .try_into()
                .unwrap(),
            certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem")
                .to_vec()
                .try_into()
                .unwrap(),
        };
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
        let vk = Vk {
            tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json")
                .to_vec()
                .try_into()
                .unwrap(),
            certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem")
                .to_vec()
                .try_into()
                .unwrap(),
        };

        set_timestamp::<T>(PRESENT);

        let r;
        #[block]
        {
            r = do_validate_vk::<T>(&vk)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn compute_statement_hash() {
        let proof = include_bytes!("resources/intel/valid_quote.dat").to_vec();
        let pubs = vec![];
        let vk = Vk {
            tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json")
                .to_vec()
                .try_into()
                .unwrap(),
            certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem")
                .to_vec()
                .try_into()
                .unwrap(),
        };

        let vk = VkOrHash::Vk(vk.into());

        #[block]
        {
            do_compute_statement_hash::<T>(&vk, &proof, &pubs);
        }
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller = funded_account::<T>();
        let vk = Vk {
            tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json")
                .to_vec()
                .try_into()
                .unwrap(),
            certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem")
                .to_vec()
                .try_into()
                .unwrap(),
        };

        set_timestamp::<T>(PRESENT);

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.clone().into());

        // Verify
        assert!(do_get_vk::<T>(&do_vk_hash::<T>(&vk)).is_some());
    }

    #[benchmark]
    fn unregister_vk() {
        // setup code
        let caller: T::AccountId = funded_account::<T>();
        let hash = sp_core::H256::repeat_byte(2);
        let vk = Vk {
            tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json")
                .to_vec()
                .try_into()
                .unwrap(),
            certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem")
                .to_vec()
                .try_into()
                .unwrap(),
        };

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
    use sp_core::{ConstU128, ConstU32, ConstU64};

    type Balance = u128;
    type AccountId = u64;

    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            Babe: pallet_babe,
            Timestamp: pallet_timestamp,
            Balances: pallet_balances,
            CommonVerifiersPallet: pallet_verifiers::common,
            VerifierPallet: crate,
        }
    );

    impl crate::Config for Test {
        type UnixTime = Timestamp;
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

    impl pallet_verifiers::Config<crate::Tee<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::TeeWeight<()>;
        type Ticket = HoldConsideration<
            AccountId,
            Balances,
            HoldReasonVkRegistration,
            LinearStoragePrice<BaseDeposit, PerByteDeposit, Balance>,
        >;
        type Currency = Balances;
    }

    impl pallet_babe::Config for Test {
        type EpochDuration = ConstU64<10>;
        type ExpectedBlockTime = ConstU64<6000>;
        // session module is the trigger
        type EpochChangeTrigger = pallet_babe::SameAuthoritiesForever;
        type DisabledValidators = ();
        type WeightInfo = ();
        type MaxAuthorities = ConstU32<10>;
        type MaxNominators = ConstU32<100>;
        type KeyOwnerProof = sp_core::Void;
        type EquivocationReportSystem = ();
    }

    impl pallet_timestamp::Config for Test {
        /// A timestamp: milliseconds since the unix epoch.
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ConstU64<5>;
        type WeightInfo = ();
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
        ext.execute_with(|| {
            System::set_block_number(1);
            Timestamp::set_timestamp(crate::benchmarking::PRESENT); // Thu, 22 Jan 2026 14:29:47 GMT
        });
        ext
    }
}
