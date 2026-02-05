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
#![deny(missing_docs)]

//! The traits and basic implementations for the verifier pallets based on `pallet-verifiers`
//! parametric pallet.

extern crate alloc;

use alloc::borrow::Cow;
use codec::{Decode, DecodeWithMemTracking, Encode, EncodeLike};
use core::fmt::Debug;
use hex_literal::hex;
use scale_info::TypeInfo;
use sp_core::{MaxEncodedLen, H256};
use sp_weights::Weight;

/// No version provided.
pub const NO_VERSION_HASH: H256 = H256(hex!(
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" // = SHA256("")
));

/// Define the minimum traits that proofs and public inputs should implement.
pub trait Arg: Debug + Clone + PartialEq + Encode + Decode + DecodeWithMemTracking + TypeInfo {}
impl<T: Debug + Clone + PartialEq + Encode + Decode + DecodeWithMemTracking + TypeInfo> Arg for T {}
/// Define the minimum traits that verification keys should implement.
pub trait VkArg: Arg + MaxEncodedLen + EncodeLike {}
impl<T: Arg + MaxEncodedLen + EncodeLike> VkArg for T {}

/// The verification error type
#[derive(Debug, PartialEq)]
pub enum VerifyError {
    /// Provided data has not valid public inputs.
    InvalidInput,
    /// Provided data has not valid proof.
    InvalidProofData,
    /// Verify proof failed.
    VerifyError,
    /// Provided an invalid verification key.
    InvalidVerificationKey,
    /// Unsupported Version.
    UnsupportedVersion,
}

/// The trait that characterizes a verifier.
pub trait Verifier {
    /// The proof format type accepted by the verifier
    type Proof: Arg;
    /// The public inputs format
    type Pubs: Arg;
    /// The verification key format
    type Vk: VkArg;

    /// The context used to generate the statement hash.
    fn hash_context_data() -> &'static [u8];

    /// Verify the proof: Should return `Ok(post_info_weight)` if the proof is coherent with
    /// the verification key, and it's valid against the provided public inputs `pubs`.
    ///
    /// The `post_info_weight` should contain an optional weight that will replace the original
    /// estimated weight if it's lower.
    ///
    /// It's useful to leverage on `post_info_weight` every time that you cannot compute good weight
    /// estimation without parsing the proof: so you can provide an upper-bound estimation based on
    /// a simple `proof` and `pubs` analysis with `WeightInfo::<V>::verify_proof()` and then return
    /// a more precise value here.
    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError>;

    /// Validate the verification key: Should return `Ok(())` if the verification key is valid.
    /// The default implementations accept all verification keys: our business logic could
    /// need something different.
    fn validate_vk(_vk: &Self::Vk) -> Result<(), VerifyError> {
        Ok(())
    }

    /// How to compute the verification key hash to use in statement hash computation.
    fn vk_hash(vk: &Self::Vk) -> H256 {
        sp_io::hashing::keccak_256(&Self::vk_bytes(vk)).into()
    }

    /// A vk's byte serialization used to compute the verification key hash. The default implementation
    /// uses the `scale::encode()` one, but you can customize it.
    fn vk_bytes(vk: &Self::Vk) -> Cow<'_, [u8]> {
        Cow::Owned(vk.encode())
    }

    /// Public inputs byte serialization used to compute the statement hash.
    /// There isn't any default implementation: you should implement it.
    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]>;

    /// Return a hash that represents the verifier version used to verify the proof.
    /// This value, if present, will be included in the computation of the statement hash.
    fn verifier_version_hash(_proof: &Self::Proof) -> H256 {
        NO_VERSION_HASH
    }
}

/// The trait used to map the `pallet-verifiers` extrinsic in you verifier implementation
/// weights.
///
/// The methods provide a borrowed proof, public inputs or vk, but your code should
/// use them just to guess the _size_ of your verification and map the method in the weights
/// that you computed for your own verifier implementation.
///
/// Any implementation SHOULD be as plain as possible (ideally static mappings) without
/// any logic and without any dependencies on some values that cannot be extracted
/// in a fast way like type or a vector length.
///
/// As an example of a non-trivial implementation, look at `pallet-groth16-verifier`
/// where almost all functions depend on the number of public inputs.
pub trait WeightInfo<V: Verifier> {
    /// Here you should map the given request to a weight computed with your verifier.
    fn verify_proof(proof: &V::Proof, pubs: &V::Pubs) -> Weight;

    /// Here you should map the given request to a weight computed with your verifier.
    fn register_vk(vk: &V::Vk) -> Weight;

    /// Here you should map the given unregister_vk request to the weight computed with
    /// your verifier.
    fn unregister_vk() -> Weight;

    /// The weight about retrieving the vk from `Vks` storage. You should overestimate
    /// it at the bigger vk that your pallet uses.
    fn get_vk() -> Weight;

    /// This is the weight about executing [`Verifier::validate_vk`].
    /// In the case that you cannot get enough information to estimate it correctly,
    /// you should return the worst case value.
    fn validate_vk(vk: &V::Vk) -> Weight;

    /// Estimate the weight about `pallet_verifiers`'s `compute_statement_hash()`
    /// for your verifier.
    fn compute_statement_hash(proof: &V::Proof, pubs: &V::Pubs) -> Weight;
}

/// `()` is a verifier that reject the proof and returns `VerifyError::VerifyError`.
impl Verifier for () {
    type Proof = ();
    type Pubs = ();
    type Vk = ();

    fn hash_context_data() -> &'static [u8] {
        b"()"
    }

    fn verify_proof(
        _vk: &Self::Vk,
        _proof: &Self::Proof,
        _pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        Err(VerifyError::VerifyError)
    }

    fn validate_vk(_vk: &Self::Vk) -> Result<(), VerifyError> {
        Ok(())
    }

    fn pubs_bytes(_pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        static EMPTY: [u8; 0] = [];
        // Example: If you would use something computed here, you can use
        // Cow::Owned(_pubs.encode())
        Cow::Borrowed(&EMPTY)
    }
}

#[cfg(test)]
mod unit_verifier {
    use super::*;

    #[test]
    fn should_raise_error() {
        assert_eq!(
            VerifyError::VerifyError,
            <() as Verifier>::verify_proof(&(), &(), &()).unwrap_err()
        )
    }
}
