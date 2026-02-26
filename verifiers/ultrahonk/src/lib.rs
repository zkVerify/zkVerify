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

use alloc::{borrow::Cow, vec::Vec};
use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::{ensure, weights::Weight};
use hp_verifiers::{Verifier, VerifyError};
use native::bn254::HostHooks as CurveHooksImpl;
use scale_info::TypeInfo;
use sp_core::{Get, H256};

pub use crate::weight_verify_proof::WeightInfo as WeightInfoVerifyProof;
pub use ultrahonk_no_std_v3_0::PUB_SIZE; // Can be obtained from an arbitrary version
pub use weight::WeightInfo;

pub type RawProof = Vec<u8>;
pub type Pubs = Vec<[u8; PUB_SIZE]>;

// Minimum allowed value for the logarithm of the polynomial evaluation domain size.
pub const MIN_BENCHMARKED_LOG_CIRCUIT_SIZE: u64 = 7;
// Maximum allowed value for the logarithm of the polynomial evaluation domain size.
pub const MAX_BENCHMARKED_LOG_CIRCUIT_SIZE: u64 = 25;

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

impl TryFrom<Proof> for ultrahonk_no_std_v3_0::ProofType {
    type Error = ();

    fn try_from(proof: Proof) -> Result<Self, Self::Error> {
        match proof {
            Proof::ZK(proof_bytes) => Ok(Self::ZK(proof_bytes.into_boxed_slice())),
            Proof::Plain(proof_bytes) => Ok(Self::Plain(proof_bytes.into_boxed_slice())),
        }
    }
}

impl TryFrom<Proof> for ultrahonk_no_std_v0_84::ProofType {
    type Error = ();

