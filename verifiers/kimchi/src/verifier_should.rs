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

pub struct ProductionEnvelopeConfig;

impl crate::Config for ProductionEnvelopeConfig {
    type MaxProofSize = ConstU32<262144>;
    type MaxPubs = ConstU32<1024>;
    type MaxVkSize = ConstU32<65536>;
    type WeightInfo = ();
}

fn dummy_vk() -> Vk<MockConfig> {
    Vk::new(vec![1_u8, 2, 3], KimchiProfileId::Vesta16)
}

fn add_public_inputs(weight: Weight, public_inputs: u64) -> Weight {
    weight.saturating_add(
        <() as WeightInfoVerifyProof>::verify_proof_public_input().saturating_mul(public_inputs),
    )
}

#[test]
fn flatten_public_inputs_for_statement_hash() {
    let pubs = vec![[1_u8; PUB_SIZE], [2_u8; PUB_SIZE]];
    let flattened = Kimchi::<MockConfig>::pubs_bytes(&pubs);

    assert_eq!(flattened.len(), PUB_SIZE * 2);
    assert!(flattened[..PUB_SIZE].iter().all(|byte| *byte == 1));
    assert!(flattened[PUB_SIZE..].iter().all(|byte| *byte == 2));
}

#[test]
fn two_chunk_profile_uses_upper_verification_weight() {
    let verifier_index: Vesta16VerifierIndex = bincode::serde::decode_from_slice(
        include_bytes!("resources/generated_131072_lookup_runtime_pubs_64/verifier_index.bin"),
        bincode::config::standard(),
    )
    .map(|(index, _)| index)
    .expect("two-chunk fixture verifier index should decode");

    assert_eq!(
        compute_verify_weight::<MockConfig>(&verifier_index),
        add_public_inputs(
            <() as WeightInfoVerifyProof>::verify_proof_domain_131072_pubs_0(),
            64
        )
    );
}

#[test]
fn four_chunk_profile_uses_upper_verification_weight() {
    let verifier_index: Vesta16VerifierIndex = bincode::serde::decode_from_slice(
        include_bytes!("resources/generated_262144_all_features_pubs_0/verifier_index.bin"),
        bincode::config::standard(),
    )
    .map(|(index, _)| index)
    .expect("four-chunk all-features fixture verifier index should decode");

    assert_eq!(
        compute_verify_weight::<MockConfig>(&verifier_index),
        <() as WeightInfoVerifyProof>::verify_proof_domain_262144_pubs_0()
    );
}

#[test]
fn four_chunk_profile_adds_public_input_weight() {
    let verifier_index: Vesta16VerifierIndex = bincode::serde::decode_from_slice(
        include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/verifier_index.bin"),
        bincode::config::standard(),
    )
    .map(|(index, _)| index)
    .expect("four-chunk public-input fixture verifier index should decode");

    assert_eq!(
        compute_verify_weight::<MockConfig>(&verifier_index),
        add_public_inputs(
            <() as WeightInfoVerifyProof>::verify_proof_domain_262144_pubs_0(),
            1024
        )
    );
}

#[test]
fn small_domain_with_fixed_vesta16_srs_uses_upper_one_chunk_weight() {
    let verifier_index: Vesta16VerifierIndex = bincode::serde::decode_from_slice(
        include_bytes!("resources/generated_4096_lookup_runtime_pubs_64/verifier_index.bin"),
        bincode::config::standard(),
    )
    .map(|(index, _)| index)
    .expect("small-domain fixed-SRS fixture verifier index should decode");

    assert_eq!(
        compute_verify_weight::<MockConfig>(&verifier_index),
        add_public_inputs(
            <() as WeightInfoVerifyProof>::verify_proof_domain_65536_pubs_0(),
            64
        )
    );
}

#[test]
fn small_max_feature_profile_uses_upper_one_chunk_weight() {
    let verifier_index: Vesta16VerifierIndex = bincode::serde::decode_from_slice(
        include_bytes!(
            "resources/generated_4096_maxpoly_4096_lookup_runtime_pubs_64/verifier_index.bin"
        ),
        bincode::config::standard(),
    )
    .map(|(index, _)| index)
    .expect("small max-feature fixture verifier index should decode");

    assert_eq!(
        compute_verify_weight::<MockConfig>(&verifier_index),
        add_public_inputs(
            <() as WeightInfoVerifyProof>::verify_proof_domain_65536_pubs_0(),
            64
        )
    );
}

