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
    type MaxVkSize = ConstU32<2048>;
    type WeightInfo = ();
}

fn dummy_vk() -> Vk<MockConfig> {
    Vk::new(vec![1_u8, 2, 3], KimchiSrsId::Vesta16)
}

#[test]
fn flatten_public_inputs_for_statement_hash() {
    let pubs = vec![[1_u8; PUB_SIZE], [2_u8; PUB_SIZE]];
    let flattened = Kimchi::<MockConfig>::pubs_bytes(&pubs);

    assert_eq!(flattened.len(), PUB_SIZE * 2);
    assert!(flattened[..PUB_SIZE].iter().all(|byte| *byte == 1));
    assert!(flattened[PUB_SIZE..].iter().all(|byte| *byte == 2));
}

mod accept {
    use super::*;

    fn fixture_is_accepted(proof_bytes: &[u8], verifier_index_bytes: &[u8], pubs: Pubs) {
        let raw_proof = proof_bytes.to_vec();
        let vk = Vk::<MockConfig>::new(verifier_index_bytes.to_vec(), KimchiSrsId::Vesta16);
        let proof = decode_proof(&raw_proof).expect("fixture proof should decode");
        let public_input = decode_public_input(&pubs).expect("fixture public input should decode");
        let mut verifier_index = decode_vk(&vk).expect("fixture verifier index should decode");
        prepare_verifier_index(&mut verifier_index, vk.srs_id)
            .expect("fixture verifier index should prepare");
        let mut rng = make_rng(&vk, &raw_proof, &pubs);

        verify_native_proof_with_rng(&verifier_index, &proof, &public_input, &mut rng)
            .expect("fixture proof should verify");
    }

    #[test]
    fn domain_4096_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/generated_4096/proof.bin"),
            include_bytes!("resources/generated_4096/verifier_index.bin"),
            Vec::new(),
        );
    }

    #[test]
    fn domain_65536_with_64_public_inputs_fixture_is_accepted() {
        let pubs = include_bytes!("resources/generated_65536_pubs_64/pubs.bin")
            .chunks_exact(PUB_SIZE)
            .map(|bytes| {
                bytes
                    .try_into()
                    .expect("fixture public input has field size")
            })
            .collect();

        fixture_is_accepted(
            include_bytes!("resources/generated_65536_pubs_64/proof.bin"),
            include_bytes!("resources/generated_65536_pubs_64/verifier_index.bin"),
            pubs,
        );
    }
}

mod reject {
    use super::*;
    use pallet_verifiers::traits::VerifyError;

    fn fixture_verifier_index() -> NativeVerifierIndex {
        bincode::serde::decode_from_slice(
            include_bytes!("resources/generated_4096/verifier_index.bin"),
            bincode::config::standard(),
        )
        .map(|(verifier_index, _)| verifier_index)
        .expect("fixture verifier index should decode")
    }

    fn fixture_proof_and_index() -> (Proof, NativeProof, NativeVerifierIndex) {
        let raw_proof = include_bytes!("resources/generated_4096/proof.bin").to_vec();
        let proof = decode_proof(&raw_proof).expect("fixture proof should decode");
        let mut verifier_index = fixture_verifier_index();
        prepare_verifier_index(&mut verifier_index, KimchiSrsId::Vesta16)
            .expect("fixture verifier index should prepare");

        (raw_proof, proof, verifier_index)
    }

    #[test]
    fn oversized_verifier_index_is_rejected() {
        let vk = Vk::new(
            vec![0_u8; MockConfig::max_vk_size() as usize + 1],
            KimchiSrsId::Vesta16,
        );

        assert_err!(
            Kimchi::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn empty_verifier_material_is_rejected() {
        let vk = Vk::new(Vec::new(), KimchiSrsId::Vesta16);

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
    fn domain_larger_than_srs_prefix_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.max_poly_size = 1024;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiSrsId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn unsupported_small_domain_is_rejected() {
        use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

        let mut verifier_index = fixture_verifier_index();
        verifier_index.domain =
            Radix2EvaluationDomain::new(512).expect("small radix-2 domain should exist");

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiSrsId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn zero_max_poly_size_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.max_poly_size = 0;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiSrsId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn non_power_of_two_max_poly_size_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.max_poly_size = 4095;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiSrsId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn max_poly_size_larger_than_builtin_srs_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.max_poly_size = KimchiSrsId::Vesta16.max_poly_size() * 2;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiSrsId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn excessive_declared_public_inputs_are_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.public = KimchiSrsId::Vesta16.max_public_inputs() + 1;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiSrsId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn opening_with_too_few_rounds_is_rejected() {
        let (raw_proof, mut proof, verifier_index) = fixture_proof_and_index();
        let vk = Vk::<MockConfig>::new(
            include_bytes!("resources/generated_4096/verifier_index.bin").to_vec(),
            KimchiSrsId::Vesta16,
        );
        let raw_pubs = Vec::new();
        proof.proof.clear_rounds_for_test();
        let mut rng = make_rng(&vk, &raw_proof, &raw_pubs);

        assert_eq!(
            verify_native_proof_with_rng(&verifier_index, &proof, &[], &mut rng),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn opening_with_too_many_rounds_is_rejected() {
        let (raw_proof, mut proof, verifier_index) = fixture_proof_and_index();
        let vk = Vk::<MockConfig>::new(
            include_bytes!("resources/generated_4096/verifier_index.bin").to_vec(),
            KimchiSrsId::Vesta16,
        );
        let raw_pubs = Vec::new();
        proof.proof.duplicate_round_for_test();
        let mut rng = make_rng(&vk, &raw_proof, &raw_pubs);

        assert_eq!(
            verify_native_proof_with_rng(&verifier_index, &proof, &[], &mut rng),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn corrupted_opening_scalar_is_rejected() {
        let (raw_proof, mut proof, verifier_index) = fixture_proof_and_index();
        let vk = Vk::<MockConfig>::new(
            include_bytes!("resources/generated_4096/verifier_index.bin").to_vec(),
            KimchiSrsId::Vesta16,
        );
        let raw_pubs = Vec::new();
        proof.proof.corrupt_z1_for_test();
        let mut rng = make_rng(&vk, &raw_proof, &raw_pubs);

        assert_eq!(
            verify_native_proof_with_rng(&verifier_index, &proof, &[], &mut rng),
            Err(VerifyError::VerifyError)
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
