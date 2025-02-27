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

#![cfg(test)]

use rstest::rstest;
use sp_core::ConstU32;

use super::*;

struct Mock;

impl Config for Mock {
    type MaxProofSize = ConstU32<1000000>; // arbitrary length for tests
    type MaxPubsSize = ConstU32<100>; // arbitrary length for tests
}

include!("resources.rs");

#[rstest]
#[case(&v1_0::VALID_VK, Proof::V1_0(v1_0::VALID_PROOF.to_vec()), &v1_0::VALID_PUBS)]
#[case(&v1_1::VALID_VK, Proof::V1_1(v1_1::VALID_PROOF.to_vec()), &v1_1::VALID_PUBS)]
#[case(&v1_2::VALID_VK, Proof::V1_2(v1_2::VALID_PROOF.to_vec()), &v1_2::VALID_PUBS)]
fn verify_valid_proof(#[case] vk: &Vk, #[case] proof: Proof, #[case] pubs: &[u8]) {
    assert!(Risc0::<Mock>::verify_proof(vk, &proof, &pubs.to_vec()).is_ok());
}

#[rstest]
#[case::v1_0(
    Proof::V1_0(Default::default()),
    H256::from(sp_io::hashing::sha2_256(b"risc0:v1.0"))
)]
#[case::v1_1(
    Proof::V1_1(Default::default()),
    H256::from(sp_io::hashing::sha2_256(b"risc0:v1.1"))
)]
#[case::v1_2(
    Proof::V1_2(Default::default()),
    H256::from(sp_io::hashing::sha2_256(b"risc0:v1.2"))
)]
#[case::do_not_depend_on_proof_content(
    Proof::V1_2([0xde;16].to_vec()),
    H256::from(sp_io::hashing::sha2_256(b"risc0:v1.2"))
)]
fn return_the_correct_verifier_version_hash(#[case] proof: Proof, #[case] expected: H256) {
    let h = Risc0::<Mock>::verifier_version_hash(&proof);

    assert_eq!(h, expected)
}

mod reject {
    use hp_verifiers::VerifyError;

    use super::*;

    #[test]
    fn invalid_proof() {
        let mut invalid_pubs = v1_0::VALID_PUBS.clone();
        invalid_pubs[invalid_pubs.len() - 1] = invalid_pubs[invalid_pubs.len() - 1].wrapping_add(1);
        let proof = Proof::V1_0(v1_0::VALID_PROOF.to_vec());
        assert_eq!(
            Risc0::<Mock>::verify_proof(&v1_0::VALID_VK, &proof, &invalid_pubs.to_vec()),
            Err(VerifyError::VerifyError)
        )
    }

    #[test]
    fn undeserializable_proof() {
        let mut malformed_proof = v1_0::VALID_PROOF.clone();
        malformed_proof[0] = malformed_proof[0].wrapping_add(1);
        let proof = Proof::V1_0(malformed_proof.to_vec());
        assert_eq!(
            Risc0::<Mock>::verify_proof(&v1_0::VALID_VK, &proof, &v1_0::VALID_PUBS.to_vec()),
            Err(VerifyError::InvalidProofData)
        )
    }

    #[test]
    fn too_big_proof() {
        let too_big_proof = vec![0; Mock::max_proof_size() as usize + 1];
        let proof = Proof::V1_0(too_big_proof);
        assert_eq!(
            Risc0::<Mock>::verify_proof(&v1_0::VALID_VK, &proof, &v1_0::VALID_PUBS.to_vec()),
            Err(VerifyError::InvalidProofData)
        )
    }

    #[test]
    fn too_big_pubs() {
        let too_big_pubs = vec![0; Mock::max_pubs_size() as usize + 1];
        let proof = Proof::V1_0(v1_0::VALID_PROOF.to_vec());
        assert_eq!(
            Risc0::<Mock>::verify_proof(&v1_0::VALID_VK, &proof, &too_big_pubs),
            Err(VerifyError::InvalidInput)
        )
    }
}
