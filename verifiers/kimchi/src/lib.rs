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
mod builtin_opening;
mod builtin_srs;
mod profile;
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
use rand_chacha::ChaCha20Rng;
use rand_core::{CryptoRng, RngCore, SeedableRng};

pub use crate::vk::{KimchiProfileId, KimchiVk as Vk};
pub use crate::weight::WeightInfo;
pub use crate::weight_verify_proof::WeightInfo as WeightInfoVerifyProof;

pub const PUB_SIZE: usize = 32;

pub type Proof = Vec<u8>;
pub type Pubs = Vec<[u8; PUB_SIZE]>;

use crate::profile::{KimchiProfile, Vesta16};
use crate::{builtin_opening::BuiltinOpeningProof, builtin_srs::BuiltinSrs};

type Vesta16OpeningProof = BuiltinOpeningProof;
type Vesta16Proof = ProverProof<Vesta, Vesta16OpeningProof, FULL_ROUNDS>;
type Vesta16Srs = BuiltinSrs;
type Vesta16VerifierIndex = VerifierIndex<FULL_ROUNDS, Vesta, Vesta16Srs>;

pub trait Config: 'static {
    /// Maximum number of bytes contained in the proof.
    type MaxProofSize: frame_support::traits::Get<u32>;
    /// Maximum number of public inputs.
    type MaxPubs: frame_support::traits::Get<u32>;
    /// Maximum number of bytes contained in the verifier index payload.
    type MaxVkSize: frame_support::traits::Get<u32>;
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
}

impl<T: Config> Vk<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        if self.verifier_index_bytes.is_empty()
            || self.verifier_index_bytes.len() > T::max_vk_size() as usize
        {
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

    fn verifier_version_hash(_proof: &Self::Proof) -> sp_core::H256 {
        sp_io::hashing::sha2_256(
            b"kimchi:v3:vesta16:bincode2-canonical:builtin-srs:chunks4-pubs1024",
        )
        .into()
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
        let mut verifier_index = decode_vk(vk)?;
        ensure!(
            raw_pubs.len() == verifier_index.public,
            VerifyError::InvalidInput
        );
        prepare_verifier_index(&mut verifier_index, vk.profile)?;
        let public_input = decode_public_input(raw_pubs)?;
        profile::validate_proof(vk.profile, &proof, &verifier_index)
            .inspect_err(|error| log::debug!("Unsupported Kimchi proof profile: {error:?}"))
            .map_err(|_| VerifyError::InvalidProofData)?;
        let verify_weight = compute_verify_weight::<T>(&verifier_index);

        let mut rng = make_rng(vk, raw_proof, raw_pubs);
        verify_vesta16_proof_with_rng(&verifier_index, &proof, &public_input, &mut rng)?;

        Ok(Some(verify_weight))
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        vk.validate_size()?;

        let mut verifier_index = decode_vk(vk)?;
        prepare_verifier_index(&mut verifier_index, vk.profile)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        let data = pubs
            .iter()
            .flat_map(|field| field.iter().copied())
            .collect::<Vec<_>>();

        Cow::Owned(data)
    }
}

fn compute_verify_weight<T: Config>(verifier_index: &Vesta16VerifierIndex) -> Weight {
    let domain_size = usize::try_from(verifier_index.domain.size).unwrap_or(usize::MAX);
    let num_chunks =
        profile::expected_chunks(verifier_index.max_poly_size, domain_size).unwrap_or(usize::MAX);

    let base = if is_small_domain_weight_eligible(verifier_index, domain_size, num_chunks) {
        T::WeightInfo::verify_proof_domain_4096_pubs_0()
    } else if num_chunks <= 1 {
        T::WeightInfo::verify_proof_domain_65536_pubs_0()
    } else if num_chunks <= 2 {
        T::WeightInfo::verify_proof_domain_131072_pubs_0()
    } else {
        T::WeightInfo::verify_proof_domain_262144_pubs_0()
    };

    base.saturating_add(public_input_weight::<T>(verifier_index.public, num_chunks))
}

fn is_small_domain_weight_eligible(
    verifier_index: &Vesta16VerifierIndex,
    domain_size: usize,
    num_chunks: usize,
) -> bool {
    domain_size <= 4096
        && verifier_index.max_poly_size <= 4096
        && num_chunks == 1
        && !has_optional_features(verifier_index)
}

fn has_optional_features(verifier_index: &Vesta16VerifierIndex) -> bool {
    verifier_index.range_check0_comm.is_some()
        || verifier_index.range_check1_comm.is_some()
        || verifier_index.foreign_field_add_comm.is_some()
        || verifier_index.foreign_field_mul_comm.is_some()
        || verifier_index.xor_comm.is_some()
        || verifier_index.rot_comm.is_some()
        || verifier_index.lookup_index.is_some()
}

