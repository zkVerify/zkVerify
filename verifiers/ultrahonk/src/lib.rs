// Copyright 2025-2026, Horizen Labs, Inc.

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
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::traits::StorageVersion;
use frame_support::{ensure, weights::Weight};
use native::bn254::HostHooks as CurveHooksImpl;
use pallet_verifiers::traits::{Verifier, VerifyError};
use scale_info::TypeInfo;
use sp_core::{Get, H256};

pub use crate::weight_verify_proof::WeightInfo as WeightInfoVerifyProof;
pub use ultrahonk_no_std_v5_0::PUB_SIZE;
pub use ultrahonk_no_std_v5_0::VK_SIZE as VK_SIZE_V5_0;
pub use weight::WeightInfo;

pub mod benchmarking;
pub mod benchmarking_verify_proof;
pub mod migrations;
mod resources;
mod verifier_should;
mod weight;
mod weight_verify_proof;

/// VK size of the deprecated `V0_84` and `Legacy` versions (`ultrahonk-no-std` v0.2.1). Their
/// verifier is no longer linked, but VKs already registered with it must stay decodable so that
/// their owners can still unregister them.
pub const VK_SIZE_V0_84: usize = 1760;
/// VK size of the deprecated `V3_0` version (`ultrahonk-no-std` v0.3.2). See [`VK_SIZE_V0_84`].
pub const VK_SIZE_V3_0: usize = 1888;

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

#[derive(
    Copy, Clone, Debug, PartialEq, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
)]
pub enum ProofType {
    ZK,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub enum Proof {
    ZK(RawProof),
    Plain(RawProof),
}

impl TryFrom<Proof> for ultrahonk_no_std_v5_0::ProofType {
    type Error = VerifyError;

