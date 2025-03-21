#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::borrow::Cow;
use frame_support::weights::Weight;

/// Error type for verification operations
#[derive(Debug, Eq, PartialEq)]
pub enum VerifyError {
    InvalidInput,
    InvalidProofData,
    InvalidVerificationKey,
    VerifyError,
}

/// Trait for implementing verifier logic
pub trait Verifier {
    type Proof;
    type Pubs;
    type Vk;

    fn hash_context_data() -> &'static [u8];
    
    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError>;

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError>;
    
    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]>;
}

/// Weight information for verifier operations
pub trait WeightInfo<T> {
    fn store_verifying_key() -> Weight;
    fn verify_proof(proof: &[u8], public_inputs: &[u8]) -> Weight;
}
