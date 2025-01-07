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

use crate::VerifyError;
use sp_runtime_interface::runtime_interface;

#[cfg(feature = "std")]
mod legacy_impl {
    use crate::VerifyError;

    fn deserialize_proof(proof: &[u8]) -> Result<risc0_verifier::InnerReceipt, VerifyError> {
        bincode::deserialize(proof).map_err(|_x| {
            if is_fake_proof(proof) {
                VerifyError::VerifyError
            } else {
                VerifyError::InvalidProofData
            }
        })
    }

    fn deserialize_pubs(pubs: &[u8]) -> Result<risc0_verifier::Journal, VerifyError> {
        bincode::deserialize(pubs).map_err(|_x| VerifyError::InvalidInput)
    }

    pub fn verify(vk: [u8; 32], proof: &[u8], pubs: &[u8]) -> Result<(), VerifyError> {
        use risc0_verifier::Digestible;

        let inner_receipt = deserialize_proof(&proof)?;
        let journal = deserialize_pubs(&pubs)?;

        let ctx = risc0_verifier::VerifierContext::v1_0();
        let proof = risc0_verifier::Proof::new(inner_receipt);
        proof
            .verify(&ctx, vk, journal.digest())
            .map_err(|_| VerifyError::VerifyError)
    }

    /// Return if the proof is an Fake or Goth16 proof
    fn is_fake_proof(proof: &[u8]) -> bool {
        return proof.starts_with(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0]);
    }
}

#[runtime_interface]
pub trait Risc0Verify {
    fn verify(vk: [u8; 32], proof: &[u8], pubs: &[u8]) -> Result<(), VerifyError> {
        legacy_impl::verify(vk, proof, pubs)
    }
}