fn public_input_weight<T: Config>(public_inputs: usize, num_chunks: usize) -> Weight {
    let public_inputs = u64::try_from(public_inputs).unwrap_or(u64::MAX);
    let num_chunks = u64::try_from(num_chunks).unwrap_or(u64::MAX).max(1);
    T::WeightInfo::verify_proof_public_input()
        .saturating_mul(public_inputs)
        .saturating_mul(num_chunks)
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

fn decode_proof(bytes: &[u8]) -> Result<Vesta16Proof, VerifyError> {
    let config = bincode::config::standard().with_limit::<{ Vesta16::MAX_DECODED_PROOF_BYTES }>();
    let (proof, consumed): (Vesta16Proof, usize) = bincode::serde::decode_from_slice(bytes, config)
        .inspect_err(|error| log::debug!("Cannot decode Kimchi proof bytes: {error}"))
        .map_err(|_| VerifyError::InvalidProofData)?;
    ensure!(consumed == bytes.len(), VerifyError::InvalidProofData);
    ensure!(
        bincode::serde::encode_to_vec(&proof, bincode::config::standard())
            .is_ok_and(|encoded| encoded == bytes),
        VerifyError::InvalidProofData
    );
    Ok(proof)
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

fn decode_vk<T: Config>(vk: &Vk<T>) -> Result<Vesta16VerifierIndex, VerifyError> {
    let config = bincode::config::standard().with_limit::<{ Vesta16::MAX_DECODED_VK_BYTES }>();
    let (verifier_index, consumed): (Vesta16VerifierIndex, usize) =
        bincode::serde::decode_from_slice(&vk.verifier_index_bytes, config)
            .inspect_err(|error| log::debug!("Cannot decode Kimchi verifier index: {error}"))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;
    ensure!(
        consumed == vk.verifier_index_bytes.len(),
        VerifyError::InvalidVerificationKey
    );
    ensure!(
        bincode::serde::encode_to_vec(&verifier_index, bincode::config::standard())
            .is_ok_and(|encoded| encoded == vk.verifier_index_bytes),
        VerifyError::InvalidVerificationKey
    );

    Ok(verifier_index)
}

fn prepare_verifier_index(
    verifier_index: &mut Vesta16VerifierIndex,
    profile: KimchiProfileId,
) -> Result<(), VerifyError> {
    profile::validate_verifier_index(profile, verifier_index)
        .inspect_err(|error| log::debug!("Unsupported Kimchi verifier index profile: {error:?}"))
        .map_err(|_| VerifyError::InvalidVerificationKey)?;
    let domain_size = usize::try_from(verifier_index.domain.size)
        .map_err(|_| VerifyError::InvalidVerificationKey)?;

    if verifier_index.max_poly_size == 0
        || !verifier_index.max_poly_size.is_power_of_two()
        || profile.max_poly_size() < verifier_index.max_poly_size
        || domain_size < profile.min_domain_size()
        || profile.max_domain_size() < domain_size
        || profile.max_public_inputs() < verifier_index.public
    {
        return Err(VerifyError::InvalidVerificationKey);
    }

    verifier_index.srs = builtin_srs(
        profile,
        verifier_index.max_poly_size,
        domain_size,
        verifier_index.public,
    )?;

    prepare_verifier_index_metadata(verifier_index)
}

fn builtin_srs(
    profile: KimchiProfileId,
    max_poly_size: usize,
    domain_size: usize,
    public_inputs: usize,
) -> Result<Arc<Vesta16Srs>, VerifyError> {
    Vesta16Srs::load(profile, max_poly_size, domain_size, public_inputs)
        .inspect_err(|_| log::debug!("Cannot load built-in Kimchi SRS parameters"))
        .map(Arc::new)
        .map_err(|_| VerifyError::InvalidVerificationKey)
}

fn prepare_verifier_index_metadata<Srs>(
    verifier_index: &mut VerifierIndex<FULL_ROUNDS, Vesta, Srs>,
) -> Result<(), VerifyError> {
    let feature_flags = compute_feature_flags(verifier_index);
    let (linearization, powers_of_alpha) = expr_linearization(Some(&feature_flags), true);
    let (endo_q, _endo_r) = poly_commitment::ipa::endos::<Pallas>();
    let domain = verifier_index.domain;
    let zk_rows = verifier_index.zk_rows;

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

fn verify_vesta16_proof_with_rng<R>(
    verifier_index: &Vesta16VerifierIndex,
    proof: &Vesta16Proof,
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
        Vesta16OpeningProof,
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
    let seed =
        sp_io::hashing::blake2_256(&(b"kimchi:v1:vesta16:verify-rng", vk, proof, pubs).encode());
    ChaCha20Rng::from_seed(seed)
}

fn compute_feature_flags<Srs>(
    verifier_index: &VerifierIndex<FULL_ROUNDS, Vesta, Srs>,
) -> FeatureFlags {
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
