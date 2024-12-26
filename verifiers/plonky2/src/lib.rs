#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use educe::Educe;
use frame_support::__private::Get;
use frame_support::ensure;
use frame_support::traits::ConstU32;
use frame_support::weights::Weight;
use hp_verifiers::{Cow, Verifier, VerifyError};
use plonky2_verifier::validate::validate_vk_default_poseidon;
use plonky2_verifier::verify_default_poseidon;
use scale_info::TypeInfo;

pub mod benchmarking;
mod resources;
pub(crate) mod verifier_should;
mod weight;

pub use weight::WeightInfo;

pub type Pubs = Vec<u8>;
pub type Proof = Vec<u8>;

// Here educe is used for Clone, Debug, and PartialEq to work around
// a long-standing compiler bug https://github.com/rust-lang/rust/issues/26925
#[derive(Educe, Encode, Decode, TypeInfo)]
#[educe(Clone, Debug, PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct Vk<T>(Vec<u8>, PhantomData<T>);

impl<T> From<Vec<u8>> for Vk<T> {
    fn from(value: Vec<u8>) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: Config> Vk<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        match self.0.len() < T::max_vk_size() as usize {
            true => Ok(()),
            false => Err(VerifyError::InvalidVerificationKey),
        }
    }
}

impl<T: Config> MaxEncodedLen for Vk<T> {
    fn max_encoded_len() -> usize {
        T::max_vk_size() as usize
    }
}

pub trait Config: 'static {
    /// Maximum number of bytes contained in the proof (otherwise rejected)
    type MaxProofSize: Get<u32>;
    /// Maximum number of bytes contained in the public inputs (otherwise rejected)
    type MaxPubsSize: Get<u32>;
    /// Maximum number of bytes contained in the verification key (otherwise rejected)
    type MaxVkSize: Get<u32>;

    fn max_proof_size() -> u32 {
        Self::MaxProofSize::get()
    }

    fn max_pubs_size() -> u32 {
        Self::MaxPubsSize::get()
    }

    fn max_vk_size() -> u32 {
        Self::MaxVkSize::get()
    }
}

#[pallet_verifiers::verifier]
pub struct Plonky2<T>;

impl<T: Config> Verifier for Plonky2<T> {
    type Proof = Proof;

    type Pubs = Pubs;

    type Vk = Vk<T>;

    fn hash_context_data() -> &'static [u8] {
        b"plonky2"
    }

    fn verify_proof(
        vk: &Self::Vk,
        raw_proof: &Self::Proof,
        raw_pubs: &Self::Pubs,
    ) -> Result<(), VerifyError> {
        ensure!(
            raw_proof.len() <= T::MaxProofSize::get() as usize,
            hp_verifiers::VerifyError::InvalidProofData
        );
        ensure!(
            raw_pubs.len() <= T::MaxPubsSize::get() as usize,
            hp_verifiers::VerifyError::InvalidInput
        );
        vk.validate_size()?;
        verify_default_poseidon(&vk.0, raw_proof, raw_pubs)
            .map_err(|e| log::debug!("Proof verification failed: {:?}", e))
            .map_err(|_| VerifyError::VerifyError)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        vk.validate_size()?;
        validate_vk_default_poseidon(&vk.0)
            .map_err(|e| log::debug!("VK validation failed: {:?}", e))
            .map_err(|_| VerifyError::InvalidVerificationKey)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]> {
        Cow::Borrowed(pubs)
    }
}

pub struct Plonky2Weight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Plonky2<T>>
    for Plonky2Weight<W>
{
    fn submit_proof(
        _proof: &<Plonky2<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Plonky2<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::submit_proof()
    }

    fn submit_proof_with_vk_hash(
        _proof: &<Plonky2<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Plonky2<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::submit_proof_with_vk_hash()
    }

    fn register_vk(_vk: &<Plonky2<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }
    fn unregister_vk() -> Weight {
        W::unregister_vk()
    }
}
