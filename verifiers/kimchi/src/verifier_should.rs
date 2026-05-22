// Copyright 2026, Horizen Labs, Inc.
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

use super::*;
use frame_support::{assert_err, traits::ConstU32};

pub struct MockConfig;

impl crate::Config for MockConfig {
    type MaxProofSize = ConstU32<1024>;
    type MaxPubs = ConstU32<4>;
    type MaxSrsSize = ConstU32<2048>;
    type MaxVkSize = ConstU32<2048>;
    type WeightInfo = ();
}

fn dummy_vk() -> Vk<MockConfig> {
    Vk::new(vec![1_u8, 2, 3], vec![4_u8, 5, 6])
}

#[test]
fn flatten_public_inputs_for_statement_hash() {
    let pubs = vec![[1_u8; PUB_SIZE], [2_u8; PUB_SIZE]];
    let flattened = Kimchi::<MockConfig>::pubs_bytes(&pubs);

    assert_eq!(flattened.len(), PUB_SIZE * 2);
    assert!(flattened[..PUB_SIZE].iter().all(|byte| *byte == 1));
    assert!(flattened[PUB_SIZE..].iter().all(|byte| *byte == 2));
}

mod reject {
    use super::*;
    use pallet_verifiers::traits::VerifyError;

    #[test]
    fn oversized_verifier_index_is_rejected() {
        let vk = Vk::new(
            vec![0_u8; MockConfig::max_vk_size() as usize + 1],
            vec![1_u8],
        );

        assert_err!(
            Kimchi::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn oversized_srs_is_rejected() {
        let vk = Vk::new(
            vec![1_u8],
            vec![0_u8; MockConfig::max_srs_size() as usize + 1],
        );

        assert_err!(
            Kimchi::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn empty_verifier_material_is_rejected() {
        let vk = Vk::new(Vec::new(), Vec::new());

        assert_err!(
            Kimchi::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn malformed_verifier_material_is_rejected() {
        assert_err!(
            Kimchi::<MockConfig>::validate_vk(&dummy_vk()),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn oversized_proof_is_rejected() {
        let proof = vec![0_u8; MockConfig::max_proof_size() as usize + 1];
        let pubs = vec![[0_u8; PUB_SIZE]];

        assert_err!(
            Kimchi::<MockConfig>::verify_proof(&dummy_vk(), &proof, &pubs),
            VerifyError::InvalidProofData
        );
    }

    #[test]
    fn oversized_public_inputs_are_rejected() {
        let proof = vec![0_u8; 8];
        let pubs = vec![[0_u8; PUB_SIZE]; MockConfig::max_pubs() as usize + 1];

        assert_err!(
            Kimchi::<MockConfig>::verify_proof(&dummy_vk(), &proof, &pubs),
            VerifyError::InvalidInput
        );
    }

    #[test]
    fn malformed_proof_is_rejected_before_verification() {
        let proof = vec![7_u8; 8];
        let pubs = vec![[0_u8; PUB_SIZE]];

        assert_err!(
            Kimchi::<MockConfig>::verify_proof(&dummy_vk(), &proof, &pubs),
            VerifyError::InvalidProofData
        );
    }
}
