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

use codec::{Decode, Encode, MaxEncodedLen};
use educe::Educe;
use errors::LibraryError;
use frame_support::weights::Weight;
use hp_verifiers::{Cow, Verifier, VerifyError};
use proof_of_sql_verifier::VerificationKey;
use scale_info::TypeInfo;
use sp_core::Get;
use sp_std::{marker::PhantomData, vec::Vec};

pub mod benchmarking;
mod errors;
mod verifier_should;
mod weight;
pub use weight::WeightInfo;

pub const LARGEST_MAX_NU_LIMIT: u32 = 8;
pub const MAX_PROOF_SIZE: u32 = 80_000;
pub const MAX_PUBS_SIZE: u32 = 12_000;

// Here educe is used for Clone, Debug, and PartialEq to work around
// a long-standing compiler bug https://github.com/rust-lang/rust/issues/26925
#[derive(Educe, Encode, Decode, TypeInfo)]
#[educe(Clone, Debug, PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct Vk<T>(Vec<u8>, PhantomData<T>);

impl<T: Config> Vk<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        let max_nu = T::largest_max_nu();
        let max_vk_size = VerificationKey::serialized_size(max_nu as usize);
        if self.0.len() > max_vk_size {
            Err(VerifyError::InvalidVerificationKey)
        } else {
            Ok(())
        }
    }
}

impl<T> From<Vec<u8>> for Vk<T> {
    fn from(value: Vec<u8>) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: Config> MaxEncodedLen for Vk<T> {
    fn max_encoded_len() -> usize {
        let max_nu = T::largest_max_nu();
        let len = VerificationKey::serialized_size(max_nu as usize);
        codec::Compact(len as u32).encoded_size() + len
    }
}

pub trait Config: 'static {
    /// Maximum value allowed for `max_nu` in the verification key
    type LargestMaxNu: Get<u32>;
}

trait LargestMaxNu {
    fn largest_max_nu() -> u32;
}

impl<T: Config> LargestMaxNu for T {
    fn largest_max_nu() -> u32 {
        <Self as Config>::LargestMaxNu::get()
    }
}

#[pallet_verifiers::verifier]
pub struct ProofOfSql<T>;

impl<T: Config> Verifier for ProofOfSql<T> {
    type Proof = Vec<u8>;

    type Pubs = Vec<u8>;

    type Vk = Vk<T>;

    fn hash_context_data() -> &'static [u8] {
        b"proofofsql"
    }

    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        vk.validate_size()?;
        if proof.len() > MAX_PROOF_SIZE as usize {
            return Err(VerifyError::InvalidProofData);
        }
        if pubs.len() > MAX_PUBS_SIZE as usize {
            return Err(VerifyError::InvalidInput);
        }
        let proof = proof_of_sql_verifier::Proof::try_from(&proof[..])
            .map_err(Into::<LibraryError>::into)?;
        let pubs = proof_of_sql_verifier::PublicInput::try_from(&pubs[..])
            .map_err(Into::<LibraryError>::into)?;
        let vk = proof_of_sql_verifier::VerificationKey::try_from_bytes_unchecked(&vk.0[..])
            .map_err(Into::<LibraryError>::into)?;
        proof_of_sql_verifier::verify_proof(&proof, &pubs, &vk)
            .map_err(Into::<LibraryError>::into)?;
        Ok(None)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        vk.validate_size()
            .inspect_err(|_| log::debug!("Verification key is too big"))?;
        Ok(())
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]> {
        let data = pubs.to_vec();
        Cow::Owned(data)
    }
}

pub struct ProofOfSqlWeight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<ProofOfSql<T>>
    for ProofOfSqlWeight<W>
{
    fn verify_proof(
        _proof: &<ProofOfSql<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<ProofOfSql<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::verify_proof()
    }

    fn register_vk(_vk: &<ProofOfSql<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> frame_support::weights::Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<ProofOfSql<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<ProofOfSql<T> as Verifier>::Proof,
        _pubs: &<ProofOfSql<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}

#[cfg(test)]
mod vk {
    use super::*;

    use frame_support::{assert_err, assert_ok};
    use rstest::{fixture, rstest};
    use sp_core::ConstU32;

    pub struct ConfigWithMaxNu<const N: u32>;

    impl<const N: u32> Config for ConfigWithMaxNu<N> {
        type LargestMaxNu = ConstU32<N>;
    }

    type VkWithMaxNu<const N: u32> = <ProofOfSql<ConfigWithMaxNu<N>> as Verifier>::Vk;

    #[fixture]
    fn bytes_of_vk_with_nu_4() -> Vec<u8> {
        include_bytes!("resources/VALID_VK_MAX_NU_4.bin").to_vec()
    }

    #[rstest]
    fn length_should_match_max_encoded_len(bytes_of_vk_with_nu_4: Vec<u8>) {
        let vk_with_max_nu_4 = VkWithMaxNu::<4>::from(bytes_of_vk_with_nu_4);
        let encoded_len = vk_with_max_nu_4.encode().len();
        let expected_encoded_len = VkWithMaxNu::<4>::max_encoded_len();
        assert_eq!(encoded_len, expected_encoded_len);
    }

    mod validate_size {
        use super::*;

        #[rstest]
        fn should_reject_too_big_vk(bytes_of_vk_with_nu_4: Vec<u8>) {
            let vk_with_max_nu_3 = VkWithMaxNu::<3>::from(bytes_of_vk_with_nu_4);
            assert_err!(
                vk_with_max_nu_3.validate_size(),
                VerifyError::InvalidVerificationKey
            );
        }

        #[rstest]
        fn should_accept_maximum_size_vk(bytes_of_vk_with_nu_4: Vec<u8>) {
            let vk_with_max_nu_4 = VkWithMaxNu::<4>::from(bytes_of_vk_with_nu_4);
            assert_ok!(vk_with_max_nu_4.validate_size());
        }

        #[rstest]
        fn should_accept_small_vk(bytes_of_vk_with_nu_4: Vec<u8>) {
            let vk_with_max_nu_5 = VkWithMaxNu::<5>::from(bytes_of_vk_with_nu_4);
            assert_ok!(vk_with_max_nu_5.validate_size());
        }
    }
}
