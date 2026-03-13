// Copyright 2025-2026, Horizen Labs, Inc.
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

pub mod benchmarking;
pub mod migrations;
mod verifier_should;
mod weight;

use alloc::{borrow::Cow, vec::Vec};
use core::marker::PhantomData;

use frame_support::{ensure, pallet_prelude::StorageVersion, traits::UnixTime, weights::Weight};
use pallet_verifiers::traits::Verifier;
pub use weight::WeightInfo;

use pallet_crl::CrlProvider;
use pallet_verifiers::traits::VerifyError;
use tee_verifier::{nitro_parse_attestation, intel_parse_quote, intel_parse_tcb_response, TcbResponse};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[pallet_verifiers::verifier]
pub struct Tee<T>;

pub type Proof = Vec<u8>;
pub type Pubs = Vec<u8>;

// Max size in bytes of the vk payloads
pub const MAX_VK_LENGTH: u32 = 65536;
// Max size in bytes of the quote
pub const MAX_PROOF_LENGTH: u32 = 65536;
// Max size in bytes of the pubs; this pallet does not need any pubs
pub const MAX_PUBS_LENGTH: u32 = 0;

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub enum Vk {
    Intel {
        tcb_response: Vec<u8>,
        certificates: Vec<u8>,
    },
    Nitro,
}

impl MaxEncodedLen for Vk {
    fn max_encoded_len() -> usize {
        // codec::Compact(len).encoded_size() + element_size * len as usize
        codec::Compact(2 * MAX_VK_LENGTH).encoded_size() + (2 * MAX_VK_LENGTH) as usize
    }
}

pub trait CaNameProvider {
    fn ca_name_for(vk: &Vk) -> &'static str;
}

pub trait Config {
    type UnixTime: UnixTime;
    type Crl: CrlProvider;
    type CaName: CaNameProvider;
}

impl<T: Config> Verifier for Tee<T> {
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    type Proof = Proof;

    type Pubs = Pubs;

    type Vk = Vk;

    fn hash_context_data() -> &'static [u8] {
        b"tee"
    }

    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        _pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        ensure!(
            proof.len() <= MAX_PROOF_LENGTH as usize,
            VerifyError::InvalidProofData
        );
        ensure!(
            _pubs.len() <= MAX_PUBS_LENGTH as usize,
            VerifyError::InvalidInput
        );

        let now = T::UnixTime::now().as_secs();

        // Always require the CA to be registered, even if empty, to allow for unpermissioned CRL
        // updates.
        let crl =
            T::Crl::get_crl(T::CaName::ca_name_for(vk)).map_err(|_| VerifyError::VerifyError)?;

        match vk {
            Vk::Intel {
                tcb_response,
                certificates,
            } => {
                ensure!(
                    tcb_response.len() <= MAX_VK_LENGTH as usize
                        && certificates.len() <= MAX_VK_LENGTH as usize
                        && !tcb_response.is_empty()
                        && !certificates.is_empty(),
                    VerifyError::InvalidVerificationKey
                );

                let quote = intel_parse_quote(proof).map_err(|_| VerifyError::InvalidProofData)?;
                let tcb_response =
                    intel_parse_tcb_response(&tcb_response[..]).map_err(|_| VerifyError::InvalidInput)?;

                tcb_response
                    .tcb_info
                    .verify(now)
                    .map_err(|_| VerifyError::VerifyError)?;

                quote
                    .verify(&tcb_response.tcb_info, &crl, now)
                    .map_err(|_| VerifyError::VerifyError)
                    .map(|_| None)
            }
            Vk::Nitro => {
                let attestation =
                    nitro_parse_attestation(proof).map_err(|_| VerifyError::InvalidProofData)?;

                attestation
                    .verify(Some(&crl), now)
                    .map_err(|_| VerifyError::VerifyError)
                    .map(|_| None)
            }
        }
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        match vk {
            Vk::Intel {
                tcb_response,
                certificates,
            } => {
                if tcb_response.len() > MAX_VK_LENGTH as usize
                    || certificates.len() > MAX_VK_LENGTH as usize
                    || tcb_response.is_empty()
                    || certificates.is_empty()
                {
                    return Err(VerifyError::InvalidVerificationKey);
                }

                let (tcb_response, _used): (TcbResponse, usize) =
                    serde_json_core::from_slice(&tcb_response[..])
                        .map_err(|_| VerifyError::InvalidVerificationKey)?;

                let now = T::UnixTime::now().as_secs();
                let crl = T::Crl::get_crl(T::CaName::ca_name_for(vk))
                    .map_err(|_| VerifyError::VerifyError)?;
                tcb_response
                    .verify(certificates.to_vec(), &crl, now)
                    .map_err(|_| VerifyError::VerifyError)
            }
            // Nitro has no VK data to validate, the attestation document is self-contained
            // => no point in registering a vk
            Vk::Nitro => Err(VerifyError::InvalidVerificationKey),
        }
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        Cow::Borrowed(pubs)
    }
}

pub struct TeeWeight<W: WeightInfo>(PhantomData<W>);

impl<T: Config, W: WeightInfo> pallet_verifiers::WeightInfo<Tee<T>> for TeeWeight<W> {
    fn verify_proof(
        _proof: &<Tee<T> as Verifier>::Proof,
        _pubs: &<Tee<T> as Verifier>::Pubs,
    ) -> Weight {
        W::verify_proof()
    }

    fn register_vk(_vk: &<Tee<T> as Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Tee<T> as Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Tee<T> as Verifier>::Proof,
        _pubs: &<Tee<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
