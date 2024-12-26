#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub use crate::vk::{Plonky2SystemConfig, VerificationKeyWithSystemConfig};
use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::__private::Get;
use frame_support::ensure;
use frame_support::traits::ConstU32;
use frame_support::weights::Weight;
use hp_verifiers::{Cow, Verifier, VerifyError};
use plonky2_verifier::validate::{validate_vk_default_keccak, validate_vk_default_poseidon};
use plonky2_verifier::{verify_default_keccak, verify_default_poseidon};

pub mod benchmarking;
mod resources;
pub(crate) mod verifier_should;
mod vk;
mod weight;

pub use weight::WeightInfo;

pub type Pubs = Vec<u8>;
pub type Proof = Vec<u8>;
pub type Vk<T> = VerificationKeyWithSystemConfig<T>;

impl<T: Config> Vk<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        match self.vk_serialized.len() < T::max_vk_size() as usize {
            true => Ok(()),
            false => Err(VerifyError::InvalidVerificationKey),
        }
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
        match vk.system_config {
            Plonky2SystemConfig::Keccak => {
                verify_default_keccak(&vk.vk_serialized, raw_proof, raw_pubs)
            }
            Plonky2SystemConfig::Poseidon => {
                verify_default_poseidon(&vk.vk_serialized, raw_proof, raw_pubs)
            }
        }
        .map_err(|e| log::debug!("Proof verification failed: {:?}", e))
        .map_err(|_| VerifyError::VerifyError)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        vk.validate_size()?;
        match vk.system_config {
            Plonky2SystemConfig::Keccak => validate_vk_default_keccak(&vk.vk_serialized),
            Plonky2SystemConfig::Poseidon => validate_vk_default_poseidon(&vk.vk_serialized),
        }
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
