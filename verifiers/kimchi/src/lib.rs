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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod benchmarking;
pub mod benchmarking_verify_proof;
mod verifier_should;
mod vk;
mod weight;
mod weight_verify_proof;

use alloc::{borrow::Cow, sync::Arc, vec::Vec};
#[cfg(not(feature = "std"))]
use core::cell::OnceCell as OnceLock;
use core::marker::PhantomData;
#[cfg(feature = "std")]
use std::sync::OnceLock;

use ark_ff::MontFp;
use ark_serialize::CanonicalDeserialize;
use codec::Encode;
use frame_support::{ensure, traits::Get, weights::Weight};
use kimchi::{
    circuits::{
        constraints::FeatureFlags,
        lookup::lookups::{LookupFeatures, LookupPatterns},
        polynomials::permutation::{permutation_vanishing_polynomial, zk_w},
    },
    error::VerifyError as KimchiVerifyError,
    groupmap::BWParameters,
    linearization::expr_linearization,
    proof::ProverProof,
    verifier::verify_with_rng,
    verifier_index::VerifierIndex,
};
use mina_curves::pasta::{Fp, Pallas, Vesta, VestaParameters};
use mina_poseidon::{
    constants::PlonkSpongeConstantsKimchi,
    pasta::FULL_ROUNDS,
    sponge::{DefaultFqSponge, DefaultFrSponge},
};
use pallet_verifiers::traits::{Verifier, VerifyError};
use poly_commitment::{
    ipa::{OpeningProof, SRS as IpaSrs},
    SRS as SrsTrait,
};
use rand_chacha::ChaCha20Rng;
use rand_core::{CryptoRng, RngCore, SeedableRng};

pub use crate::vk::KimchiVk as Vk;
pub use crate::weight::WeightInfo;
pub use crate::weight_verify_proof::WeightInfo as WeightInfoVerifyProof;

pub const PUB_SIZE: usize = 32;
pub const MAX_BENCHMARKED_DOMAIN_SIZE: usize = 4096;

pub type Proof = Vec<u8>;
pub type Pubs = Vec<[u8; PUB_SIZE]>;

type NativeOpeningProof = OpeningProof<Vesta, FULL_ROUNDS>;
type NativeProof = ProverProof<Vesta, NativeOpeningProof, FULL_ROUNDS>;
type NativeSrs = IpaSrs<Vesta>;
type NativeVerifierIndex = VerifierIndex<FULL_ROUNDS, Vesta, NativeSrs>;

pub trait Config: 'static {
    /// Maximum number of bytes contained in the proof.
    type MaxProofSize: frame_support::traits::Get<u32>;
    /// Maximum number of public inputs.
    type MaxPubs: frame_support::traits::Get<u32>;
    /// Maximum number of bytes contained in the verifier index payload.
    type MaxVkSize: frame_support::traits::Get<u32>;
    /// Maximum number of bytes contained in the serialized SRS payload.
    type MaxSrsSize: frame_support::traits::Get<u32>;
    /// Parameterized weights for Kimchi proof verification.
    type WeightInfo: WeightInfoVerifyProof;

    fn max_proof_size() -> u32 {
        Self::MaxProofSize::get()
    }

    fn max_pubs() -> u32 {
        Self::MaxPubs::get()
    }

    fn max_vk_size() -> u32 {
        Self::MaxVkSize::get()
    }

    fn max_srs_size() -> u32 {
        Self::MaxSrsSize::get()
    }

    fn max_verify_proof_weight() -> Weight {
        Self::WeightInfo::verify_proof_domain_4096()
    }
}

impl<T: Config> Vk<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        if self.verifier_index_bytes.is_empty()
            || self.verifier_index_bytes.len() > T::max_vk_size() as usize
        {
            return Err(VerifyError::InvalidVerificationKey);
        }

        if self.srs_bytes.is_empty() || self.srs_bytes.len() > T::max_srs_size() as usize {
            return Err(VerifyError::InvalidVerificationKey);
        }

        Ok(())
    }
}

#[pallet_verifiers::verifier]
pub struct Kimchi<T>;

impl<T: Config> Verifier for Kimchi<T> {
    type Proof = Proof;
    type Pubs = Pubs;
    type Vk = Vk<T>;