    fn try_from(proof: Proof) -> Result<Self, Self::Error> {
        Ok(match proof {
            Proof::ZK(proof_bytes) => Self::ZK(proof_bytes.into_boxed_slice()),
            Proof::Plain(proof_bytes) => Self::Plain(proof_bytes.into_boxed_slice()),
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ProtocolVersion {
    V0_84,
    V3_0,
    Legacy,
    V5_0,
}

impl From<&VersionedProof> for ProtocolVersion {
    fn from(value: &VersionedProof) -> Self {
        match value {
            VersionedProof::V0_84(_) => ProtocolVersion::V0_84,
            VersionedProof::V3_0(_) => ProtocolVersion::V3_0,
            VersionedProof::Legacy(_) => ProtocolVersion::Legacy,
            VersionedProof::V5_0(_) => ProtocolVersion::V5_0,
        }
    }
}

// Important Notes:
// i) Please DO NOT alter the indices of existing VersionedProof's variants,
// ii) If you are introducing new VersionedProof variants, ensure that
// indices match those in VersionedVk.
//
// `V0_84`, `V3_0` and `Legacy` are deprecated: they carry the pre-fix Barretenberg
// SmallSubgroupIPA (sound only from bb v5.0.0) and accept non-canonical public inputs. They are
// kept so that indices and stored VKs stay valid, but every proof and VK for them is rejected
// with `VerifyError::UnsupportedVersion`.
#[derive(Clone, Debug, PartialEq, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub enum VersionedProof {
    #[codec(index = 0)]
    V0_84(Proof),
    #[codec(index = 1)]
    V3_0(Proof),
    #[codec(index = 2)]
    Legacy(Proof),
    #[codec(index = 3)]
    V5_0(Proof),
}

// Important Notes:
// i) Please DO NOT alter the indices of existing VersionedVk's variants,
// ii) If you are introducing new VersionedVk variants, ensure that
// indices match those in VersionedProof.
#[derive(
    Clone, Debug, PartialEq, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
)]
pub enum VersionedVk {
    #[codec(index = 0)]
    V0_84([u8; VK_SIZE_V0_84]),
    #[codec(index = 1)]
    V3_0([u8; VK_SIZE_V3_0]),
    #[codec(index = 2)]
    Legacy([u8; VK_SIZE_V0_84]),
    #[codec(index = 3)]
    V5_0([u8; VK_SIZE_V5_0]),
}

impl Proof {
    pub fn new(proof_type: ProofType, proof_bytes: RawProof) -> Self {
        match proof_type {
            ProofType::ZK => Self::ZK(proof_bytes),
            ProofType::Plain => Self::Plain(proof_bytes),
        }
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

impl From<&ultrahonk_no_std_v5_0::ProofType> for ProofType {
    fn from(proof: &ultrahonk_no_std_v5_0::ProofType) -> Self {
        match proof {
            ultrahonk_no_std_v5_0::ProofType::ZK(_) => Self::ZK,
            ultrahonk_no_std_v5_0::ProofType::Plain(_) => Self::Plain,
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

trait IntoVerifyError {
    fn into_verify_error(self) -> VerifyError;
}

impl IntoVerifyError for ultrahonk_no_std_v5_0::errors::VerifyError {
    fn into_verify_error(self) -> VerifyError {
        match self {
            ultrahonk_no_std_v5_0::errors::VerifyError::VerificationError { message: _ } => {
                VerifyError::VerifyError
            }
            ultrahonk_no_std_v5_0::errors::VerifyError::PublicInputError { message: _ } => {
                VerifyError::InvalidInput
            }
            ultrahonk_no_std_v5_0::errors::VerifyError::KeyError => {
                VerifyError::InvalidVerificationKey
            }
            ultrahonk_no_std_v5_0::errors::VerifyError::InvalidProofError { message: _ } => {
                VerifyError::InvalidProofData
            }
            ultrahonk_no_std_v5_0::errors::VerifyError::OtherError => VerifyError::VerifyError,
        }
    }
}

#[pallet_verifiers::verifier]
pub struct Ultrahonk<T>;

impl<T: Config> Verifier for Ultrahonk<T> {
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

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
            VerifyError::InvalidInput
        );

        match (proof, vk) {
            (VersionedProof::V5_0(inner_proof), VersionedVk::V5_0(vk_bytes)) => {
                // Transform input proof into an UltraHonk verifier-compatible proof
                let prepared: ultrahonk_no_std_v5_0::ProofType = inner_proof.clone().try_into()?;
                let log_circuit_size = valid_log_circuit_size(vk_bytes)?;

                log::trace!("Verifying (no-std)");
                ultrahonk_no_std_v5_0::verify::<CurveHooksImpl>(vk_bytes, &prepared, pubs)
                    .inspect_err(|e| log::debug!("Cannot verify proof: {e:?}"))
                    .map_err(IntoVerifyError::into_verify_error)
                    .map(|_| {
                        compute_weight::<T>(proof.into(), (&prepared).into(), log_circuit_size)
                            .into()
                    })
            }
            // V5_0 is the only supported version: any other combination has a deprecated proof
            // or vk (e.g. a V5_0 proof against a V3_0 vk that was never re-registered).
            _ => {
                log::debug!("Deprecated proof or vk version");
                Err(VerifyError::UnsupportedVersion)
            }
        }
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        match vk {
            VersionedVk::V5_0(vk_bytes) => {
                let _vk = ultrahonk_no_std_v5_0::key::VerificationKey::<CurveHooksImpl>::try_from(
                    &vk_bytes[..],
                )
                .map_err(|e| log::debug!("Invalid Vk: {e:?}"))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
                Ok(())
            }
            VersionedVk::V0_84(_) | VersionedVk::V3_0(_) | VersionedVk::Legacy(_) => {
                log::debug!("Deprecated vk version");
                Err(VerifyError::UnsupportedVersion)
            }
        }
    }

    fn vk_hash(vk: &Self::Vk) -> H256 {
        match vk {
            // Legacy uses SHA2-256 of raw VK bytes to match the pre-versioning hash.
            // See commit 113a728c, verifiers/ultrahonk/src/lib.rs:183-184
            VersionedVk::Legacy(_) => sp_io::hashing::sha2_256(&Self::vk_bytes(vk)).into(),
            _ => sp_io::hashing::keccak_256(&Self::vk_bytes(vk)).into(),
        }
    }

    fn vk_bytes(vk: &Self::Vk) -> Cow<'_, [u8]> {
        match vk {
            // Legacy returns raw bytes (no enum prefix) to match the pre-versioning encoding.
            // See commit 113a728c, verifiers/ultrahonk/src/lib.rs:187-188,202-204
            VersionedVk::Legacy(bytes) => Cow::Owned(bytes.to_vec()),
            _ => Cow::Owned(vk.encode()),
        }
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
        match proof {
            VersionedProof::V0_84(_) => H256(hex_literal::hex!(
                "4966cd7801ae9ef9d7afb52ec3de92f0693e720f58c5c8ecfb23d85b0934f018"
            )),
            VersionedProof::V3_0(_) => H256(hex_literal::hex!(
                "55b52ad2b4153c872e27d688f567c1406f0d93b5528dd2b0bf2a9a40df97f1f9"
            )),
            // Legacy returns NO_VERSION_HASH to preserve backward compatibility with
            // the pre-versioning statement hash (before commit 83e40f29).
            VersionedProof::Legacy(_) => pallet_verifiers::traits::NO_VERSION_HASH,
            VersionedProof::V5_0(_) => H256(hex_literal::hex!(
                "91ea9b035e570dfe3d6b0dff19fb13586dfdd5a9da15c14f49f7ce6b3b2bc589"
            )),
        }
    }
}

fn valid_log_circuit_size(vk_bytes: &[u8; VK_SIZE_V5_0]) -> Result<u64, VerifyError> {
    let log_circuit_size =
        ultrahonk_no_std_v5_0::key::VerificationKey::<CurveHooksImpl>::extract_log_circuit_size(
            vk_bytes,
        )
        .map_err(|_| VerifyError::InvalidVerificationKey)?;
    ensure!(
        log_circuit_size <= MAX_BENCHMARKED_LOG_CIRCUIT_SIZE,
        VerifyError::InvalidVerificationKey
    );
    Ok(log_circuit_size)
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
        (ProtocolVersion::V5_0, ProofType::ZK, log_n) => {
            T::WeightInfo::verify_zk_proof_v5_0(log_n as u32)
        }
        (ProtocolVersion::V5_0, ProofType::Plain, log_n) => {
            T::WeightInfo::verify_plain_proof_v5_0(log_n as u32)
        }
        _ => panic!("Invalid value given for log_circuit_size."),
    }
}

/// The struct to use in runtime pallet configuration to map the weight computed by this crate
/// benchmarks to the weight needed by the `pallet-verifiers`.
pub struct UltrahonkWeight<W: WeightInfo>(PhantomData<W>);

impl<T: Config, W: WeightInfo> pallet_verifiers::WeightInfo<Ultrahonk<T>> for UltrahonkWeight<W> {
    fn verify_proof(
        proof: &<Ultrahonk<T> as Verifier>::Proof,
        _pubs: &<Ultrahonk<T> as Verifier>::Pubs,
    ) -> Weight {
        // The weight is parameterized by log_circuit_size: we conservatively charge the maximum
        // (worst case = 25) and refund after verification. Deprecated versions are rejected
        // before any verification, but are charged the same so that they are no cheaper to spam.
        let inner = match proof {
            VersionedProof::V5_0(inner)
            | VersionedProof::V0_84(inner)
            | VersionedProof::V3_0(inner)
            | VersionedProof::Legacy(inner) => inner,
        };
        (match inner {
            Proof::ZK(_) => T::WeightInfo::verify_zk_proof_v5_0,
            Proof::Plain(_) => T::WeightInfo::verify_plain_proof_v5_0,
        })(MAX_BENCHMARKED_LOG_CIRCUIT_SIZE as u32)
    }

    fn register_vk(_vk: &<Ultrahonk<T> as Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Ultrahonk<T> as Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Ultrahonk<T> as Verifier>::Proof,
        _pubs: &<Ultrahonk<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
