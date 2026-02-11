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

// #![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{borrow::Cow, vec::Vec};
use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::{ensure, weights::Weight};
use hp_verifiers::{Verifier, VerifyError};
use native::bn254::HostHooks as CurveHooksImpl;
use scale_info::TypeInfo;
use sp_core::{Get, H256};
use ultrahonk_no_std::ProofType as UltraHonkProofType;

pub use crate::weight_verify_proof::WeightInfo as WeightInfoVerifyProof;
use ultrahonk_no_std::key::VerificationKey;
pub use ultrahonk_no_std::{PUB_SIZE, VK_SIZE};
pub use weight::WeightInfo;

pub type RawProof = Vec<u8>;
pub type Pubs = Vec<[u8; PUB_SIZE]>;
pub type Vk = [u8; VK_SIZE];

// Maximum allowed value for the logarithm of the polynomial evaluation domain size.
const MAX_BENCHMARKED_LOG_CIRCUIT_SIZE: u64 = 25;

pub trait Config {
    /// Maximum supported number of public inputs.
    type MaxPubs: Get<u32>;
    /// Weight info used to compute the verify proof weight
    type WeightInfo: WeightInfoVerifyProof;
}

#[derive(Copy, Clone, Debug, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ProofType {
    ZK,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub enum Proof {
    ZK(RawProof),
    Plain(RawProof),
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub enum VersionedProof {
    V3_0(Proof),
}

impl Proof {
    pub fn new(proof_type: ProofType, proof_bytes: RawProof) -> Self {
        match proof_type {
            ProofType::ZK => Self::ZK(proof_bytes),
            ProofType::Plain => Self::Plain(proof_bytes),
        }
    }
}

impl Default for Proof {
    fn default() -> Self {
        Self::ZK(Vec::new()) // mirrors Noir's default
    }
}

impl From<&Proof> for ProofType {
    fn from(proof: &Proof) -> Self {
        match proof {
            Proof::ZK(_) => Self::ZK,
            Proof::Plain(_) => Self::Plain,
        }
    }
}

impl From<&UltraHonkProofType> for ProofType {
    fn from(proof: &UltraHonkProofType) -> Self {
        match proof {
            UltraHonkProofType::ZK(_) => Self::ZK,
            UltraHonkProofType::Plain(_) => Self::Plain,
        }
    }
}

impl From<Proof> for RawProof {
    fn from(proof: Proof) -> Self {
        match proof {
            Proof::ZK(proof_bytes) | Proof::Plain(proof_bytes) => proof_bytes,
        }
    }
}

impl From<&VersionedProof> for UltraHonkProofType {
    fn from(proof: &VersionedProof) -> Self {
        match proof {
            VersionedProof::V3_0(Proof::ZK(proof_bytes)) => {
                UltraHonkProofType::ZK(proof_bytes.clone().into_boxed_slice())
            }
            VersionedProof::V3_0(Proof::Plain(proof_bytes)) => {
                UltraHonkProofType::Plain(proof_bytes.clone().into_boxed_slice())
            }
        }
    }
}

pub mod benchmarking;
pub mod benchmarking_verify_proof;
mod resources;
mod verifier_should;
mod weight;
mod weight_verify_proof;

#[pallet_verifiers::verifier]
pub struct Ultrahonk<T>;

impl<T: Config> Verifier for Ultrahonk<T> {
    type Proof = VersionedProof;

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
        ensure!(
            pubs.len() <= T::MaxPubs::get() as usize,
            hp_verifiers::VerifyError::InvalidInput
        );

        // Transform input proof into an UltraHonk verifier-compatible proof
        let prepared_proof: UltraHonkProofType = proof.into();

        let w = {
            let log_circuit_size = VerificationKey::<CurveHooksImpl>::extract_log_circuit_size(vk)
                .map_err(|_| hp_verifiers::VerifyError::InvalidVerificationKey)?;
            ensure!(
                log_circuit_size <= MAX_BENCHMARKED_LOG_CIRCUIT_SIZE,
                hp_verifiers::VerifyError::InvalidVerificationKey
            );

            compute_weight::<T>(log_circuit_size, ProofType::from(&prepared_proof))
        };

        log::trace!("Verifying (no-std)");
        ultrahonk_no_std::verify::<CurveHooksImpl>(vk, &prepared_proof, pubs)
            .inspect_err(|e| log::debug!("Cannot verify proof: {e:?}"))
            .map_err(|e| match e {
                ultrahonk_no_std::errors::VerifyError::VerificationError { message: _ } => {
                    println!("HERE!");
                    hp_verifiers::VerifyError::VerifyError
                }
                ultrahonk_no_std::errors::VerifyError::PublicInputError { message: _ } => {
                    hp_verifiers::VerifyError::InvalidInput
                }
                ultrahonk_no_std::errors::VerifyError::KeyError => {
                    hp_verifiers::VerifyError::InvalidVerificationKey
                }
                ultrahonk_no_std::errors::VerifyError::InvalidProofError { message: _ } => {
                    hp_verifiers::VerifyError::InvalidProofData
                }
                ultrahonk_no_std::errors::VerifyError::OtherError => {
                    hp_verifiers::VerifyError::VerifyError
                }
            })
            // .map(|_| None)
            .map(|_| Some(w))
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

fn compute_weight<T: Config>(log_circuit_size: u64, proof_type: ProofType) -> Weight {
    // Note that for very small circuits (i.e., log_circuit_size < MIN_BENCHMARKED_LOG_CIRCUIT_SIZE),
    // we compute weights using log_circuit_size = MIN_BENCHMARKED_LOG_CIRCUIT_SIZE
    match (log_circuit_size, proof_type) {
        (1, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_7(),
        (1, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_7(),
        (2, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_7(),
        (2, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_7(),
        (3, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_7(),
        (3, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_7(),
        (4, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_7(),
        (4, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_7(),
        (5, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_7(),
        (5, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_7(),
        (6, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_7(),
        (6, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_7(),
        (7, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_7(),
        (7, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_7(),
        (8, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_8(),
        (8, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_8(),
        (9, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_9(),
        (9, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_9(),
        (10, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_10(),
        (10, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_10(),
        (11, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_11(),
        (11, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_11(),
        (12, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_12(),
        (12, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_12(),
        (13, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_13(),
        (13, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_13(),
        (14, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_14(),
        (14, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_14(),
        (15, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_15(),
        (15, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_15(),
        (16, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_16(),
        (16, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_16(),
        (17, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_17(),
        (17, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_17(),
        (18, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_18(),
        (18, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_18(),
        (19, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_19(),
        (19, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_19(),
        (20, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_20(),
        (20, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_20(),
        (21, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_21(),
        (21, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_21(),
        (22, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_22(),
        (22, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_22(),
        (23, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_23(),
        (23, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_23(),
        (24, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_24(),
        (24, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_24(),
        (25, ProofType::ZK) => T::WeightInfo::verify_zk_proof_log_25(),
        (25, ProofType::Plain) => T::WeightInfo::verify_plain_proof_log_25(),
        _ => panic!("Invalid value given for log_circuit_size."),
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
pub struct UltrahonkWeight<W: WeightInfo>(PhantomData<W>);

impl<T: Config, W: WeightInfo> pallet_verifiers::WeightInfo<Ultrahonk<T>> for UltrahonkWeight<W> {
    fn verify_proof(
        proof: &<Ultrahonk<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Ultrahonk<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        let prepared_proof: UltraHonkProofType = proof.into();
        let proof_type = ProofType::from(&prepared_proof);
        match proof_type {
            ProofType::ZK => T::WeightInfo::verify_zk_proof_log_25(),
            ProofType::Plain => T::WeightInfo::verify_plain_proof_log_25(),
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
