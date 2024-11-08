#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::weights::Weight;
use plonky2_verifier::validate::validate_vk_default_poseidon;
use plonky2_verifier::verify_default_poseidon;
use hp_verifiers::{Cow, Verifier, VerifyError};

pub mod benchmarking;
mod verifier_should;
mod weight;

pub use weight::WeightInfo;

pub type Pubs = Vec<u8>;
pub type Proof = Vec<u8>;
pub type Vk = Vec<u8>;

#[pallet_verifiers::verifier]
pub struct Plonky2;

impl Verifier for Plonky2 {
    type Proof = Proof;

    type Pubs = Pubs;

    type Vk = Vk;

    fn hash_context_data() -> &'static [u8] {
        b"plonky2"
    }

    fn verify_proof(
        vk: &Self::Vk,
        raw_proof: &Self::Proof,
        raw_pubs: &Self::Pubs,
    ) -> Result<(), VerifyError> {
        verify_default_poseidon(vk, raw_proof, raw_pubs)
            .map_err(|e| log::debug!("Proof verification failed: {:?}", e))
            .map_err(|_| VerifyError::VerifyError)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
       validate_vk_default_poseidon(vk)
           .map_err(|e| log::debug!("VK validation failed: {:?}", e))
           .map_err(|_| VerifyError::InvalidVerificationKey)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]> {
        Cow::Borrowed(pubs)
    }
}

pub struct Plonky2Weight<W: weight::WeightInfo>(PhantomData<W>);

impl<W: weight::WeightInfo> pallet_verifiers::WeightInfo<Plonky2> for Plonky2Weight<W> {
    fn submit_proof(
        _proof: &<Plonky2 as hp_verifiers::Verifier>::Proof,
        _pubs: &<Plonky2 as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::submit_proof()
    }

    fn submit_proof_with_vk_hash(
        _proof: &<Plonky2 as hp_verifiers::Verifier>::Proof,
        _pubs: &<Plonky2 as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::submit_proof_with_vk_hash()
    }

    fn register_vk(_vk: &<Plonky2 as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }
}
