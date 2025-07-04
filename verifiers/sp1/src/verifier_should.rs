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
    let proof = vec![0u8; crate::MAX_PROOF_SIZE + 1];
    let pubs = VALID_PUBS.to_vec();
    let vk = VALID_VK;

    assert_eq!(
        Sp1::<Mock>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::InvalidProofData)
    )
}

#[test]
fn reject_too_long_pubs() {
    let proof = include_bytes!("resources/proof.bin").to_vec();
    let pubs = vec![0u8; Mock::max_pubs_size() as usize + 1];
    let vk = VALID_VK;

    assert_eq!(
        Sp1::<Mock>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::InvalidInput)
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
        Err(hp_verifiers::VerifyError::VerifyError)
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
        Err(hp_verifiers::VerifyError::VerifyError)
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
        Err(hp_verifiers::VerifyError::VerifyError)
    )
}