#[test]
fn small_base_profile_uses_small_verification_weight() {
    let verifier_index: Vesta16VerifierIndex = bincode::serde::decode_from_slice(
        include_bytes!("resources/generated_4096/verifier_index.bin"),
        bincode::config::standard(),
    )
    .map(|(index, _)| index)
    .expect("small base fixture verifier index should decode");

    assert_eq!(
        compute_verify_weight::<MockConfig>(&verifier_index),
        <() as WeightInfoVerifyProof>::verify_proof_domain_4096_pubs_0()
    );
}

#[test]
fn small_simple_profile_uses_small_weight_with_public_input_coefficient() {
    let verifier_index: Vesta16VerifierIndex = bincode::serde::decode_from_slice(
        include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_64/verifier_index.bin"),
        bincode::config::standard(),
    )
    .map(|(index, _)| index)
    .expect("small simple fixture verifier index should decode");

    assert_eq!(
        compute_verify_weight::<MockConfig>(&verifier_index),
        add_public_inputs(
            <() as WeightInfoVerifyProof>::verify_proof_domain_4096_pubs_0(),
            64
        )
    );
}

mod accept {
    use super::*;
    use poly_commitment::{
        ipa::{OpeningProof, SRS as IpaSrs},
        SRS as _,
    };
    use std::sync::Arc;

    fn fixture_is_accepted(proof_bytes: &[u8], verifier_index_bytes: &[u8], pubs: Pubs) {
        let raw_proof = proof_bytes.to_vec();
        let vk = Vk::<MockConfig>::new(verifier_index_bytes.to_vec(), KimchiProfileId::Vesta16);
        let proof = decode_proof(&raw_proof).expect("fixture proof should decode");
        let public_input = decode_public_input(&pubs).expect("fixture public input should decode");
        let mut verifier_index = decode_vk(&vk).expect("fixture verifier index should decode");
        prepare_verifier_index(&mut verifier_index, vk.profile)
            .expect("fixture verifier index should prepare");
        let mut rng = make_rng(&vk, &raw_proof, &pubs);

        verify_vesta16_proof_with_rng(&verifier_index, &proof, &public_input, &mut rng)
            .expect("fixture proof should verify");
    }

