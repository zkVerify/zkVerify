//! Proof to be morphed into `plonky2` variant.
use crate::Config;
use sp_std::vec::Vec;

use sp_std::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use educe::Educe;
use frame_support::pallet_prelude::TypeInfo;

// Here educe is used for Clone, Debug, and PartialEq to work around
// a long-standing compiler bug https://github.com/rust-lang/rust/issues/26925
#[derive(Educe, Encode, Decode, TypeInfo)]
#[educe(Clone, Debug, PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct Proof<T> {
    // pub compressed: bool,
    pub bytes: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<T: Config> Proof<T> {
    // pub fn new(compressed: bool, bytes: Vec<u8>) -> Self {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            // compressed,
            bytes,
            _marker: PhantomData,
        }
    }
}

impl<T: Config> MaxEncodedLen for Proof<T> {
    fn max_encoded_len() -> usize {
        bool::max_encoded_len()
            + codec::Compact(T::max_proof_size()).encoded_size()
            + T::max_proof_size() as usize
    }
}

impl<T: Config> From<Proof<T>> for plonky2_verifier::Proof {
    fn from(proof: Proof<T>) -> Self {
        Self {
            // TODO: Restore when we support compressed proofs
            compressed: false, // proof.compressed,
            bytes: proof.bytes,
        }
    }
}

impl<T: Config> Default for Proof<T> {
    fn default() -> Self {
        Self {
            // compressed: false,
            bytes: Vec::default(),
            _marker: PhantomData,
        }
    }
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
impl<T: Config> Proof<T> {
    pub(crate) fn from_default_with_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            ..Default::default()
        }
    }
}
