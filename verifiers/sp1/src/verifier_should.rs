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

#![cfg(test)]

use sp_core::ConstU32;

use super::*;

include!("resources/vk_pubs.rs");

struct Mock;
impl Config for Mock {
    type MaxPubsSize = ConstU32<32>;
}

#[test]
fn verify_valid_proof() {
    let proof = include_bytes!("resources/proof.bin").to_vec();
    let pubs = VALID_PUBS.to_vec();
    let vk = VALID_VK;

    assert!(Sp1::<Mock>::verify_proof(&vk, &proof, &pubs).is_ok());
}

#[test]
fn reject_too_long_proof() {
    let proof = vec![0u8; MAX_PROOF_SIZE + 1];
    let pubs = VALID_PUBS.to_vec();
    let vk = VALID_VK;

    assert_eq!(
        Sp1::<Mock>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::InvalidProofData)
    )
}

#[test]
fn reject_too_long_pubs() {
    let proof = include_bytes!("resources/proof.bin").to_vec();
    let pubs = vec![0u8; Mock::max_pubs_size() as usize + 1];
    let vk = VALID_VK;

    assert_eq!(
        Sp1::<Mock>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::InvalidInput)
    )
}

#[test]
fn reject_invalid_proof() {
    let mut proof = include_bytes!("resources/proof.bin").to_vec();
    let pubs = VALID_PUBS.to_vec();
    let vk = VALID_VK;

    proof[0] = proof[0].wrapping_add(0x01);

    assert_eq!(
        Sp1::<Mock>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    )
}

#[test]
fn reject_invalid_vk() {
    let proof = include_bytes!("resources/proof.bin").to_vec();
    let pubs = VALID_PUBS.to_vec();
    let mut vk = VALID_VK;

    let vk_0 = vk.as_bytes()[0];
    vk.as_bytes_mut()[0] = vk_0.wrapping_add(0x01);

    assert_eq!(
        Sp1::<Mock>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    )
}

#[test]
fn reject_invalid_pubs() {
    let proof = include_bytes!("resources/proof.bin").to_vec();
    let mut pubs = VALID_PUBS.to_vec();
    let vk = VALID_VK;

    pubs[0] = pubs[0].wrapping_add(0x01);

    assert_eq!(
        Sp1::<Mock>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    )
}

/// Add the `BabyBear` modulus to the `word`-th big-endian 4-byte word of `vk`.
///
/// The result is a different 32-byte string that the SP1 verifier maps to the same field
/// element, because `P < 2^31` makes `w + P` fit in a `u32` for every canonical `w`.
fn alias_word(mut vk: H256, word: usize) -> H256 {
    let at = word * 4;
    let bytes = vk.as_bytes_mut();
    let aliased = u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        .checked_add(BABY_BEAR_P)
        .expect("a canonical word plus P always fits in a u32");
    bytes[at..at + 4].copy_from_slice(&aliased.to_be_bytes());
    vk
}

#[test]
fn reject_non_canonical_vk() {
    let proof = include_bytes!("resources/proof.bin").to_vec();
    let pubs = VALID_PUBS.to_vec();

    // Unlike `reject_invalid_vk`, which perturbs the vk into a genuinely different digest and
    // so fails at verification, each of these is an alias of `VALID_VK`: the verifier would
    // accept it and attest it as a distinct statement.
    for word in 0..8 {
        assert_eq!(
            Sp1::<Mock>::verify_proof(&alias_word(VALID_VK, word), &proof, &pubs),
            Err(VerifyError::InvalidVerificationKey),
            "word {word} aliased by P was not rejected"
        );
    }
}

#[test]
fn validate_canonical_vk() {
    assert_eq!(Sp1::<Mock>::validate_vk(&VALID_VK), Ok(()));
}

#[test]
fn reject_non_canonical_vk_on_validate() {
    for word in 0..8 {
        assert_eq!(
            Sp1::<Mock>::validate_vk(&alias_word(VALID_VK, word)),
            Err(VerifyError::InvalidVerificationKey),
            "word {word} aliased by P was not rejected"
        );
    }
}

#[test]
fn validate_vk_on_the_canonical_boundary() {
    let vk_from_first_word = |word: u32| {
        let mut vk = H256::zero();
        vk.as_bytes_mut()[..4].copy_from_slice(&word.to_be_bytes());
        vk
    };

    assert_eq!(
        Sp1::<Mock>::validate_vk(&vk_from_first_word(BABY_BEAR_P - 1)),
        Ok(())
    );
    assert_eq!(
        Sp1::<Mock>::validate_vk(&vk_from_first_word(BABY_BEAR_P)),
        Err(VerifyError::InvalidVerificationKey)
    );
}
