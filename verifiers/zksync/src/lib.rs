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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod benchmarking;
mod verifier_should;
mod weight;

use core::marker::PhantomData;

use frame_support::weights::Weight;
use hp_verifiers::{Cow, Verifier, VerifyError};

pub use native::ZKSYNC_PROOF_SIZE as PROOF_SIZE;
pub use native::ZKSYNC_PUBS_SIZE as PUBS_SIZE;

#[pallet_verifiers::verifier]
pub struct Zksync;
pub use weight::WeightInfo;

impl Verifier for Zksync {
    type Proof = [u8; PROOF_SIZE];

    type Pubs = [u8; PUBS_SIZE];

    type Vk = ();

    fn hash_context_data() -> &'static [u8] {
        b"zksync"
    }

    fn verify_proof(
        _vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        native::zksync_verify::verify(proof, *pubs)
            .map_err(Into::into)
            .map(|_| None)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]> {
        Cow::Borrowed(pubs)
    }

    fn vk_bytes(_vk: &Self::Vk) -> Cow<[u8]> {
        // This verifier doesn't need any vk and the only admissible vk is the empty one.
        // So, to make it in the same shape of the other verifiers, we return an empty array.
        static VOID: &[u8] = &[];
        Cow::Borrowed(VOID)
    }
}

/// The struct to use in runtime pallet configuration to map the weight computed by this crate
/// benchmarks to the weight needed by the `pallet-verifiers`.
/// In this case the implementation doesn't depends from the kind of proof or public input and
/// the crate's benchmarks are mapped 1-1 to the `pallet-verifiers`'s one.
pub struct ZksyncWeight<W: weight::WeightInfo>(PhantomData<W>);

impl<W: weight::WeightInfo> pallet_verifiers::WeightInfo<Zksync> for ZksyncWeight<W> {
    fn register_vk(_vk: &<Zksync as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> frame_support::weights::Weight {
        W::unregister_vk()
    }

    fn verify_proof(
        _proof: &<Zksync as Verifier>::Proof,
        _pubs: &<Zksync as Verifier>::Pubs,
    ) -> Weight {
        W::verify_proof()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Zksync as hp_verifiers::Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Zksync as Verifier>::Proof,
        _pubs: &<Zksync as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
