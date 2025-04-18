#![cfg(test)]

use super::*;
use frame_support::assert_ok;
use rstest::*;

include!("resources.rs");

#[fixture]
fn valid_test_data() -> TestData<MockConfig> {
    get_valid_test_data()
}

#[fixture]
fn valid_compressed_poseidon_test_data() -> TestData<MockConfig> {
    get_valid_compressed_poseidon_test_data()
}

#[fixture]
fn valid_uncompressed_poseidon_test_data() -> TestData<MockConfig> {
    get_valid_uncompressed_poseidon_test_data()
}

#[fixture]
fn valid_compressed_keccak_test_data() -> TestData<MockConfig> {
    get_valid_compressed_keccak_test_data()
}

#[fixture]
fn valid_uncompressed_keccak_test_data() -> TestData<MockConfig> {
    get_valid_uncompressed_keccak_test_data()
}

#[rstest]
fn verify_valid_proof(valid_test_data: TestData<MockConfig>) {
    assert_ok!(Plonky2::<MockConfig>::verify_proof(
        &valid_test_data.vk,
        &valid_test_data.proof,
        &valid_test_data.pubs
    ));
}

#[rstest]
#[case(valid_compressed_poseidon_test_data())]
#[case(valid_uncompressed_poseidon_test_data())]
#[case(valid_compressed_keccak_test_data())]
#[case(valid_uncompressed_keccak_test_data())]
fn verify_valid_proof_cases(#[case] test_data: TestData<MockConfig>) {
    assert_ok!(Plonky2::<MockConfig>::verify_proof(
        &test_data.vk,
        &test_data.proof,
        &test_data.pubs
    ));
}

#[rstest]
fn compute_correct_weight_for_proof(valid_compressed_poseidon_test_data: TestData<MockConfig>) {
    let expected = <() as crate::WeightInfoVerifyProof>::verify_proof_poseidon_compressed_19();

    assert_eq!(
        Some(expected),
        Plonky2::<MockConfig>::verify_proof(
            &valid_compressed_poseidon_test_data.vk,
            &valid_compressed_poseidon_test_data.proof,
            &valid_compressed_poseidon_test_data.pubs
        )
        .unwrap()
    );
}

mod reject {
    use frame_support::assert_err;
    use hp_verifiers::VerifyError;

    use super::*;

    #[rstest]
    fn should_fail_on_invalid_pubs(valid_test_data: TestData<MockConfig>) {
        let TestData {
            vk,
            proof,
            mut pubs,
        } = valid_test_data;

        pubs[0] = pubs.first().unwrap().wrapping_add(1);

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::VerifyError
        );
    }

    #[rstest]
    fn should_not_verify_false_proof(valid_test_data: TestData<MockConfig>) {
        let TestData {
            vk,
            mut proof,
            pubs,
        } = valid_test_data;

        let len = proof.bytes.len();
        proof.bytes[len - 1] = pubs.last().unwrap().wrapping_add(1);

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::VerifyError
        );
    }

    #[rstest]
    fn should_not_validate_corrupted_vk(valid_test_data: TestData<MockConfig>) {
        let mut vk = valid_test_data.vk.clone();

        if let Some(last_byte) = vk.bytes.last_mut() {
            *last_byte = last_byte.wrapping_add(1);
        }

        assert_err!(
            Plonky2::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[rstest]
    fn should_not_verify_oversized_vk(valid_test_data: TestData<MockConfig>) {
        let TestData {
            mut vk,
            proof,
            pubs,
        } = valid_test_data;

        vk.bytes = vec![0u8; MockConfig::max_vk_size() as usize + 1];

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::InvalidVerificationKey
        );
    }

    #[rstest]
    fn should_not_validate_vk_with_too_large_degree_bits(valid_test_data: TestData<MockConfig>) {
        let TestData {
            mut vk,
            proof: _,
            pubs: _,
        } = valid_test_data;

        // Set the byte controlling degree_bits to a very high value
        vk.bytes[732] = u8::MAX;

        assert_err!(
            Plonky2::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[rstest]
    fn should_not_verify_oversized_proof(valid_test_data: TestData<MockConfig>) {
        let TestData {
            vk,
            mut proof,
            pubs,
        } = valid_test_data;

        proof.bytes = vec![0u8; MockConfig::max_proof_size() as usize + 1];

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::InvalidProofData
        );
    }

    #[rstest]
    fn should_not_verify_oversized_pubs(valid_test_data: TestData<MockConfig>) {
        let TestData { vk, proof, .. } = valid_test_data;

        let pubs = vec![0u8; MockConfig::max_pubs_size() as usize + 1];

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::InvalidInput
        );
    }
}
