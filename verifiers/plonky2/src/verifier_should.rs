#![cfg(test)]

use super::*;
use frame_support::assert_ok;
use rstest::*;

include!("resources.rs");

#[fixture]
fn worst_case_test_data() -> TestData<MockConfig> {
    // get_parameterized_test_data(MAX_DEGREE_BITS, crate::vk::Plonky2Config::Poseidon, true)
    get_parameterized_test_data(MAX_DEGREE_BITS, crate::vk::Plonky2Config::Poseidon)
}

#[rstest]
fn verify_valid_proof(worst_case_test_data: TestData<MockConfig>) {
    assert_ok!(Plonky2::<MockConfig>::verify_proof(
        &worst_case_test_data.vk,
        &worst_case_test_data.proof,
        &worst_case_test_data.pubs
    ));
}

#[rstest]
// #[case(MAX_DEGREE_BITS, Plonky2Config::Keccak, true)]
#[case(MAX_DEGREE_BITS, Plonky2Config::Keccak)]
// #[case(MAX_DEGREE_BITS, Plonky2Config::Poseidon, true)]
#[case(MAX_DEGREE_BITS, Plonky2Config::Poseidon)]
fn test_parameterized_test_data(
    #[case] deg: usize,
    #[case] config: Plonky2Config,
    // #[case] compress: bool,
) {
    // let test_data = get_parameterized_test_data(deg, config, compress);
    let test_data = get_parameterized_test_data(deg, config);
    assert_ok!(Plonky2::<MockConfig>::verify_proof(
        &test_data.vk,
        &test_data.proof,
        &test_data.pubs
    ));
}

#[rstest]
fn compute_correct_weight_for_proof() {
    // let test_data = get_parameterized_test_data(19, Plonky2Config::Poseidon, true);
    let test_data = get_parameterized_test_data(19, Plonky2Config::Poseidon);
    let expected = <() as crate::WeightInfoVerifyProof>::verify_proof_poseidon_uncompressed_19();

    assert_eq!(
        Some(expected),
        Plonky2::<MockConfig>::verify_proof(&test_data.vk, &test_data.proof, &test_data.pubs)
            .unwrap()
    );
}

mod reject {
    use frame_support::assert_err;
    use hp_verifiers::VerifyError;

    use super::*;

    #[rstest]
    fn should_fail_on_invalid_pubs(worst_case_test_data: TestData<MockConfig>) {
        let TestData {
            vk,
            proof,
            mut pubs,
        } = worst_case_test_data;

        let n = pubs.len();
        pubs[n - 1] = pubs.last().unwrap().wrapping_add(1);

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::VerifyError
        );
    }

    #[rstest]
    fn should_not_verify_false_proof(worst_case_test_data: TestData<MockConfig>) {
        let TestData {
            vk,
            mut proof,
            pubs,
        } = worst_case_test_data;

        let len = proof.bytes.len();
        proof.bytes[len - 1] = pubs.last().unwrap().wrapping_add(1);

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::VerifyError
        );
    }

    #[rstest]
    fn should_not_validate_corrupted_vk(worst_case_test_data: TestData<MockConfig>) {
        let mut vk = worst_case_test_data.vk.clone();

        if let Some(last_byte) = vk.bytes.last_mut() {
            *last_byte = last_byte.wrapping_add(1);
        }

        assert_err!(
            Plonky2::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[rstest]
    fn should_not_verify_oversized_vk(worst_case_test_data: TestData<MockConfig>) {
        let TestData {
            mut vk,
            proof,
            pubs,
        } = worst_case_test_data;

        vk.bytes = vec![0u8; MockConfig::max_vk_size() as usize + 1];

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::InvalidVerificationKey
        );
    }

    #[rstest]
    fn should_not_validate_vk_with_too_large_degree_bits(
        worst_case_test_data: TestData<MockConfig>,
    ) {
        let TestData {
            mut vk,
            proof: _,
            pubs: _,
        } = worst_case_test_data;

        // Set the byte controlling degree_bits to a value beyond what is acceptable
        vk.bytes[732] = u8::try_from(MAX_DEGREE_BITS)
            .expect("MAX_DEGREE_BITS should always be convertible to u8")
            + 1;

        assert_err!(
            Plonky2::<MockConfig>::validate_vk(&vk),
            VerifyError::InvalidVerificationKey
        );
    }

    #[rstest]
    fn should_not_verify_oversized_proof(worst_case_test_data: TestData<MockConfig>) {
        let TestData {
            vk,
            mut proof,
            pubs,
        } = worst_case_test_data;

        proof.bytes = vec![0u8; MockConfig::max_proof_size() as usize + 1];

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::InvalidProofData
        );
    }

    #[rstest]
    fn should_not_verify_oversized_pubs(worst_case_test_data: TestData<MockConfig>) {
        let TestData { vk, proof, .. } = worst_case_test_data;

        let pubs = vec![0u8; MockConfig::max_pubs_size() as usize + 1];

        assert_err!(
            Plonky2::<MockConfig>::verify_proof(&vk, &proof, &pubs),
            VerifyError::InvalidInput
        );
    }
}