    fn try_from(proof: Proof) -> Result<Self, Self::Error> {
        match proof {
            Proof::ZK(proof_bytes) => Ok(Self::ZK(proof_bytes.try_into().map_err(|_| ())?)),
            Proof::Plain(proof_bytes) => Ok(Self::Plain(proof_bytes.try_into().map_err(|_| ())?)),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ProtocolVersion {
    V0_84,
    V3_0,
}

impl From<&VersionedProof> for ProtocolVersion {
    fn from(value: &VersionedProof) -> Self {
        match value {
            VersionedProof::V0_84(_) => ProtocolVersion::V0_84,
            VersionedProof::V3_0(_) => ProtocolVersion::V3_0,
        }
    }
}

// Important Notes:
// i) Please DO NOT alter the indices of existing VersionedProof's variants,
// ii) If you are introducing new VersionedProof variants, ensure that
// indices match those in VersionedVk.
#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub enum VersionedProof {
    #[codec(index = 0)]
    V0_84(Proof),
    #[codec(index = 1)]
    V3_0(Proof),
}

// Important Notes:
// i) Please DO NOT alter the indices of existing VersionedVk's variants,
// ii) If you are introducing new VersionedVk variants, ensure that
// indices match those in VersionedProof.
#[derive(Clone, Debug, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum VersionedVk {
    #[codec(index = 0)]
    V0_84([u8; ultrahonk_no_std_v0_84::VK_SIZE]),
    #[codec(index = 1)]
    V3_0([u8; ultrahonk_no_std_v3_0::VK_SIZE]),
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

impl Default for VersionedProof {
    fn default() -> Self {
        Self::V3_0(Proof::default())
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

impl From<&ultrahonk_no_std_v0_84::ProofType> for ProofType {
    fn from(proof: &ultrahonk_no_std_v0_84::ProofType) -> Self {
        match proof {
            ultrahonk_no_std_v0_84::ProofType::ZK(_) => Self::ZK,
            ultrahonk_no_std_v0_84::ProofType::Plain(_) => Self::Plain,
        }
    }
}

impl From<&ultrahonk_no_std_v3_0::ProofType> for ProofType {
    fn from(proof: &ultrahonk_no_std_v3_0::ProofType) -> Self {
        match proof {
            ultrahonk_no_std_v3_0::ProofType::ZK(_) => Self::ZK,
            ultrahonk_no_std_v3_0::ProofType::Plain(_) => Self::Plain,
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

    type Vk = VersionedVk;

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

        match (proof, vk) {
            (VersionedProof::V0_84(inner_proof), VersionedVk::V0_84(vk_bytes)) => {
                // Transform input proof into an UltraHonk verifier-compatible proof
                let prepared: ultrahonk_no_std_v0_84::ProofType = inner_proof
                    .clone()
                    .try_into()
                    .map_err(|_| hp_verifiers::VerifyError::InvalidProofData)?;

                log::trace!("Verifying (no-std)");
                ultrahonk_no_std_v0_84::verify::<CurveHooksImpl>(vk_bytes, &prepared, pubs)
                    .inspect_err(|e| log::debug!("Cannot verify proof: {e:?}"))
                    .map_err(|e| match e {
                        ultrahonk_no_std_v0_84::errors::VerifyError::VerificationError {
                            message: _,
                        } => hp_verifiers::VerifyError::VerifyError,
                        ultrahonk_no_std_v0_84::errors::VerifyError::PublicInputError {
                            message: _,
                        } => hp_verifiers::VerifyError::InvalidInput,
                        ultrahonk_no_std_v0_84::errors::VerifyError::KeyError => {
                            hp_verifiers::VerifyError::InvalidVerificationKey
                        }
                        ultrahonk_no_std_v0_84::errors::VerifyError::InvalidProofError {} => {
                            hp_verifiers::VerifyError::InvalidProofData
                        }
                        ultrahonk_no_std_v0_84::errors::VerifyError::OtherError => {
                            hp_verifiers::VerifyError::VerifyError
                        }
                    })
                    .map(|_| None)
            }
            (VersionedProof::V3_0(inner_proof), VersionedVk::V3_0(vk_bytes)) => {
                // Transform input proof into an UltraHonk verifier-compatible proof
                let prepared: ultrahonk_no_std_v3_0::ProofType = inner_proof
                    .clone()
                    .try_into()
                    .map_err(|_| hp_verifiers::VerifyError::InvalidProofData)?;

                let w = {
                    let log_circuit_size = ultrahonk_no_std_v3_0::key::VerificationKey::<
                        CurveHooksImpl,
                    >::extract_log_circuit_size(vk_bytes)
                    .map_err(|_| hp_verifiers::VerifyError::InvalidVerificationKey)?;
                    ensure!(
                        log_circuit_size <= MAX_BENCHMARKED_LOG_CIRCUIT_SIZE,
                        hp_verifiers::VerifyError::InvalidVerificationKey
                    );

                    compute_weight::<T>(
                        ProtocolVersion::from(proof),
                        ProofType::from(&prepared),
                        log_circuit_size,
                    )
                };

                log::trace!("Verifying (no-std)");
                ultrahonk_no_std_v3_0::verify::<CurveHooksImpl>(vk_bytes, &prepared, pubs)
                    .inspect_err(|e| log::debug!("Cannot verify proof: {e:?}"))
                    .map_err(|e| match e {
                        ultrahonk_no_std_v3_0::errors::VerifyError::VerificationError {
                            message: _,
                        } => hp_verifiers::VerifyError::VerifyError,
                        ultrahonk_no_std_v3_0::errors::VerifyError::PublicInputError {
                            message: _,
                        } => hp_verifiers::VerifyError::InvalidInput,
                        ultrahonk_no_std_v3_0::errors::VerifyError::KeyError => {
                            hp_verifiers::VerifyError::InvalidVerificationKey
                        }
                        ultrahonk_no_std_v3_0::errors::VerifyError::InvalidProofError {
                            message: _,
                        } => hp_verifiers::VerifyError::InvalidProofData,
                        ultrahonk_no_std_v3_0::errors::VerifyError::OtherError => {
                            hp_verifiers::VerifyError::VerifyError
                        }
                    })
                    .map(|_| Some(w))
            }
            _ => {
                log::debug!("Proof version does not match Vk version!");
                return Err(hp_verifiers::VerifyError::VerifyError);
            }
        }
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        let vk_bytes: &[u8] = match vk {
            VersionedVk::V0_84(vk_bytes) => vk_bytes,
            VersionedVk::V3_0(vk_bytes) => vk_bytes,
        };
        match vk {
            VersionedVk::V0_84(_) => {
                let _vk = ultrahonk_no_std_v0_84::key::VerificationKey::<CurveHooksImpl>::try_from(
                    &vk_bytes[..],
                )
                .map_err(|e| log::debug!("Invalid Vk: {e:?}"))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
            }
            VersionedVk::V3_0(_) => {
                let _vk = ultrahonk_no_std_v3_0::key::VerificationKey::<CurveHooksImpl>::try_from(
                    &vk_bytes[..],
                )
                .map_err(|e| log::debug!("Invalid Vk: {e:?}"))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
            }
        }

        Ok(())
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        let data = pubs
            .iter()
            .flat_map(|s| s.iter().cloned())
            .collect::<Vec<_>>();
        Cow::Owned(data)
    }

    fn verifier_version_hash(proof: &Self::Proof) -> H256 {
        // Computed as: SHA2-256("ultrahonk:vx.y")
        let h = match proof {
            VersionedProof::V0_84(_) => hex_literal::hex!(
                "4966cd7801ae9ef9d7afb52ec3de92f0693e720f58c5c8ecfb23d85b0934f018"
            ),
            VersionedProof::V3_0(_) => hex_literal::hex!(
                "55b52ad2b4153c872e27d688f567c1406f0d93b5528dd2b0bf2a9a40df97f1f9"
            ),
        };
        H256(h)
    }
}

fn compute_weight<T: Config>(
    protocol_version: ProtocolVersion,
    proof_type: ProofType,
    log_circuit_size: u64,
) -> Weight {
    // Note that for very small circuits (i.e., log_circuit_size < MIN_BENCHMARKED_LOG_CIRCUIT_SIZE),
    // we compute weights using log_circuit_size = MIN_BENCHMARKED_LOG_CIRCUIT_SIZE
    match (
        protocol_version,
        proof_type,
        log_circuit_size.max(MIN_BENCHMARKED_LOG_CIRCUIT_SIZE),
    ) {
        (ProtocolVersion::V3_0, ProofType::ZK, log_n) => T::WeightInfo::verify_zk_proof_v3_0(log_n),
        (ProtocolVersion::V3_0, ProofType::Plain, log_n) => {
            T::WeightInfo::verify_plain_proof_v3_0(log_n)
        }
        _ => panic!("Invalid value given for log_circuit_size."),
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
        match proof {
            VersionedProof::V0_84(inner) => match ProofType::from(inner) {
                ProofType::ZK => T::WeightInfo::verify_zk_proof_v0_84(),
                ProofType::Plain => T::WeightInfo::verify_plain_proof_v0_84(),
            },
            VersionedProof::V3_0(inner) => {
                // V3.0: weight is parameterized by log_circuit_size (worst case = 25)
                // Without access to the vk here, we conservatively charge the maximum.
                match inner {
                    Proof::ZK(_) => {
                        T::WeightInfo::verify_zk_proof_v3_0(MAX_BENCHMARKED_LOG_CIRCUIT_SIZE)
                    }
                    Proof::Plain(_) => {
                        T::WeightInfo::verify_plain_proof_v3_0(MAX_BENCHMARKED_LOG_CIRCUIT_SIZE)
                    }
                }
            }
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
