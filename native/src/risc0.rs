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

use risc0_verifier::poseidon2_injection::{BabyBearElem, POSEIDON2_CELLS};

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

        let inner_receipt = deserialize_proof(proof)?;
        let journal = deserialize_pubs(pubs)?;

        let ctx = risc0_verifier::VerifierContext::v1_0();
        let proof = risc0_verifier::Proof::new(inner_receipt);
        proof
            .verify(&ctx, vk, journal.digest())
            .map_err(|_| VerifyError::VerifyError)
    }

    /// Return if the proof is a `Fake` proof
    fn is_fake_proof(proof: &[u8]) -> bool {
        proof.starts_with(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    }
}

#[runtime_interface]
pub trait Risc0Verify {
    fn verify(vk: [u8; 32], proof: &[u8], pubs: &[u8]) -> Result<(), VerifyError> {
        legacy_impl::verify(vk, proof, pubs)
    }
}

/// Define the byte slice for poseidon2 mix call argument type.
pub type Poseidon2ArgBytes = [u8; (u32::BITS as usize / u8::BITS as usize) * POSEIDON2_CELLS];
/// Define the `BabyBearElem` slice for poseidon2 mix call argument type.
type Poseidon2Slice = [BabyBearElem; POSEIDON2_CELLS];

/// A struct that hide the `poseidon2_mix()` function native call. Use this struct call the native
/// implementation gives the possibility to be sure of don't pass some invalid data that is different from
/// a `BabyBearElem` `POSEIDON2_CELLS` array.
pub struct Poseidon2Mix<'a> {
    inner: &'a mut Poseidon2Slice,
}

impl<'a> Poseidon2Mix<'a> {
    #[inline]
    /// Create a new `Poseidon2Mix`.
    pub fn new(cells: &'a mut Poseidon2Slice) -> Self {
        Self { inner: cells }
    }

    /// Consume `self` and call the native `poseidon2_mix()` function on the inner
    /// `BabyBearElem` array.
    pub fn poseidon2_mix(self) {
        risc_0_accelerate::poseidon2_mix(self.into_mut_bytes())
    }

    #[inline]
    #[cfg(feature = "std")]
    /// SAFETY: BabyBearElem is always u32 and use `repr(transparent)`. Moreover
    /// this method is private and it's just used by `poseidon2_mix` that cannot be
    /// accessed outside of this module: only `Self::poseidon2_mix()` call it
    /// that can be just called from a `Poseidon2Mix` struct.
    /// The `Poseidon2Mix` struct can be built just from a mutable slice of `BabyBearElem`
    /// with the correct size.
    fn from_mut_bytes(bytes: &mut Poseidon2ArgBytes) -> Self {
        Self::new(unsafe {
            core::mem::transmute::<&mut Poseidon2ArgBytes, &mut Poseidon2Slice>(bytes)
        })
    }

    /// SAFETY: BabyBearElem is always u32 and use `repr(transparent)`. The inner
    /// mut slice can just be built from a mutable slice of `BabyBearElem`
    /// with the correct size. Moreover, all invariants of the `BabyBearElem` will
    /// be maintained from the only operation that can change them: `poseidon2_mix()`
    /// that get them again as `BabyBearElem`'s before call the `poseidon2_mix()` from
    /// risc0_verifier.
    fn into_mut_bytes(self) -> &'a mut Poseidon2ArgBytes {
        unsafe { core::mem::transmute::<&mut Poseidon2Slice, &mut Poseidon2ArgBytes>(self.inner) }
    }
}

#[runtime_interface]
pub trait Risc0Accelerate {
    fn poseidon2_mix(bytes: &mut Poseidon2ArgBytes) {
        let cells = Poseidon2Mix::from_mut_bytes(bytes);
        risc0_verifier::poseidon2_injection::poseidon2_mix(cells.inner);
    }
}
