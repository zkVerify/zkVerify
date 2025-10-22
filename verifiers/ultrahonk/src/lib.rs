// Copyright 2025, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{borrow::Cow, boxed::Box, vec::Vec};
use codec::Decode;
use codec::Encode;
use codec::MaxEncodedLen;
use core::marker::PhantomData;
use frame_support::{ensure, weights::Weight};
use hp_verifiers::{Verifier, VerifyError};
use scale_info::TypeInfo;
use sp_core::{Get, H256};
use ultrahonk_no_std::ProofType as UltraHonkProofType;

use native::bn254::HostHooks as CurveHooksImpl;

use ultrahonk_no_std::key::VerificationKey;
pub use ultrahonk_no_std::PLAIN_PROOF_SIZE;
pub use ultrahonk_no_std::PUB_SIZE;
pub use ultrahonk_no_std::VK_SIZE;
pub use ultrahonk_no_std::ZK_PROOF_SIZE;
pub type RawProof = Vec<u8>;
pub type Pubs = Vec<[u8; PUB_SIZE]>;
pub type Vk = [u8; VK_SIZE];
pub use weight::WeightInfo;

pub trait Config {
    /// Maximum supported number of public inputs.
    type MaxPubs: Get<u32>;
}

#[derive(Copy, Clone, Debug, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ProofType {
    ZK,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub struct RawProofWithType {
    pub proof_type: ProofType,
    pub proof_bytes: RawProof,
}

impl Default for RawProofWithType {
    fn default() -> Self {
        Self {
            proof_type: ProofType::Plain, // mirror Noir's default
            proof_bytes: Vec::new(),
        }
    }
}

impl RawProofWithType {
    pub fn new(proof_type: ProofType, proof_bytes: RawProof) -> Self {
        Self {
            proof_type,
            proof_bytes,
        }
    }
}

impl From<RawProofWithType> for RawProof {
    fn from(proof_with_type: RawProofWithType) -> Self {
        proof_with_type.proof_bytes
    }
}

impl ProofType {
    fn proof_size(self) -> usize {
        match self {
            ProofType::ZK => ZK_PROOF_SIZE,
            ProofType::Plain => PLAIN_PROOF_SIZE,
        }
    }
}

impl From<&RawProofWithType> for UltraHonkProofType {
    fn from(proof: &RawProofWithType) -> Self {
        match proof.proof_type {
            ProofType::ZK => {
                let mut proof_bytes = [0u8; ZK_PROOF_SIZE];
                proof_bytes.copy_from_slice(&proof.proof_bytes);
                UltraHonkProofType::ZK(Box::new(proof_bytes))
            }
            ProofType::Plain => {
                let mut proof_bytes = [0u8; PLAIN_PROOF_SIZE];
                proof_bytes.copy_from_slice(&proof.proof_bytes);
                UltraHonkProofType::Plain(Box::new(proof_bytes))
            }
        }
    }
}

pub mod benchmarking;
mod verifier_should;
pub mod weight;

#[pallet_verifiers::verifier]
pub struct Ultrahonk<T>;

impl<T: Config> Verifier for Ultrahonk<T> {
    type Proof = RawProofWithType;

    type Pubs = Pubs;

    type Vk = Vk;

    fn hash_context_data() -> &'static [u8] {
        b"ultrahonk"
    }

    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        // Transform input proof into an UltraHonk verifier-compatible proof
        ensure!(
            proof.proof_bytes.len() == proof.proof_type.proof_size(),
            VerifyError::InvalidProofData
        );
        ensure!(
            pubs.len() <= T::MaxPubs::get() as usize,
            hp_verifiers::VerifyError::InvalidInput
        );
        let prepared_proof: UltraHonkProofType = proof.into();

        log::trace!("Verifying (no-std)");
        ultrahonk_no_std::verify::<CurveHooksImpl>(vk, &prepared_proof, pubs)
            .inspect_err(|e| log::debug!("Cannot verify proof: {e:?}"))
            .map_err(|e| match e {
                ultrahonk_no_std::errors::VerifyError::VerificationError { message: _ } => {
                    hp_verifiers::VerifyError::VerifyError
                }
                ultrahonk_no_std::errors::VerifyError::PublicInputError { message: _ } => {
                    hp_verifiers::VerifyError::InvalidInput
                }
                ultrahonk_no_std::errors::VerifyError::KeyError => {
                    hp_verifiers::VerifyError::InvalidVerificationKey
                }
                ultrahonk_no_std::errors::VerifyError::InvalidProofError => {
                    hp_verifiers::VerifyError::InvalidProofData
                }
                ultrahonk_no_std::errors::VerifyError::OtherError => {
                    hp_verifiers::VerifyError::VerifyError
                }
            })
            .map(|_| None)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        let _vk = VerificationKey::<CurveHooksImpl>::try_from(&vk[..])
            .map_err(|e| log::debug!("Invalid Vk: {e:?}"))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;

        Ok(())
    }

    fn vk_hash(vk: &Self::Vk) -> H256 {
        sp_io::hashing::sha2_256(&Self::vk_bytes(vk)).into()
    }

    fn vk_bytes(vk: &Self::Vk) -> Cow<'_, [u8]> {
        Self::encode_vk(vk)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        let data = pubs
            .iter()
            .flat_map(|s| s.iter().cloned())
            .collect::<Vec<_>>();
        Cow::Owned(data)
    }
}

impl<T: Config> Ultrahonk<T> {
    // Utility function for future-proofing.
    fn encode_vk(vk: &Vk) -> Cow<'_, [u8]> {
        Cow::Owned(vk.to_vec())
    }
}

/// The struct to use in runtime pallet configuration to map the weight computed by this crate
/// benchmarks to the weight needed by the `pallet-verifiers`.
pub struct UltrahonkWeight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Ultrahonk<T>>
    for UltrahonkWeight<W>
{
    fn verify_proof(
        proof: &<Ultrahonk<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Ultrahonk<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        match proof.proof_type {
            ProofType::ZK => W::verify_proof_zk_32(),
            ProofType::Plain => W::verify_proof_plain_32(),
        }
    }

    fn register_vk(_vk: &<Ultrahonk<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> frame_support::weights::Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Ultrahonk<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Ultrahonk<T> as Verifier>::Proof,
        _pubs: &<Ultrahonk<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