    fn hash_context_data() -> &'static [u8] {
        b"kimchi"
    }

    fn verify_proof(
        vk: &Self::Vk,
        raw_proof: &Self::Proof,
        raw_pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        vk.validate_size()?;
        ensure!(
            raw_proof.len() <= T::max_proof_size() as usize,
            VerifyError::InvalidProofData
        );
        ensure!(
            raw_pubs.len() <= T::max_pubs() as usize,
            VerifyError::InvalidInput
        );

        let proof = decode_proof(raw_proof)?;
        let public_input = decode_public_input(raw_pubs)?;
        let (mut verifier_index, srs) = decode_vk(vk)?;
        prepare_verifier_index(&mut verifier_index, srs)?;
        let verify_weight = compute_verify_weight::<T>(&verifier_index);

        let mut rng = make_rng(vk, raw_proof, raw_pubs);
        verify_native_proof_with_rng(&verifier_index, &proof, &public_input, &mut rng)?;

        Ok(Some(verify_weight))
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        vk.validate_size()?;

        let (mut verifier_index, srs) = decode_vk(vk)?;
        prepare_verifier_index(&mut verifier_index, srs)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        let data = pubs
            .iter()
            .flat_map(|field| field.iter().copied())
            .collect::<Vec<_>>();

        Cow::Owned(data)
    }
}

fn compute_verify_weight<T: Config>(verifier_index: &NativeVerifierIndex) -> Weight {
    if verifier_index.max_poly_size <= MAX_BENCHMARKED_DOMAIN_SIZE {
        T::WeightInfo::verify_proof_domain_4096()
    } else {
        T::max_verify_proof_weight()
    }
}

pub struct KimchiWeight<W: WeightInfo>(PhantomData<W>);

impl<T: Config, W: WeightInfo> pallet_verifiers::WeightInfo<Kimchi<T>> for KimchiWeight<W> {
    fn verify_proof(
        _proof: &<Kimchi<T> as Verifier>::Proof,
        _pubs: &<Kimchi<T> as Verifier>::Pubs,
    ) -> Weight {
        W::verify_proof()
    }

