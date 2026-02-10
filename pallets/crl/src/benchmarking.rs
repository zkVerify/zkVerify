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

use super::*;
use frame_benchmarking::v2::*;
use frame_support::sp_runtime::traits::UniqueSaturatedInto;
use frame_system::RawOrigin;
use pallet::*;

/// Mar 1 2026 00:00:00 UTC, in milliseconds.
/// This falls within the validity period of the generated benchmark certificates
/// (valid from Feb 9 2026 to Feb 4 2046).
const PRESENT_MS: u64 = 1_772_323_200_000;

fn set_timestamp<T>(ts: u64)
where
    T: pallet_babe::Config + pallet_timestamp::Config,
{
    // We only actually need the timestamp to be in the valid time frame for the TcbInfo.
    // BABE must be aligned to prevent assertion failures.
    pallet_babe::CurrentSlot::<T>::put(sp_consensus_babe::Slot::from(ts / 6000)); // slot time
    let timestamp: T::Moment = ts.unique_saturated_into();
    pallet_timestamp::Pallet::<T>::set_timestamp(timestamp);
}

fn register_test_ca<T: Config>() -> CaName<T> {
    let root_cert = include_bytes!("resources/bench/root_ca.der").to_vec();
    let name = b"Benchmark_CA".to_vec();
    let bounded_name: CaName<T> = name
        .try_into()
        .expect("CA name should fit within MaxCaNameLength");
    let bounded_root_cert: BoundedVec<u8, ConstU32<MAX_ROOT_CERT_LENGTH>> = root_cert
        .try_into()
        .expect("Root cert should fit within MAX_ROOT_CERT_LENGTH");

    CertificateAuthorities::<T>::insert(
        &bounded_name,
        CaInfo {
            root_cert: bounded_root_cert,
            revoked_count: 0,
            crl_version: 0,
        },
    );
    bounded_name
}

/// Select the pre-generated CRL whose revoked certificate count is closest to `n`.
/// Available test CRLs have 1, 10, 100, 500, and 1000 revoked certificates.
fn select_crl(n: u32) -> alloc::vec::Vec<u8> {
    if n <= 5 {
        include_bytes!("resources/bench/crl_1.pem").to_vec()
    } else if n <= 55 {
        include_bytes!("resources/bench/crl_10.pem").to_vec()
    } else if n <= 300 {
        include_bytes!("resources/bench/crl_100.pem").to_vec()
    } else if n <= 750 {
        include_bytes!("resources/bench/crl_500.pem").to_vec()
    } else {
        include_bytes!("resources/bench/crl_1000.pem").to_vec()
    }
}

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_timestamp::Config + pallet_babe::Config)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_ca() {
        let name = b"Test_CA".to_vec();
        let root_cert = include_bytes!("resources/bench/root_ca.der").to_vec();

        #[extrinsic_call]
        register_ca(RawOrigin::Root, name.clone(), root_cert);

        // Verify the CA was registered.
        let bounded_name: CaName<T> = name.try_into().unwrap();
        assert!(CertificateAuthorities::<T>::contains_key(&bounded_name));
    }

    #[benchmark]
    fn unregister_ca() {
        let ca_name = register_test_ca::<T>();
        let name = ca_name.to_vec();

        #[extrinsic_call]
        unregister_ca(RawOrigin::Root, name);

        // Verify the CA was removed.
        assert!(!CertificateAuthorities::<T>::contains_key(&ca_name));
    }

    /// Benchmark `update_crl` parameterized by the number of revoked certificates.
    ///
    /// Uses pre-generated CRLs with 1, 10, 100, 500, and 1000 revoked entries.
    /// The closest matching CRL is selected for each value of `n`.
    #[benchmark]
    fn update_crl(n: Linear<1, 1000>) {
        let ca_name = register_test_ca::<T>();
        let name = ca_name.to_vec();
        let crl_pem = select_crl(n);
        let cert_chain_pem = include_bytes!("resources/bench/chain.pem").to_vec();

        let caller: T::AccountId = whitelisted_caller();
        set_timestamp::<T>(PRESENT_MS);

        #[extrinsic_call]
        update_crl(RawOrigin::Signed(caller), name, crl_pem, cert_chain_pem);

        // Verify the CRL was stored.
        let ca_info = CertificateAuthorities::<T>::get(&ca_name).unwrap();
        assert!(ca_info.crl_version > 0);
    }

    #[benchmark]
    fn clear_crl() {
        let ca_name = register_test_ca::<T>();

        #[block]
        {
            Revoked::<T>::remove(&ca_name);
        }

        assert!(Revoked::<T>::get(&ca_name).is_none());
    }

    #[cfg(test)]
    use crate::Pallet as CrlPallet;
    impl_benchmark_test_suite!(CrlPallet, super::mock::test_ext(), super::mock::Test);
}

#[cfg(test)]
mod mock {
    use frame_support::{
        derive_impl, parameter_types,
        sp_runtime::{traits::IdentityLookup, BuildStorage},
    };
    use sp_core::{ConstU32, ConstU64};

    type AccountId = u64;

    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            Babe: pallet_babe,
            Timestamp: pallet_timestamp,
            CrlPallet: crate,
        }
    );

    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlockU32<Test>;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    parameter_types! {
        pub const MaxCaNameLength: u32 = 64;
    }

    impl crate::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type ManagerOrigin = frame_system::EnsureRoot<AccountId>;
        type WeightInfo = ();
        type MaxCaNameLength = MaxCaNameLength;
        type UnixTime = Timestamp;
    }

    impl pallet_timestamp::Config for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = sp_core::ConstU64<5>;
        type WeightInfo = ();
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

    pub fn test_ext() -> sp_io::TestExternalities {
        let mut ext = sp_io::TestExternalities::from(
            frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .unwrap(),
        );
        ext.execute_with(|| {
            System::set_block_number(1);
            pallet_timestamp::Now::<Test>::put(super::PRESENT_MS);
        });
        ext
    }
}
