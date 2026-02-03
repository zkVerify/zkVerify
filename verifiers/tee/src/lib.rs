// Copyright 2025, Horizen Labs, Inc.
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
mod verifier_should;
mod weight;

use alloc::{borrow::Cow, vec::Vec, vec};
use core::marker::PhantomData;

use frame_support::{ensure, traits::UnixTime, weights::Weight};
use hp_verifiers::Verifier;
pub use weight::WeightInfo;

use hp_verifiers::VerifyError;
use tee_verifier::{cert::RevokedCertId, intel::{collaterals::TcbResponse, quote::QuoteV4}};

use chrono::DateTime;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[pallet_verifiers::verifier]
pub struct Tee<T>;

pub type Proof = Vec<u8>;
pub type Pubs = Vec<u8>;

// Max size in bytes of the vk payloads
pub const MAX_VK_LENGTH: u32 = 8192;
// Max size in bytes of the quote
pub const MAX_PROOF_LENGTH: u32 = 8192;
// Max size in bytes of the pubs; this pallet does not need any pubs
pub const MAX_PUBS_LENGTH: u32 = 0;

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub struct Vk {
    pub tcb_response: Vec<u8>,
    pub certificates: Vec<u8>,
}

impl MaxEncodedLen for Vk {
    fn max_encoded_len() -> usize {
        // codec::Compact(len).encoded_size() + element_size * len as usize
        codec::Compact(2 * MAX_VK_LENGTH).encoded_size() + (2 * MAX_VK_LENGTH) as usize
    }
}

pub trait Config {
    type UnixTime: UnixTime;
}

impl<T: Config> Verifier for Tee<T> {
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
    ) -> Result<Option<Weight>, hp_verifiers::VerifyError> {
        ensure!(
            proof.len() <= MAX_PROOF_LENGTH as usize,
            VerifyError::InvalidProofData
        );
        ensure!(
            _pubs.len() <= MAX_PUBS_LENGTH as usize,
            VerifyError::InvalidInput
        );
        ensure!(
            vk.tcb_response.len() <= MAX_VK_LENGTH as usize
                && vk.certificates.len() <= MAX_VK_LENGTH as usize
                && !vk.tcb_response.is_empty()
                && !vk.certificates.is_empty(),
            VerifyError::InvalidVerificationKey
        );

        let quote = QuoteV4::from_bytes(&proof[..]).map_err(|_| VerifyError::InvalidProofData)?;
        let (tcb_response, _used): (TcbResponse, usize) =
            serde_json_core::from_slice(&vk.tcb_response[..])
                .map_err(|_| VerifyError::InvalidInput)?;

        // Check that the tcbInfo is still valid at the verification timestamp
        tcb_response
            .tcb_info
            .verify(
                DateTime::from_timestamp_secs(T::UnixTime::now().as_secs().try_into().unwrap())
                    .unwrap(),
            )
            .map_err(|_| VerifyError::VerifyError)?;

        let crl: Vec<RevokedCertId> = vec![];
        // Verify the attestation
        quote
            .verify(Some(tcb_response.tcb_info), &crl)
            .map_err(|_| VerifyError::VerifyError)
            .map(|_| None)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        if vk.tcb_response.len() > MAX_VK_LENGTH as usize
            || vk.certificates.len() > MAX_VK_LENGTH as usize
            || vk.tcb_response.is_empty()
            || vk.certificates.is_empty()
        {
            return Err(VerifyError::InvalidVerificationKey);
        }

        let (tcb_response, _used): (TcbResponse, usize) =
            serde_json_core::from_slice(&vk.tcb_response[..])
                .map_err(|_| VerifyError::InvalidVerificationKey)?;

        let crl: Vec<RevokedCertId> = vec![];
        // Check that the tcbInfo is still valid at the verification timestamp and that the
        // signature is valid
        tcb_response
            .verify(
                vk.certificates.to_vec(),
                DateTime::from_timestamp_secs(T::UnixTime::now().as_secs().try_into().unwrap())
                    .unwrap(),
                &crl
            )
            .map_err(|_| VerifyError::VerifyError)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
        Cow::Borrowed(pubs)
    }
}

pub struct TeeWeight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Tee<T>> for TeeWeight<W> {
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
