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

use codec::{Decode, Encode};
use sp_runtime_interface::pass_by::PassByCodec;

mod groth16;
mod ultraplonk;
mod zksync;

#[derive(PassByCodec, Encode, Decode)]
pub enum VerifyError {
    InvalidInput,
    InvalidProofData,
    VerifyError,
    InvalidVerificationKey,
}

impl From<VerifyError> for hp_verifiers::VerifyError {
    fn from(value: VerifyError) -> Self {
        match value {
            VerifyError::InvalidInput => hp_verifiers::VerifyError::InvalidInput,
            VerifyError::InvalidProofData => hp_verifiers::VerifyError::InvalidProofData,
            VerifyError::InvalidVerificationKey => {
                hp_verifiers::VerifyError::InvalidVerificationKey
            }
            VerifyError::VerifyError => hp_verifiers::VerifyError::VerifyError,
        }
    }
}

pub use zksync::zksync_verify;
pub use zksync::PROOF_SIZE as ZKSYNC_PROOF_SIZE;
pub use zksync::PUBS_SIZE as ZKSYNC_PUBS_SIZE;
#[cfg(feature = "std")]
pub use zksync_verify::HostFunctions as ZksyncVerifierHostFunctions;

pub use ultraplonk::ultraplonk_verify;
#[cfg(feature = "std")]
pub use ultraplonk::ultraplonk_verify::HostFunctions as UltraplonkVerifierHostFunctions;
pub use ultraplonk::PROOF_SIZE as ULTRAPLONK_PROOF_SIZE;
pub use ultraplonk::PUBS_SIZE as ULTRAPLONK_PUBS_SIZE;
pub use ultraplonk::VK_SIZE as ULTRAPLONK_VK_SIZE;

pub use groth16::groth_16_bls_12_381_verify;
#[cfg(feature = "std")]
pub use groth16::groth_16_bls_12_381_verify::HostFunctions as Groth16Bls12VerifierHostFunctions;
pub use groth16::groth_16_bn_254_verify;
#[cfg(feature = "std")]
pub use groth16::groth_16_bn_254_verify::HostFunctions as Groth16Bn254VerifierHostFunctions;

#[cfg(feature = "std")]
pub type HLNativeHostFunctions = (
    ZksyncVerifierHostFunctions,
    UltraplonkVerifierHostFunctions,
    Groth16Bn254VerifierHostFunctions,
    Groth16Bls12VerifierHostFunctions,
);
