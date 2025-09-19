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
use scale_info::TypeInfo;
use sp_core::{Get, H256};

use native::bn254::HostHooks as CurveHooksImpl;

pub use ezkl_no_std::PUBS_SIZE;

// Maximum supported VKA length in bytes. Set to a multiple of 32.
pub const MAX_VK_LENGTH: u32 = 3808;
pub struct EzklVkMaxByteLen;
impl frame_support::traits::Get<u32> for EzklVkMaxByteLen {
    fn get() -> u32 {
        MAX_VK_LENGTH
    }
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub struct EzklVk {
    pub vk_bytes: Vec<u8>,
}

impl EzklVk {
    pub fn new(vk_bytes: Vec<u8>) -> Self {
        EzklVk { vk_bytes }
    }
}

pub type Proof = Vec<u8>;
pub type Pubs = Vec<[u8; PUBS_SIZE]>;
pub type Vk = EzklVk;
pub use weight::WeightInfo;

pub trait Config {
    /// Maximum supported number of public inputs.
    type MaxPubs: Get<u32>;
}

impl MaxEncodedLen for EzklVk {
    fn max_encoded_len() -> usize {
        // codec::Compact(len).encoded_size() + element_size * len as usize
        codec::Compact(MAX_VK_LENGTH).encoded_size() + 1 * MAX_VK_LENGTH as usize
    }
}

pub mod benchmarking;
mod verifier_should;
pub mod weight;

#[pallet_verifiers::verifier]
pub struct Ezkl<T>;

impl<T: Config> Verifier for Ezkl<T> {
    type Proof = Proof;

    type Pubs = Pubs;

    type Vk = EzklVk;

    fn hash_context_data() -> &'static [u8] {
        b"ezkl"
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

        log::trace!("Verifying (no-std)");
        ezkl_no_std::verify::<CurveHooksImpl>(&vk.vk_bytes, &proof, pubs)
            .inspect_err(|e| log::debug!("Cannot verify proof: {e:?}"))
            .map_err(|e| match e {
                ezkl_no_std::errors::VerifyError::VerificationError => {
                    hp_verifiers::VerifyError::VerifyError
                }
                ezkl_no_std::errors::VerifyError::PublicInputError { message: _ } => {
                    hp_verifiers::VerifyError::InvalidInput
                }
                ezkl_no_std::errors::VerifyError::KeyError { message: _ } => {
                    hp_verifiers::VerifyError::InvalidVerificationKey
                }
                ezkl_no_std::errors::VerifyError::InvalidProofError { message: _ } => {
                    hp_verifiers::VerifyError::InvalidProofData
                }
                ezkl_no_std::errors::VerifyError::OtherError { message: _ } => {
                    hp_verifiers::VerifyError::VerifyError
                }
            })
            .map(|_| None)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        if vk.vk_bytes.len() == 0
            || vk.vk_bytes.len() & 31 != 0
            || vk.vk_bytes.len() > MAX_VK_LENGTH as usize
        {
            return Err(VerifyError::InvalidVerificationKey);
        }

        Ok(())
    }

    fn vk_hash(vk: &Self::Vk) -> H256 {
        // sp_io::hashing::sha2_256(&Self::vk_bytes(vk)).into()
        sp_io::hashing::keccak_256(&Self::vk_bytes(vk)).into()
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

impl<T: Config> Ezkl<T> {
    // Utility function for future-proofing.
    fn encode_vk(vk: &Vk) -> Cow<'_, [u8]> {
        Cow::Owned(vk.vk_bytes.to_vec())
    }
}

/// The struct to use in runtime pallet configuration to map the weight computed by this crate
/// benchmarks to the weight needed by the `pallet-verifiers`.
pub struct EzklWeight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Ezkl<T>> for EzklWeight<W> {
    fn verify_proof(
        _proof: &<Ezkl<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Ezkl<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::verify_proof()
    }

    fn register_vk(_vk: &<Ezkl<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> frame_support::weights::Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Ezkl<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Ezkl<T> as Verifier>::Proof,
        _pubs: &<Ezkl<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
