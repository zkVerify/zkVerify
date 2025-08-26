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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod benchmarking;
mod verifier_should;
mod weight;

use alloc::{borrow::Cow, vec::Vec};
use core::marker::PhantomData;

use frame_support::weights::Weight;
use hp_verifiers::Verifier;
use sp1_zkv_verifier::Proof as SP1Proof;
use sp_core::{Get, H256};
pub use weight::WeightInfo;

#[pallet_verifiers::verifier]
pub struct Sp1<T>;

pub type Proof = Vec<u8>;
pub type Pubs = Vec<u8>;

const MAX_PROOF_SIZE: usize = 786432;

pub trait Config {
    /// Maximum number of bytes contained in the public inputs (otherwise rejected)
    type MaxPubsSize: Get<u32>;

    fn max_pubs_size() -> u32 {
        Self::MaxPubsSize::get()
    }
}

impl<T: Config> Verifier for Sp1<T> {
    type Proof = Proof;

    type Pubs = Pubs;

    type Vk = H256;

    fn hash_context_data() -> &'static [u8] {
        b"sp1"
    }

    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, hp_verifiers::VerifyError> {
        if proof.len() > MAX_PROOF_SIZE {
            log::debug!("Proof exceeds maximum size");
            Err(hp_verifiers::VerifyError::InvalidProofData)?;
        }
        if pubs.len() > T::max_pubs_size() as usize {
            log::debug!("Public input exceeds maximum size");
            Err(hp_verifiers::VerifyError::InvalidInput)?;
        }

        let proof: SP1Proof =
            bincode::serde::decode_from_slice(&proof[..], bincode::config::legacy())
                .inspect_err(|err| log::debug!("Cannot deserialize proof: {err}"))
                .map_err(|_| hp_verifiers::VerifyError::InvalidProofData)?
                .0;

        sp1_zkv_verifier::verify(vk.as_fixed_bytes(), &proof, pubs)
            .inspect_err(|err| log::debug!("Verification error: {err}"))
            .map_err(|_| hp_verifiers::VerifyError::VerifyError)?;

        Ok(None)
    }

    fn vk_hash(vk: &Self::Vk) -> H256 {
        *vk
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        Cow::Borrowed(pubs)
    }
}

pub struct Sp1Weight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Sp1<T>> for Sp1Weight<W> {
    fn verify_proof(
        _proof: &<Sp1<T> as Verifier>::Proof,
        _pubs: &<Sp1<T> as Verifier>::Pubs,
    ) -> Weight {
        W::verify_proof()
    }

    fn register_vk(_vk: &<Sp1<T> as Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Sp1<T> as Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Sp1<T> as Verifier>::Proof,
        _pubs: &<Sp1<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
