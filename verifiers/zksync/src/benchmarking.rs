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
use frame_system::RawOrigin;
use hp_verifiers::Verifier;
use pallet_verifiers::{VkOrHash, Vks};

pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: pallet_verifiers::Config<Zksync> {}
impl<T: pallet_verifiers::Config<Zksync>> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Zksync>;

include!("resources.rs");

#[benchmarks]
mod benchmarks {

    use super::*;

    #[benchmark]
    fn submit_proof() {
        // setup code
        let caller = whitelisted_caller();
        let vk = VkOrHash::from_vk(());
        let proof = PROOF.into();
        let pubs = PUBS.into();

        #[extrinsic_call]
        submit_proof(RawOrigin::Signed(caller), vk, proof, pubs, None);
    }

    #[benchmark]
    fn submit_proof_with_vk_hash() {
        // setup code
        let caller = whitelisted_caller();
        let proof = PROOF.into();
        let pubs = PUBS.into();
        let hash = sp_core::H256::repeat_byte(2);
        Vks::<T, Zksync>::insert(hash, ());
        let vk_or_hash = VkOrHash::from_hash(hash);

        #[extrinsic_call]
        submit_proof(RawOrigin::Signed(caller), vk_or_hash, proof, pubs, None);
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller = whitelisted_caller();
        let vk = ().into();

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk);

        // Verify
        assert!(Vks::<T, Zksync>::get(Zksync::vk_hash(&())).is_some());
    }
}