    fn register_vk(_vk: &<Kimchi<T> as Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Kimchi<T> as Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Kimchi<T> as Verifier>::Proof,
        _pubs: &<Kimchi<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}

fn decode_proof(bytes: &[u8]) -> Result<NativeProof, VerifyError> {
    bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .map(|(proof, _)| proof)
        .inspect_err(|error| log::debug!("Cannot decode Kimchi proof bytes: {error}"))
        .map_err(|_| VerifyError::InvalidProofData)
}

fn decode_public_input(raw_pubs: &Pubs) -> Result<Vec<Fp>, VerifyError> {
    raw_pubs
        .iter()
        .map(|bytes| {
            Fp::deserialize_compressed(&bytes[..])
                .inspect_err(|error| log::debug!("Cannot decode Kimchi public input: {error}"))
                .map_err(|_| VerifyError::InvalidInput)
        })
        .collect()
}

fn decode_vk<T: Config>(vk: &Vk<T>) -> Result<(NativeVerifierIndex, Arc<NativeSrs>), VerifyError> {
    let verifier_index: NativeVerifierIndex =
        bincode::serde::decode_from_slice(&vk.verifier_index_bytes, bincode::config::standard())
            .map(|(verifier_index, _)| verifier_index)
            .inspect_err(|error| log::debug!("Cannot decode Kimchi verifier index: {error}"))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;
    let srs: NativeSrs =
        bincode::serde::decode_from_slice(&vk.srs_bytes, bincode::config::standard())
            .map(|(srs, _)| srs)
            .inspect_err(|error| log::debug!("Cannot decode Kimchi SRS: {error}"))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;

    Ok((verifier_index, Arc::new(srs)))
}

fn prepare_verifier_index(
    verifier_index: &mut NativeVerifierIndex,
    srs: Arc<NativeSrs>,
) -> Result<(), VerifyError> {
    if srs.max_poly_size() < verifier_index.max_poly_size {
        return Err(VerifyError::InvalidVerificationKey);
    }

    let feature_flags = compute_feature_flags(verifier_index);
    let (linearization, powers_of_alpha) = expr_linearization(Some(&feature_flags), true);
    let (endo_q, _endo_r) = poly_commitment::ipa::endos::<Pallas>();
    let domain = verifier_index.domain;
    let zk_rows = verifier_index.zk_rows;

    verifier_index.srs = srs;
    verifier_index.endo = endo_q;
    verifier_index.linearization = linearization;
    verifier_index.powers_of_alpha = powers_of_alpha;

    let w = OnceLock::new();
    w.set(zk_w(domain, zk_rows))
        .map_err(|_| VerifyError::InvalidVerificationKey)?;
    verifier_index.w = w;

    let permutation_vanishing_polynomial_m = OnceLock::new();
    permutation_vanishing_polynomial_m
        .set(permutation_vanishing_polynomial(domain, zk_rows))
        .map_err(|_| VerifyError::InvalidVerificationKey)?;
    verifier_index.permutation_vanishing_polynomial_m = permutation_vanishing_polynomial_m;

    Ok(())
}

fn verify_native_proof_with_rng<R>(
    verifier_index: &NativeVerifierIndex,
    proof: &NativeProof,
    public_input: &[Fp],
    rng: &mut R,
) -> Result<(), VerifyError>
where
    R: CryptoRng + RngCore,
{
    let group_map = vesta_group_map();

    verify_with_rng::<
        FULL_ROUNDS,
        Vesta,
        DefaultFqSponge<VestaParameters, PlonkSpongeConstantsKimchi, FULL_ROUNDS>,
        DefaultFrSponge<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>,
        NativeOpeningProof,
        R,
    >(&group_map, verifier_index, proof, public_input, rng)
    .inspect_err(|error| log::debug!("Kimchi verification failed: {error:?}"))
    .map_err(map_kimchi_verify_error)
}

fn vesta_group_map() -> BWParameters<VestaParameters> {
    BWParameters {
        u: MontFp!("1"),
        fu: MontFp!("6"),
        sqrt_neg_three_u_squared_minus_u_over_2: MontFp!(
            "2942865608506852014473558576493638302197734138389222805617480874486368177743"
        ),
        sqrt_neg_three_u_squared: MontFp!(
            "5885731217013704028947117152987276604395468276778445611234961748972736355487"
        ),
        inv_three_u_squared: MontFp!(
            "19298681539552699237261830834781317975575370987961098253119828498928908632065"
        ),
    }
}

fn map_kimchi_verify_error(error: KimchiVerifyError) -> VerifyError {
    match error {
        KimchiVerifyError::IncorrectPubicInputLength(_) => VerifyError::InvalidInput,
        KimchiVerifyError::IncorrectCommitmentLength(_, _, _)
        | KimchiVerifyError::IncorrectPrevChallengesLength(_, _)
        | KimchiVerifyError::IncorrectEvaluationsLength(_, _, _)
        | KimchiVerifyError::LookupCommitmentMissing
        | KimchiVerifyError::LookupEvalsMissing
        | KimchiVerifyError::ProofInconsistentLookup
        | KimchiVerifyError::IncorrectRuntimeProof
        | KimchiVerifyError::MissingEvaluation(_)
        | KimchiVerifyError::MissingPublicInputEvaluation
        | KimchiVerifyError::MissingCommitment(_) => VerifyError::InvalidProofData,
        KimchiVerifyError::DifferentSRS | KimchiVerifyError::SRSTooSmall => {
            VerifyError::InvalidVerificationKey
        }
        KimchiVerifyError::OpenProof => VerifyError::VerifyError,
    }
}

fn make_rng<T: Config>(vk: &Vk<T>, proof: &Proof, pubs: &Pubs) -> ChaCha20Rng {
    // Kimchi verification needs RNG input for batch-combination scalars. Hashing
    // the statement material keeps runtime execution deterministic across nodes.
    let seed = sp_io::hashing::blake2_256(&(vk, proof, pubs).encode());
    ChaCha20Rng::from_seed(seed)
}

fn compute_feature_flags(verifier_index: &NativeVerifierIndex) -> FeatureFlags {
    let xor = verifier_index.xor_comm.is_some();
    let range_check0 = verifier_index.range_check0_comm.is_some();
    let range_check1 = verifier_index.range_check1_comm.is_some();
    let foreign_field_add = verifier_index.foreign_field_add_comm.is_some();
    let foreign_field_mul = verifier_index.foreign_field_mul_comm.is_some();
    let rot = verifier_index.rot_comm.is_some();

    let lookup = verifier_index
        .lookup_index
        .as_ref()
        .is_some_and(|lookup_index| lookup_index.lookup_info.features.patterns.lookup);

    let runtime_tables = verifier_index
        .lookup_index
        .as_ref()
        .is_some_and(|lookup_index| lookup_index.runtime_tables_selector.is_some());

    let patterns = LookupPatterns {
        xor,
        lookup,
        range_check: range_check0 || range_check1 || rot,
        foreign_field_mul,
    };

    FeatureFlags {
        range_check0,
        range_check1,
        foreign_field_add,
        foreign_field_mul,
        xor,
        rot,
        lookup_features: LookupFeatures {
            patterns,
            joint_lookup_used: patterns.joint_lookups_used(),
            uses_runtime_tables: runtime_tables,
        },
    }
}
