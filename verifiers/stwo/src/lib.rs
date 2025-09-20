// Copyright 2024, zkVerify Contributors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod benchmarking;
mod stwo;
mod verifier_should;
mod weight;

use alloc::{borrow::Cow, vec::Vec};
use core::marker::PhantomData;
use frame_support::pallet_prelude::Weight;
use hp_verifiers::{Verifier, VerifyError};

pub const MAX_NUM_INPUTS: u32 = 64;
pub use weight::WeightInfo;

pub trait Config {
    /// Maximum supported number of public inputs.
    const MAX_NUM_INPUTS: u32;
}

/// STARK verification key for Stwo proofs
#[derive(Clone, Debug, PartialEq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub struct StwoVerificationKey {
    pub domain_size: u32,
    pub constraint_count: u32,
    pub public_input_count: u32,
    pub fri_lde_degree: u32,
    pub fri_last_layer_degree_bound: u32,
    pub fri_n_queries: u32,
    pub fri_commitment_merkle_tree_depth: u32,
    pub fri_lde_commitment_merkle_tree_depth: u32,
    pub fri_lde_commitment_merkle_tree_root: Vec<u8>,
    pub fri_query_commitments_crc: u32,
    pub fri_lde_commitments_crc: u32,
    pub constraint_polynomials_info: Vec<u8>,
    pub public_input_polynomials_info: Vec<u8>,
    pub composition_polynomial_info: Vec<u8>,
    pub n_verifier_friendly_commitment_hashes: u32,
    pub verifier_friendly_commitment_hashes: Vec<Vec<u8>>,
}

impl codec::MaxEncodedLen for StwoVerificationKey {
    fn max_encoded_len() -> usize {
        // Conservative estimate for all fields
        4 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + // u32 fields
        32 + // fri_lde_commitment_merkle_tree_root (32 bytes)
        4 + 4 + // CRC fields
        1024 + // constraint_polynomials_info (max 1KB)
        1024 + // public_input_polynomials_info (max 1KB)
        1024 + // composition_polynomial_info (max 1KB)
        4 + // n_verifier_friendly_commitment_hashes
        64 * 32 // verifier_friendly_commitment_hashes (max 64 hashes of 32 bytes each)
    }
}

/// STARK proof for Stwo
#[derive(Clone, Debug, PartialEq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub struct StwoProof {
    pub fri_proof: FriProof,
    pub trace_lde_commitment: Vec<u8>,
    pub constraint_polynomials_lde_commitment: Vec<u8>,
    pub public_input_polynomials_lde_commitment: Vec<u8>,
    pub composition_polynomial_lde_commitment: Vec<u8>,
    pub trace_lde_commitment_merkle_tree_root: Vec<u8>,
    pub constraint_polynomials_lde_commitment_merkle_tree_root: Vec<u8>,
    pub public_input_polynomials_lde_commitment_merkle_tree_root: Vec<u8>,
    pub composition_polynomial_lde_commitment_merkle_tree_root: Vec<u8>,
    pub trace_lde_commitment_merkle_tree_path: Vec<Vec<u8>>,
    pub constraint_polynomials_lde_commitment_merkle_tree_path: Vec<Vec<u8>>,
    pub public_input_polynomials_lde_commitment_merkle_tree_path: Vec<Vec<u8>>,
    pub composition_polynomial_lde_commitment_merkle_tree_path: Vec<Vec<u8>>,
    pub trace_lde_commitment_merkle_tree_leaf_index: u32,
    pub constraint_polynomials_lde_commitment_merkle_tree_leaf_index: u32,
    pub public_input_polynomials_lde_commitment_merkle_tree_leaf_index: u32,
    pub composition_polynomial_lde_commitment_merkle_tree_leaf_index: u32,
}

/// FRI proof structure
#[derive(Clone, Debug, PartialEq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub struct FriProof {
    pub fri_lde_commitment: Vec<u8>,
    pub fri_lde_commitment_merkle_tree_root: Vec<u8>,
    pub fri_lde_commitment_merkle_tree_path: Vec<Vec<u8>>,
    pub fri_lde_commitment_merkle_tree_leaf_index: u32,
    pub fri_query_proofs: Vec<FriQueryProof>,
}

/// FRI query proof structure
#[derive(Clone, Debug, PartialEq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub struct FriQueryProof {
    pub fri_layer_proofs: Vec<FriLayerProof>,
}

/// FRI layer proof structure
#[derive(Clone, Debug, PartialEq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub struct FriLayerProof {
    pub fri_layer_commitment: Vec<u8>,
    pub fri_layer_commitment_merkle_tree_root: Vec<u8>,
    pub fri_layer_commitment_merkle_tree_path: Vec<Vec<u8>>,
    pub fri_layer_commitment_merkle_tree_leaf_index: u32,
    pub fri_layer_value: Vec<u8>,
}

/// Public inputs for STARK verification
#[derive(Clone, Debug, PartialEq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub struct StwoPublicInputs {
    pub inputs: Vec<u8>,
}

#[pallet_verifiers::verifier]
pub struct Stwo<T>;
pub type Pubs = StwoPublicInputs;

impl<T: Config> Verifier for Stwo<T> {
    type Proof = StwoProof;
    type Pubs = Pubs;
    type Vk = StwoVerificationKey;

    fn hash_context_data() -> &'static [u8] {
        b"stwo"
    }

    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        if pubs.inputs.len() > T::MAX_NUM_INPUTS as usize {
            return Err(VerifyError::InvalidInput);
        }

        stwo::StwoVerifier::verify_proof(vk, proof, pubs)
            .and_then(|r| r.then_some(()).ok_or(VerifyError::VerifyError))
            .map(|_| None)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        Cow::Borrowed(&pubs.inputs)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        stwo::StwoVerifier::validate_vk(vk).map_err(Into::into)
    }
}

/// The struct to use in runtime pallet configuration to map the weight computed by this crate
/// benchmarks to the weight needed by the `pallet-verifiers`.
pub struct StwoWeight<W: WeightInfo>(PhantomData<W>);

impl<T: Config, W: WeightInfo> pallet_verifiers::WeightInfo<Stwo<T>> for StwoWeight<W> {
    fn register_vk(vk: &<Stwo<T> as Verifier>::Vk) -> Weight {
        let n = vk.public_input_count.min(T::MAX_NUM_INPUTS);
        W::register_vk(n)
    }

    fn unregister_vk() -> Weight {
        W::unregister_vk()
    }

    fn verify_proof(
        proof: &<Stwo<T> as Verifier>::Proof,
        pubs: &<Stwo<T> as Verifier>::Pubs,
    ) -> Weight {
        let n = pubs.inputs.len().min(T::MAX_NUM_INPUTS as usize) as u32;
        W::verify_proof(n)
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(vk: &<Stwo<T> as Verifier>::Vk) -> Weight {
        let n = vk.public_input_count.min(T::MAX_NUM_INPUTS);
        W::validate_vk(n)
    }

    fn compute_statement_hash(
        proof: &<Stwo<T> as Verifier>::Proof,
        pubs: &<Stwo<T> as Verifier>::Pubs,
    ) -> Weight {
        let n = pubs.inputs.len().min(T::MAX_NUM_INPUTS as usize) as u32;
        W::compute_statement_hash(n)
    }
}