    fn fixture_pubs(bytes: &[u8]) -> Pubs {
        bytes
            .chunks_exact(PUB_SIZE)
            .map(|bytes| {
                bytes
                    .try_into()
                    .expect("fixture public input has field size")
            })
            .collect()
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
    fn small_max_feature_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!(
                "resources/generated_4096_maxpoly_4096_lookup_runtime_pubs_64/proof.bin"
            ),
            include_bytes!(
                "resources/generated_4096_maxpoly_4096_lookup_runtime_pubs_64/verifier_index.bin"
            ),
            fixture_pubs(include_bytes!(
                "resources/generated_4096_maxpoly_4096_lookup_runtime_pubs_64/pubs.bin"
            )),
        );
    }

    #[test]
    fn small_domain_with_fixed_vesta16_srs_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/generated_4096_lookup_runtime_pubs_64/proof.bin"),
            include_bytes!("resources/generated_4096_lookup_runtime_pubs_64/verifier_index.bin"),
            fixture_pubs(include_bytes!(
                "resources/generated_4096_lookup_runtime_pubs_64/pubs.bin"
            )),
        );
    }

    #[test]
    fn domain_65536_with_64_public_inputs_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/generated_65536_pubs_64/proof.bin"),
            include_bytes!("resources/generated_65536_pubs_64/verifier_index.bin"),
            fixture_pubs(include_bytes!("resources/generated_65536_pubs_64/pubs.bin")),
        );
    }

    #[test]
    fn one_chunk_lookup_runtime_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/generated_65536_lookup_runtime_pubs_64/proof.bin"),
            include_bytes!("resources/generated_65536_lookup_runtime_pubs_64/verifier_index.bin"),
            fixture_pubs(include_bytes!(
                "resources/generated_65536_lookup_runtime_pubs_64/pubs.bin"
            )),
        );
    }

    #[test]
    fn two_chunk_domain_131072_with_64_public_inputs_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/generated_131072_pubs_64/proof.bin"),
            include_bytes!("resources/generated_131072_pubs_64/verifier_index.bin"),
            fixture_pubs(include_bytes!(
                "resources/generated_131072_pubs_64/pubs.bin"
            )),
        );
    }

    #[test]
    fn two_chunk_lookup_runtime_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/generated_131072_lookup_runtime_pubs_64/proof.bin"),
            include_bytes!("resources/generated_131072_lookup_runtime_pubs_64/verifier_index.bin"),
            fixture_pubs(include_bytes!(
                "resources/generated_131072_lookup_runtime_pubs_64/pubs.bin"
            )),
        );
    }

    #[test]
    fn four_chunk_lookup_runtime_with_1024_public_inputs_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/proof.bin"),
            include_bytes!(
                "resources/generated_262144_lookup_runtime_pubs_1024/verifier_index.bin"
            ),
            fixture_pubs(include_bytes!(
                "resources/generated_262144_lookup_runtime_pubs_1024/pubs.bin"
            )),
        );
    }

    #[test]
    fn four_chunk_all_features_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/generated_262144_all_features_pubs_0/proof.bin"),
            include_bytes!("resources/generated_262144_all_features_pubs_0/verifier_index.bin"),
            fixture_pubs(include_bytes!(
                "resources/generated_262144_all_features_pubs_0/pubs.bin"
            )),
        );
    }

    #[test]
    fn production_envelope_accepts_four_chunk_all_features_1024_public_inputs() {
        let proof =
            include_bytes!("resources/generated_262144_all_features_pubs_1024/proof.bin").to_vec();
        let vk = Vk::<ProductionEnvelopeConfig>::new(
            include_bytes!("resources/generated_262144_all_features_pubs_1024/verifier_index.bin")
                .to_vec(),
            KimchiProfileId::Vesta16,
        );
        let pubs = fixture_pubs(include_bytes!(
            "resources/generated_262144_all_features_pubs_1024/pubs.bin"
        ));

        assert!(
            Kimchi::<ProductionEnvelopeConfig>::verify_proof(&vk, &proof, &pubs).is_ok(),
            "four-chunk all-features/1024-public-input fixture should verify under production caps"
        );
    }

    #[test]
    fn production_envelope_rejects_eight_chunk_verifier_index() {
        use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

        let mut verifier_index: Vesta16VerifierIndex = bincode::serde::decode_from_slice(
            include_bytes!(
                "resources/generated_262144_lookup_runtime_pubs_1024/verifier_index.bin"
            ),
            bincode::config::standard(),
        )
        .map(|(index, _)| index)
        .expect("four-chunk fixture verifier index should decode");
        verifier_index.domain =
            Radix2EvaluationDomain::new(1 << 19).expect("eight-chunk domain should exist");
        verifier_index.zk_rows =
            (kimchi::circuits::constraints::zk_rows_strict_lower_bound(8) + 1) as u64;

        let vk = Vk::<ProductionEnvelopeConfig>::new(
            bincode::serde::encode_to_vec(&verifier_index, bincode::config::standard())
                .expect("mutated verifier index should encode"),
            KimchiProfileId::Vesta16,
        );

        assert_err!(
            Kimchi::<ProductionEnvelopeConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn foreign_field_mul_fixture_is_accepted() {
        fixture_is_accepted(
            include_bytes!("resources/foreign_field_mul/proof.bin"),
            include_bytes!("resources/foreign_field_mul/verifier_index.bin"),
            Vec::new(),
        );
    }

    fn accelerated_opening_matches_upstream(raw_proof: Vec<u8>, raw_vk: Vec<u8>, raw_pubs: Pubs) {
        type UpstreamOpeningProof = OpeningProof<Vesta, FULL_ROUNDS>;
        type UpstreamProof = ProverProof<Vesta, UpstreamOpeningProof, FULL_ROUNDS>;
        type UpstreamVerifierIndex = VerifierIndex<FULL_ROUNDS, Vesta, IpaSrs<Vesta>>;

        let vk = Vk::<MockConfig>::new(raw_vk.clone(), KimchiProfileId::Vesta16);
        let proof: UpstreamProof =
            bincode::serde::decode_from_slice(&raw_proof, bincode::config::standard())
                .map(|(proof, _)| proof)
                .expect("fixture proof should decode with upstream opening proof");
        let mut verifier_index: UpstreamVerifierIndex =
            bincode::serde::decode_from_slice(&raw_vk, bincode::config::standard())
                .map(|(index, _)| index)
                .expect("fixture verifier index should decode with upstream SRS");
        verifier_index.srs = Arc::new(IpaSrs::create(verifier_index.max_poly_size));
        prepare_verifier_index_metadata(&mut verifier_index)
            .expect("upstream verifier index metadata should prepare");
        let public_input =
            decode_public_input(&raw_pubs).expect("fixture public inputs should decode");
        let mut rng = make_rng(&vk, &raw_proof, &raw_pubs);

        verify_with_rng::<
            FULL_ROUNDS,
            Vesta,
            DefaultFqSponge<VestaParameters, PlonkSpongeConstantsKimchi, FULL_ROUNDS>,
            DefaultFrSponge<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>,
            UpstreamOpeningProof,
            _,
        >(
            &vesta_group_map(),
            &verifier_index,
            &proof,
            &public_input,
            &mut rng,
        )
        .expect("upstream IPA verifier should accept accelerated fixture");
    }

    #[test]
    fn accelerated_opening_matches_upstream_for_domain_4096_fixture() {
        accelerated_opening_matches_upstream(
            include_bytes!("resources/generated_4096/proof.bin").to_vec(),
            include_bytes!("resources/generated_4096/verifier_index.bin").to_vec(),
            Vec::new(),
        );
    }

    #[test]
    fn accelerated_opening_matches_upstream_for_small_domain_fixed_vesta16_srs_fixture() {
        accelerated_opening_matches_upstream(
            include_bytes!("resources/generated_4096_lookup_runtime_pubs_64/proof.bin").to_vec(),
            include_bytes!("resources/generated_4096_lookup_runtime_pubs_64/verifier_index.bin")
                .to_vec(),
            fixture_pubs(include_bytes!(
                "resources/generated_4096_lookup_runtime_pubs_64/pubs.bin"
            )),
        );
    }

    #[test]
    fn accelerated_opening_matches_upstream_for_two_chunk_fixture() {
        accelerated_opening_matches_upstream(
            include_bytes!("resources/generated_131072_lookup_runtime_pubs_64/proof.bin").to_vec(),
            include_bytes!("resources/generated_131072_lookup_runtime_pubs_64/verifier_index.bin")
                .to_vec(),
            fixture_pubs(include_bytes!(
                "resources/generated_131072_lookup_runtime_pubs_64/pubs.bin"
            )),
        );
    }
}

mod reject {
    use super::*;
    use ark_ff::One;
    use kimchi::{
        circuits::lookup::{
            index::LookupSelectors,
            lookups::{LookupFeatures, LookupInfo, LookupPatterns},
        },
        proof::RecursionChallenge,
        verifier_index::LookupVerifierIndex,
    };
    use pallet_verifiers::traits::VerifyError;

    fn fixture_verifier_index() -> Vesta16VerifierIndex {
        bincode::serde::decode_from_slice(
            include_bytes!("resources/generated_4096/verifier_index.bin"),
            bincode::config::standard(),
        )
        .map(|(verifier_index, _)| verifier_index)
        .expect("fixture verifier index should decode")
    }

    fn fixture_verifier_index_from_bytes(bytes: &[u8]) -> Vesta16VerifierIndex {
        bincode::serde::decode_from_slice(bytes, bincode::config::standard())
            .map(|(verifier_index, _)| verifier_index)
            .expect("fixture verifier index should decode")
    }

    fn two_chunk_fixture_verifier_index() -> Vesta16VerifierIndex {
        fixture_verifier_index_from_bytes(include_bytes!(
            "resources/generated_131072_lookup_runtime_pubs_64/verifier_index.bin"
        ))
    }

    fn four_chunk_fixture_verifier_index() -> Vesta16VerifierIndex {
        fixture_verifier_index_from_bytes(include_bytes!(
            "resources/generated_262144_all_features_pubs_0/verifier_index.bin"
        ))
    }

    fn set_verifier_index_shape(
        verifier_index: &mut Vesta16VerifierIndex,
        max_poly_size: usize,
        domain_size: usize,
    ) {
        use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

        verifier_index.max_poly_size = max_poly_size;
        verifier_index.domain =
            Radix2EvaluationDomain::new(domain_size).expect("radix-2 domain should exist");
        let num_chunks = profile::expected_chunks(max_poly_size, domain_size)
            .expect("test shape should derive chunks");
        verifier_index.zk_rows =
            (kimchi::circuits::constraints::zk_rows_strict_lower_bound(num_chunks) + 1) as u64;
    }

    fn fixture_proof_and_index() -> (Proof, Vesta16Proof, Vesta16VerifierIndex) {
        let raw_proof = include_bytes!("resources/generated_4096/proof.bin").to_vec();
        let proof = decode_proof(&raw_proof).expect("fixture proof should decode");
        let mut verifier_index = fixture_verifier_index();
        prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16)
            .expect("fixture verifier index should prepare");

        (raw_proof, proof, verifier_index)
    }

    fn two_chunk_fixture_proof_and_index() -> (Vesta16Proof, Vesta16VerifierIndex) {
        let proof = decode_proof(include_bytes!(
            "resources/generated_131072_pubs_64/proof.bin"
        ))
        .expect("two-chunk fixture proof should decode");
        let vk = Vk::<MockConfig>::new(
            include_bytes!("resources/generated_131072_pubs_64/verifier_index.bin").to_vec(),
            KimchiProfileId::Vesta16,
        );
        let mut verifier_index = decode_vk(&vk).expect("two-chunk fixture index should decode");
        prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16)
            .expect("two-chunk fixture verifier index should prepare");

        (proof, verifier_index)
    }

    fn encode_proof(proof: &Vesta16Proof) -> Proof {
        bincode::serde::encode_to_vec(proof, bincode::config::standard())
            .expect("fixture proof should encode")
    }

    fn encode_verifier_index(verifier_index: &Vesta16VerifierIndex) -> Vec<u8> {
        bincode::serde::encode_to_vec(verifier_index, bincode::config::standard())
            .expect("fixture verifier index should encode")
    }

    fn lookup_features(patterns: LookupPatterns, uses_runtime_tables: bool) -> LookupFeatures {
        LookupFeatures {
            patterns,
            joint_lookup_used: patterns.joint_lookups_used(),
            uses_runtime_tables,
        }
    }

    #[test]
    fn oversized_verifier_index_is_rejected() {
        let vk = Vk::new(
            vec![0_u8; MockConfig::max_vk_size() as usize + 1],
            KimchiProfileId::Vesta16,
        );

        assert_err!(
            Kimchi::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn empty_verifier_material_is_rejected() {
        let vk = Vk::new(Vec::new(), KimchiProfileId::Vesta16);

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
    fn proof_with_trailing_bytes_is_rejected() {
        let mut proof = include_bytes!("resources/generated_4096/proof.bin").to_vec();
        proof.push(0);

        assert_err!(decode_proof(&proof), VerifyError::InvalidProofData);
    }

    #[test]
    fn verifier_index_with_trailing_bytes_is_rejected() {
        let mut bytes = include_bytes!("resources/generated_4096/verifier_index.bin").to_vec();
        bytes.push(0);
        let vk = Vk::<MockConfig>::new(bytes, KimchiProfileId::Vesta16);

        assert_eq!(
            decode_vk(&vk).err(),
            Some(VerifyError::InvalidVerificationKey)
        );
    }

    #[test]
    fn domain_requiring_more_than_four_chunks_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.max_poly_size = 512;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn domain_larger_than_work_envelope_is_rejected() {
        use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

        let mut verifier_index = fixture_verifier_index();
        verifier_index.domain = Radix2EvaluationDomain::new(1 << 18)
            .expect("radix-2 domain above the work envelope should exist");

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn smallest_zkfunction_domain_is_accepted() {
        use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

        let mut verifier_index = fixture_verifier_index();
        verifier_index.domain =
            Radix2EvaluationDomain::new(8).expect("small radix-2 domain should exist");

        assert!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16).is_ok(),
            "domain 2^3 is the smallest domain for a valid Kimchi circuit"
        );
    }

    #[test]
    fn domain_too_small_for_a_valid_kimchi_circuit_is_rejected() {
        use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

        let mut verifier_index = fixture_verifier_index();
        verifier_index.domain =
            Radix2EvaluationDomain::new(4).expect("small radix-2 domain should exist");

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn zero_max_poly_size_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.max_poly_size = 0;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn non_power_of_two_max_poly_size_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.max_poly_size = 4095;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn max_poly_size_larger_than_builtin_srs_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.max_poly_size = KimchiProfileId::Vesta16.max_poly_size() * 2;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn excessive_declared_public_inputs_are_rejected() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.public = KimchiProfileId::Vesta16.max_public_inputs() + 1;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn invalid_zk_rows_are_rejected_before_metadata_preparation() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.zk_rows = verifier_index.domain.size;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn recursive_verifier_index_is_rejected_by_vesta16_profile() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.prev_challenges = 1;

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn malformed_lookup_verifier_index_is_rejected_by_profile() {
        let mut verifier_index = fixture_verifier_index();
        let mut lookup_info = LookupInfo::create(LookupFeatures::default());
        lookup_info.max_per_row = usize::MAX;
        verifier_index.lookup_index = Some(LookupVerifierIndex {
            joint_lookup_used: false,
            lookup_table: Vec::new(),
            lookup_selectors: LookupSelectors {
                xor: None,
                lookup: None,
                range_check: None,
                ffmul: None,
            },
            table_ids: None,
            lookup_info,
            runtime_tables_selector: None,
        });

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn unexpected_extra_verifier_commitment_chunk_is_rejected() {
        let mut verifier_index = fixture_verifier_index();
        let extra_chunk = verifier_index.generic_comm.chunks[0];
        verifier_index.generic_comm.chunks.push(extra_chunk);

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn one_chunk_matching_smaller_max_poly_size_is_accepted_by_profile() {
        let mut verifier_index = fixture_verifier_index();
        set_verifier_index_shape(&mut verifier_index, 1 << 15, 1 << 15);

        assert!(
            profile::validate_verifier_index(KimchiProfileId::Vesta16, &verifier_index).is_ok()
        );
    }

    #[test]
    fn two_chunk_benchmarked_shape_is_accepted_by_profile() {
        let verifier_index = two_chunk_fixture_verifier_index();

        assert!(
            profile::validate_verifier_index(KimchiProfileId::Vesta16, &verifier_index).is_ok()
        );
    }

    #[test]
    fn four_chunk_benchmarked_shape_is_accepted_by_profile() {
        let verifier_index = four_chunk_fixture_verifier_index();

        assert!(
            profile::validate_verifier_index(KimchiProfileId::Vesta16, &verifier_index).is_ok()
        );
    }

    #[test]
    fn four_chunk_benchmarked_fixture_enables_all_optional_features() {
        let verifier_index = four_chunk_fixture_verifier_index();
        let lookup_index = verifier_index
            .lookup_index
            .as_ref()
            .expect("all-features fixture should include lookup index");

        assert_eq!(verifier_index.domain.size, 1 << 18);
        assert_eq!(verifier_index.max_poly_size, 1 << 16);
        assert_eq!(verifier_index.public, 0);
        assert!(verifier_index.range_check0_comm.is_some());
        assert!(verifier_index.range_check1_comm.is_some());
        assert!(verifier_index.foreign_field_add_comm.is_some());
        assert!(verifier_index.foreign_field_mul_comm.is_some());
        assert!(verifier_index.xor_comm.is_some());
        assert!(verifier_index.rot_comm.is_some());
        assert_eq!(
            lookup_index.lookup_info.features.patterns,
            LookupPatterns {
                xor: true,
                lookup: true,
                range_check: true,
                foreign_field_mul: true,
            }
        );
        assert!(lookup_index.lookup_selectors.xor.is_some());
        assert!(lookup_index.lookup_selectors.lookup.is_some());
        assert!(lookup_index.lookup_selectors.range_check.is_some());
        assert!(lookup_index.lookup_selectors.ffmul.is_some());
        assert!(lookup_index.runtime_tables_selector.is_some());
    }

    #[test]
    fn production_profile_rejects_unpriced_two_chunk_shape() {
        let mut verifier_index = two_chunk_fixture_verifier_index();
        set_verifier_index_shape(&mut verifier_index, 1 << 15, 1 << 16);

        assert!(
            profile::validate_verifier_index(KimchiProfileId::Vesta16, &verifier_index).is_err()
        );
    }

    #[test]
    fn production_profile_rejects_unpriced_four_chunk_shape() {
        let mut verifier_index = four_chunk_fixture_verifier_index();
        set_verifier_index_shape(&mut verifier_index, 1 << 15, 1 << 17);

        assert!(
            profile::validate_verifier_index(KimchiProfileId::Vesta16, &verifier_index).is_err()
        );
    }

    #[test]
    fn foreign_field_add_verifier_index_is_accepted_by_profile() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.foreign_field_add_comm = Some(verifier_index.generic_comm.clone());

        assert!(
            profile::validate_verifier_index(KimchiProfileId::Vesta16, &verifier_index).is_ok()
        );
    }

    #[test]
    fn lookup_verifier_index_with_all_patterns_and_runtime_tables_is_accepted_by_profile() {
        let mut verifier_index = fixture_verifier_index();
        let commitment = verifier_index.generic_comm.clone();
        verifier_index.xor_comm = Some(commitment.clone());
        verifier_index.range_check0_comm = Some(commitment.clone());
        verifier_index.foreign_field_mul_comm = Some(commitment.clone());

        let patterns = LookupPatterns {
            xor: true,
            lookup: true,
            range_check: true,
            foreign_field_mul: true,
        };
        let lookup_info = LookupInfo::create(lookup_features(patterns, true));
        verifier_index.lookup_index = Some(LookupVerifierIndex {
            joint_lookup_used: true,
            lookup_table: vec![commitment.clone(), commitment.clone(), commitment.clone()],
            lookup_selectors: LookupSelectors {
                xor: Some(commitment.clone()),
                lookup: Some(commitment.clone()),
                range_check: Some(commitment.clone()),
                ffmul: Some(commitment.clone()),
            },
            table_ids: Some(commitment.clone()),
            lookup_info,
            runtime_tables_selector: Some(commitment),
        });

        assert!(
            profile::validate_verifier_index(KimchiProfileId::Vesta16, &verifier_index).is_ok()
        );
    }

    #[test]
    fn lookup_gate_without_matching_lookup_index_is_rejected_by_profile() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.range_check0_comm = Some(verifier_index.generic_comm.clone());

        assert_err!(
            prepare_verifier_index(&mut verifier_index, KimchiProfileId::Vesta16),
            VerifyError::InvalidVerificationKey
        );
    }

    #[test]
    fn recursive_proof_is_rejected_before_verification() {
        let (_, mut proof, verifier_index) = fixture_proof_and_index();
        let recursion_commitment = proof.commitments.w_comm[0].clone();
        proof.prev_challenges.push(RecursionChallenge::new(
            vec![Fp::one(); 64],
            recursion_commitment,
        ));

        assert!(
            profile::validate_proof(KimchiProfileId::Vesta16, &proof, &verifier_index).is_err()
        );
    }

    #[test]
    fn unexpected_extra_proof_commitment_chunk_is_rejected_before_verification() {
        let (_, mut proof, verifier_index) = fixture_proof_and_index();
        let extra_chunk = proof.commitments.w_comm[0].chunks[0];
        proof.commitments.w_comm[0].chunks.push(extra_chunk);

        assert!(
            profile::validate_proof(KimchiProfileId::Vesta16, &proof, &verifier_index).is_err()
        );
    }

    #[test]
    fn two_chunk_proof_with_missing_commitment_chunk_is_rejected_before_verification() {
        let (mut proof, verifier_index) = two_chunk_fixture_proof_and_index();
        proof.commitments.w_comm[0].chunks.pop();

        assert!(
            profile::validate_proof(KimchiProfileId::Vesta16, &proof, &verifier_index).is_err()
        );
    }

    #[test]
    fn two_chunk_proof_with_missing_evaluation_chunk_is_rejected_before_verification() {
        let (mut proof, verifier_index) = two_chunk_fixture_proof_and_index();
        proof.evals.w[0].zeta.pop();

        assert!(
            profile::validate_proof(KimchiProfileId::Vesta16, &proof, &verifier_index).is_err()
        );
    }

    #[test]
    fn excessive_quotient_commitment_chunks_are_rejected_before_verification() {
        let (_, mut proof, verifier_index) = fixture_proof_and_index();
        let extra_chunk = proof.commitments.t_comm.chunks[0];
        while proof.commitments.t_comm.len() <= 7 {
            proof.commitments.t_comm.chunks.push(extra_chunk);
        }

        assert!(
            profile::validate_proof(KimchiProfileId::Vesta16, &proof, &verifier_index).is_err()
        );
    }

    #[test]
    fn proof_without_public_evaluations_is_accepted_for_zero_public_inputs() {
        let (_, mut proof, verifier_index) = fixture_proof_and_index();
        proof.evals.public = None;

        assert!(profile::validate_proof(KimchiProfileId::Vesta16, &proof, &verifier_index).is_ok());
    }

    #[test]
    fn modified_proof_encoding_remains_canonical() {
        let (_, mut proof, _) = fixture_proof_and_index();
        proof.proof.corrupt_z1_for_test();
        let encoded = encode_proof(&proof);

        assert_eq!(decode_proof(&encoded), Ok(proof));
    }

    #[test]
    fn modified_verifier_index_encoding_remains_canonical() {
        let mut verifier_index = fixture_verifier_index();
        verifier_index.zk_rows = 4;
        let encoded = encode_verifier_index(&verifier_index);
        let vk = Vk::<MockConfig>::new(encoded, KimchiProfileId::Vesta16);

        assert!(decode_vk(&vk).is_ok());
    }

    #[test]
    fn opening_with_too_few_rounds_is_rejected() {
        let (raw_proof, mut proof, verifier_index) = fixture_proof_and_index();
        let vk = Vk::<MockConfig>::new(
            include_bytes!("resources/generated_4096/verifier_index.bin").to_vec(),
            KimchiProfileId::Vesta16,
        );
        let raw_pubs = Vec::new();
        proof.proof.clear_rounds_for_test();
        let mut rng = make_rng(&vk, &raw_proof, &raw_pubs);

        assert_eq!(
            verify_vesta16_proof_with_rng(&verifier_index, &proof, &[], &mut rng),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn opening_with_too_many_rounds_is_rejected() {
        let (raw_proof, mut proof, verifier_index) = fixture_proof_and_index();
        let vk = Vk::<MockConfig>::new(
            include_bytes!("resources/generated_4096/verifier_index.bin").to_vec(),
            KimchiProfileId::Vesta16,
        );
        let raw_pubs = Vec::new();
        proof.proof.duplicate_round_for_test();
        let mut rng = make_rng(&vk, &raw_proof, &raw_pubs);

        assert_eq!(
            verify_vesta16_proof_with_rng(&verifier_index, &proof, &[], &mut rng),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn corrupted_opening_scalar_is_rejected() {
        let (raw_proof, mut proof, verifier_index) = fixture_proof_and_index();
        let vk = Vk::<MockConfig>::new(
            include_bytes!("resources/generated_4096/verifier_index.bin").to_vec(),
            KimchiProfileId::Vesta16,
        );
        let raw_pubs = Vec::new();
        proof.proof.corrupt_z1_for_test();
        let mut rng = make_rng(&vk, &raw_proof, &raw_pubs);

        assert_eq!(
            verify_vesta16_proof_with_rng(&verifier_index, &proof, &[], &mut rng),
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
    fn production_envelope_rejects_1025_public_inputs_early() {
        let proof = include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/proof.bin")
            .to_vec();
        let vk = Vk::<ProductionEnvelopeConfig>::new(
            include_bytes!(
                "resources/generated_262144_lookup_runtime_pubs_1024/verifier_index.bin"
            )
            .to_vec(),
            KimchiProfileId::Vesta16,
        );
        let pubs = vec![[0_u8; PUB_SIZE]; ProductionEnvelopeConfig::max_pubs() as usize + 1];

        assert_err!(
            Kimchi::<ProductionEnvelopeConfig>::verify_proof(&vk, &proof, &pubs),
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
