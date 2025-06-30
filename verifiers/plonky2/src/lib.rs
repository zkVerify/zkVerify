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

extern crate alloc;

pub use crate::proof::Proof as MorphProof;
pub use crate::vk::{Plonky2Config, VkWithConfig};

use alloc::{borrow::Cow, vec::Vec};
use core::marker::PhantomData;
use frame_support::ensure;
use frame_support::pallet_prelude::Get;
use frame_support::weights::Weight;
use hp_verifiers::{Verifier, VerifyError};
use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, KeccakGoldilocksConfig, PoseidonGoldilocksConfig};
use plonky2_verifier::validate::ValidateError;
use plonky2_verifier::{deserialize_vk, verify};

pub mod benchmarking;
pub mod benchmarking_verify_proof;
mod proof;
mod resources;
mod verifier_should;
mod vk;
mod weight;
mod weight_verify_proof;

pub use crate::weight_verify_proof::WeightInfo as WeightInfoVerifyProof;
pub use weight::WeightInfo;

pub type Pubs = Vec<u8>;
pub type Proof<T> = MorphProof<T>;
pub type Vk<T> = VkWithConfig<T>;

const MAX_DEGREE_BITS: usize = 19;

impl<T: Config> Vk<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        if self.bytes.len() > T::max_vk_size() as usize {
            return Err(VerifyError::InvalidVerificationKey);
        }
        Ok(())
    }
}

impl<T: Config> Proof<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        if self.bytes.len() > T::max_proof_size() as usize {
            return Err(VerifyError::InvalidProofData);
        }
        Ok(())
    }
}

pub trait Config: 'static {
    /// Maximum number of bytes contained in the proof (otherwise rejected)
    type MaxProofSize: Get<u32>;
    /// Maximum number of bytes contained in the public inputs (otherwise rejected)
    type MaxPubsSize: Get<u32>;
    /// Maximum number of bytes contained in the verification key (otherwise rejected)
    type MaxVkSize: Get<u32>;
    /// Weight info used to compute the verify proof weight
    type WeightInfo: WeightInfoVerifyProof;

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
    type Proof = Proof<T>;

    type Pubs = Pubs;

    type Vk = Vk<T>;

    fn hash_context_data() -> &'static [u8] {
        b"plonky2"
    }

    fn verify_proof(
        vk: &Self::Vk,
        raw_proof: &Self::Proof,
        raw_pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        vk.validate_size()?;
        raw_proof.validate_size()?;
        ensure!(
            raw_pubs.len() <= T::MaxPubsSize::get() as usize,
            hp_verifiers::VerifyError::InvalidInput
        );

        let vk = plonky2_verifier::Vk::from(vk.clone());
        let proof = plonky2_verifier::Proof::from(raw_proof.clone());

        let degree_bits = match vk.config {
            plonky2_verifier::Plonky2Config::Keccak => {
                const D: usize = 2;
                type C = KeccakGoldilocksConfig;
                type F = <C as GenericConfig<D>>::F;

                deserialize_vk::<F, C, D>(&vk.bytes)
                    .map_err(|_| VerifyError::InvalidVerificationKey)?
                    .common
                    .fri_params
                    .degree_bits
            }
            plonky2_verifier::Plonky2Config::Poseidon => {
                const D: usize = 2;
                type C = PoseidonGoldilocksConfig;
                type F = <C as GenericConfig<D>>::F;

                deserialize_vk::<F, C, D>(&vk.bytes)
                    .map_err(|_| VerifyError::InvalidVerificationKey)?
                    .common
                    .fri_params
                    .degree_bits
            }
        };

        let w = compute_weight::<T>(degree_bits, vk.config);

        verify(&vk, &proof, raw_pubs)
            .inspect_err(|e| log::debug!("Proof verification failed: {:?}", e))
            .map_err(|_| VerifyError::VerifyError)
            .map(|_| Some(w))
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        vk.validate_size()?;

        let vk = plonky2_verifier::Vk::from(vk.clone());

        let vk = match vk.config {
            plonky2_verifier::Plonky2Config::Keccak => {
                const D: usize = 2;
                type C = KeccakGoldilocksConfig;
                type F = <C as GenericConfig<D>>::F;

                validate_vk_inner::<F, C, D>(&vk.bytes)
            }
            plonky2_verifier::Plonky2Config::Poseidon => {
                const D: usize = 2;
                type C = PoseidonGoldilocksConfig;
                type F = <C as GenericConfig<D>>::F;

                validate_vk_inner::<F, C, D>(&vk.bytes)
            }
        };

        vk.inspect_err(|e| log::debug!("VK validation failed: {:?}", e))
            .map_err(|_| VerifyError::InvalidVerificationKey)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        Cow::Borrowed(pubs)
    }
}

fn validate_vk_inner<F, C, const D: usize>(vk: &[u8]) -> plonky2_verifier::ValidateResult
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    deserialize_vk::<F, C, D>(vk)
        .map_err(ValidateError::from)
        .and_then(|vk| {
            (vk.common.fri_params.degree_bits <= MAX_DEGREE_BITS
                && vk.common.config == CircuitConfig::standard_recursion_config())
            .then_some(())
            .ok_or(ValidateError::UnsupportedCircuitConfig)
        })
}

fn compute_weight<T: Config>(
    degree_bits: usize,
    config: plonky2_verifier::Plonky2Config,
) -> Weight {
    match (degree_bits, config) {
        (1, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_2()
        }
        (1, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_2()
        }
        (2, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_2()
        }
        (2, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_2()
        }
        (3, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_3()
        }
        (3, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_3()
        }
        (4, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_4()
        }
        (4, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_4()
        }
        (5, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_5()
        }
        (5, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_5()
        }
        (6, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_6()
        }
        (6, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_6()
        }
        (7, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_7()
        }
        (7, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_7()
        }
        (8, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_8()
        }
        (8, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_8()
        }
        (9, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_9()
        }
        (9, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_9()
        }
        (10, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_10()
        }
        (10, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_10()
        }
        (11, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_11()
        }
        (11, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_11()
        }
        (12, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_12()
        }
        (12, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_12()
        }
        (13, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_13()
        }
        (13, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_13()
        }
        (14, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_14()
        }
        (14, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_14()
        }
        (15, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_15()
        }
        (15, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_15()
        }
        (16, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_16()
        }
        (16, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_16()
        }
        (17, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_17()
        }
        (17, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_17()
        }
        (18, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_18()
        }
        (18, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_18()
        }
        (19, plonky2_verifier::Plonky2Config::Poseidon) => {
            T::WeightInfo::verify_proof_poseidon_uncompressed_19()
        }
        (19, plonky2_verifier::Plonky2Config::Keccak) => {
            T::WeightInfo::verify_proof_keccak_uncompressed_19()
        }
        _ => panic!("Invalid value given for degree_bits."),
    }
}

pub struct Plonky2Weight<W: WeightInfo>(PhantomData<W>);

impl<T: Config, W: WeightInfo> pallet_verifiers::WeightInfo<Plonky2<T>> for Plonky2Weight<W> {
    fn verify_proof(
        _proof: &<Plonky2<T> as Verifier>::Proof,
        _pubs: &<Plonky2<T> as Verifier>::Pubs,
    ) -> Weight {
        // TODO: Update once compression is supported
        T::WeightInfo::verify_proof_poseidon_uncompressed_19()
        // W::verify_proof()
    }

    fn register_vk(_vk: &<Plonky2<T> as Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Plonky2<T> as Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Plonky2<T> as Verifier>::Proof,
        _pubs: &<Plonky2<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
