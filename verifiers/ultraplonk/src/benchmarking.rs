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
use frame_system::RawOrigin;
use hp_verifiers::Verifier;
use pallet_verifiers::{VkOrHash, Vks};
use sp_std::{vec, vec::Vec};
pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Ultraplonk<T>>;

include!("resources.rs");

#[benchmarks(where T: pallet_verifiers::Config<Ultraplonk<T>>)]
pub mod benchmarks {

    use super::*;

    #[benchmark]
    fn submit_proof() {
        // setup code
        let caller = whitelisted_caller();
        let proof = VALID_PROOF.to_vec().into();
        let pubs = public_input().into();
        let vk = VALID_VK;

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof,
            pubs,
            None,
        );
    }

    #[benchmark]
    fn submit_proof_1() {
        // setup code
        let caller = whitelisted_caller();
        let proof = include_bytes!("resources/01_proof").to_vec().into();
        let pubs = vec![*include_bytes!("resources/01_pubs")].into();
        let vk = *include_bytes!("resources/01_vk");

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof,
            pubs,
            None,
        );
    }

    #[benchmark]
    fn submit_proof_8() {
        // setup code
        let caller = whitelisted_caller();
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
            None,
        );
    }

    #[benchmark]
    fn submit_proof_16() {
        // setup code
        let caller = whitelisted_caller();
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
            None,
        );
    }

    #[benchmark]
    fn submit_proof_32() {
        // setup code
        let caller = whitelisted_caller();
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
            None,
        );
    }

    #[benchmark]
    fn submit_proof_with_vk_hash() {
        // setup code
        let caller = whitelisted_caller();
        let proof = VALID_PROOF.to_vec().into();
        let pubs = public_input();
        let vk = VALID_VK;
        let hash = sp_core::H256::repeat_byte(2);
        Vks::<T, Ultraplonk<T>>::insert(hash, vk);

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_hash(hash),
            proof,
            pubs.into(),
            None,
        );
    }

    #[benchmark]
    fn submit_proof_32_with_vk_hash() {
        // setup code
        let caller = whitelisted_caller();
        let proof = include_bytes!("resources/32_proof").to_vec().into();
        let pubs: Vec<_> = include_bytes!("resources/32_pubs")
            .chunks_exact(crate::PUBS_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        let vk = *include_bytes!("resources/32_vk");
        let hash = sp_core::H256::repeat_byte(2);
        Vks::<T, Ultraplonk<T>>::insert(hash, vk);

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_hash(hash),
            proof,
            pubs.into(),
            None,
        );
    }

    #[benchmark]
    fn register_vk() {
        // setup code
        let caller = whitelisted_caller();
        let vk = VALID_VK;

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.clone().into());

        // Verify
        assert!(Vks::<T, Ultraplonk<T>>::get(Ultraplonk::<T>::vk_hash(&vk)).is_some());
    }

    // WE CANNOT IMPLEMENT TESTS FOR BENCHMARKS FOR THIS PALLET
    // That's because thie pallet need that the tests are not run in parallel
    // (we use `serial_test` crate to achieve this) but using this macro doesn't
    // give a way to achieve this. When we'll write our own maybe we'll can do it.
    // impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
}
