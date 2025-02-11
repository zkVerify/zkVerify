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

use super::*;

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use hp_on_proof_verified::OnProofVerified;
use sp_core::{Get, H256};
use sp_runtime::SaturatedConversion;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn on_proof_verified() {
        #[block]
        {
            Pallet::<T>::on_proof_verified(None, None, Default::default());
        }
    }

    #[benchmark]
    fn publish_attestation() {
        // Setup code
        let max_attestations: u64 = T::MaxStorageAttestations::get().into();

        let mut hash: [u8; 32] = [0; 32];
        // Fill completely first attestation that will be pruned
        for _ in 0..T::MinProofsForPublishing::get() * 2 {
            hash[0] += 1;
            Values::<T>::insert(0, H256::from_slice(&hash), ());
        }

        // Fill the others partially
        for id in 1..max_attestations - 1 {
            Values::<T>::insert(id, H256::default(), ());
        }

        // Fill the last one completely (as it is the one that will be published)
        for _ in 0..T::MinProofsForPublishing::get() * 2 {
            hash[0] += 1;
            Values::<T>::insert(max_attestations - 1, H256::from_slice(&hash), ());
        }

        // Check all inserted values are present
        assert_eq!(
            Values::<T>::iter_keys().count(),
            ((2 * 2 * T::MinProofsForPublishing::get().saturated_into::<u64>())
                + (max_attestations - 2)) as usize
        );

        NextAttestation::<T>::set(max_attestations - 1);

        #[extrinsic_call]
        publish_attestation(RawOrigin::Root);

        // Check that indeed oldest attestation has been removed
        assert!(Values::<T>::iter_key_prefix(0).next().is_none());

        // Check the others are still present
        for id in 1..max_attestations - 1 {
            assert!(Values::<T>::iter_key_prefix(id).next().is_some());
        }

        assert_eq!(
            Values::<T>::iter_key_prefix(max_attestations - 1).count(),
            (T::MinProofsForPublishing::get() * 2) as usize
        );
    }

    #[cfg(test)]
    use crate::Pallet as Poe;
    impl_benchmark_test_suite!(Poe, crate::mock::new_test_ext(), crate::mock::Test,);
}
