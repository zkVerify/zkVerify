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
mod groth16;
mod verifier_should;
mod weight;

use core::marker::PhantomData;
use frame_support::pallet_prelude::Weight;
use groth16::Curve;
pub use groth16::{ProofWithCurve as Proof, VerificationKeyWithCurve as Vk};
use hp_groth16::Scalar;
use hp_verifiers::{Verifier, VerifyError};
use sp_std::vec::Vec;

pub const MAX_NUM_INPUTS: u32 = 32;
pub use weight::WeightInfo;

pub trait Config: 'static {
    /// Maximum supported number of public inputs.
    const MAX_NUM_INPUTS: u32;
}

#[pallet_verifiers::verifier]
pub struct Groth16<T>;
pub type Pubs = Vec<Scalar>;

impl<T: Config> Verifier for Groth16<T> {
    type Proof = Proof;

    type Pubs = Pubs;

    type Vk = Vk;

    fn hash_context_data() -> &'static [u8] {
        b"groth16"
    }

    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        if pubs.len() > T::MAX_NUM_INPUTS as usize {
            return Err(hp_verifiers::VerifyError::InvalidInput);
        }
        if pubs.len() + 1 != vk.gamma_abc_g1.len() {
            return Err(hp_verifiers::VerifyError::InvalidInput);
        }

        groth16::Groth16::verify_proof(proof.clone().into(), vk.clone(), pubs)
            .map_err(Into::into)
            .and_then(|r| {
                r.then_some(())
                    .ok_or(hp_verifiers::VerifyError::VerifyError)
            })
            .map(|_| None)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> hp_verifiers::Cow<[u8]> {
        let data = pubs
            .iter()
            .flat_map(|s| s.0.iter().cloned())
            .collect::<Vec<_>>();
        hp_verifiers::Cow::Owned(data)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), hp_verifiers::VerifyError> {
        let curve = vk.curve;
        let vk = vk.clone().vk();
        match curve {
            Curve::Bn254 => native::groth_16_bn_254_verify::validate_key(vk),
            Curve::Bls12_381 => native::groth_16_bls_12_381_verify::validate_key(vk),
        }
        .map_err(Into::into)
    }
}

/// The struct to use in runtime pallet configuration to map the weight computed by this crate
/// benchmarks to the weight needed by the `pallet-verifiers`.
pub struct Groth16Weight<W: WeightInfo>(PhantomData<W>);

impl<T: Config, W: WeightInfo> pallet_verifiers::WeightInfo<Groth16<T>> for Groth16Weight<W> {
    fn register_vk(vk: &<Groth16<T> as Verifier>::Vk) -> frame_support::weights::Weight {
        let n = (vk.gamma_abc_g1.len().saturating_sub(1))
            .try_into()
            .expect(concat!(
                "Public inputs should be less than",
                stringify!(T::MAX_NUM_INPUTS),
                ".qed"
            ));
        match vk.curve {
            Curve::Bn254 => W::register_vk_bn254(n),
            Curve::Bls12_381 => W::register_vk_bls12_381(n),
        }
    }

    fn unregister_vk() -> frame_support::weights::Weight {
        W::unregister_vk()
    }

    fn verify_proof(
        proof: &<Groth16<T> as Verifier>::Proof,
        pubs: &<Groth16<T> as Verifier>::Pubs,
    ) -> frame_support::weights::Weight {
        let n = pubs.len().try_into().expect(concat!(
            "Public inputs should be less than",
            stringify!(T::MAX_NUM_INPUTS),
            ".qed"
        ));
        match proof.curve {
            Curve::Bn254 => W::verify_proof_bn254(n),
            Curve::Bls12_381 => W::verify_proof_bls12_381(n),
        }
    }

    fn get_vk() -> frame_support::weights::Weight {
        W::get_vk()
    }

    fn validate_vk(vk: &<Groth16<T> as Verifier>::Vk) -> Weight {
        let Vk {
            curve,
            gamma_abc_g1,
            ..
        } = vk;
        let pubs_len = gamma_abc_g1.len().saturating_sub(1) as u32;
        match curve {
            Curve::Bn254 => W::validate_vk_bn254(pubs_len),
            Curve::Bls12_381 => W::validate_vk_bls12_381(pubs_len),
        }
    }

    fn compute_statement_hash(
        proof: &<Groth16<T> as Verifier>::Proof,
        pubs: &<Groth16<T> as Verifier>::Pubs,
    ) -> frame_support::weights::Weight {
        let Proof { curve, .. } = proof;
        let pubs_len = pubs.len() as u32;
        match curve {
            Curve::Bn254 => W::compute_statement_hash(pubs_len),
            Curve::Bls12_381 => W::compute_statement_hash(pubs_len),
        }
    }
}
