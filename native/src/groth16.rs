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

use hp_groth16::{Groth16Error, Proof, Scalar, VerificationKey};
use sp_runtime_interface::pass_by::{
    AllocateAndReturnByCodec, PassFatPointerAndDecode, PassFatPointerAndDecodeSlice,
};
use sp_runtime_interface::runtime_interface;

use crate::VerifyError;

impl From<Groth16Error> for VerifyError {
    fn from(error: Groth16Error) -> Self {
        match error {
            Groth16Error::InvalidProof => VerifyError::InvalidProofData,
            Groth16Error::InvalidVerificationKey => VerifyError::InvalidVerificationKey,
            Groth16Error::InvalidInput => VerifyError::InvalidInput,
            Groth16Error::VerifyError => VerifyError::VerifyError,
        }
    }
}

#[runtime_interface]
pub trait Groth16Bn254Verify {
    fn verify(
        vk: PassFatPointerAndDecode<VerificationKey>,
        proof: PassFatPointerAndDecode<Proof>,
        pubs: PassFatPointerAndDecodeSlice<&[Scalar]>,
    ) -> AllocateAndReturnByCodec<Result<bool, VerifyError>> {
        hp_groth16::verify_proof::<hp_groth16::Bn254>(vk, proof, pubs).map_err(Into::into)
    }
    fn validate_key(
        vk: PassFatPointerAndDecode<VerificationKey>,
    ) -> AllocateAndReturnByCodec<Result<(), VerifyError>> {
        hp_groth16::validate_key::<hp_groth16::Bn254>(vk).map_err(Into::into)
    }
}

#[runtime_interface]
pub trait Groth16Bls12_381Verify {
    fn verify(
        vk: PassFatPointerAndDecode<VerificationKey>,
        proof: PassFatPointerAndDecode<Proof>,
        pubs: PassFatPointerAndDecodeSlice<&[Scalar]>,
    ) -> AllocateAndReturnByCodec<Result<bool, VerifyError>> {
        hp_groth16::verify_proof::<hp_groth16::Bls12_381>(vk, proof, pubs).map_err(Into::into)
    }
    fn validate_key(
        vk: PassFatPointerAndDecode<VerificationKey>,
    ) -> AllocateAndReturnByCodec<Result<(), VerifyError>> {
        hp_groth16::validate_key::<hp_groth16::Bls12_381>(vk).map_err(Into::into)
    }
}
