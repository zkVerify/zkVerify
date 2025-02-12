#![cfg(test)]

use std::io::{BufReader, Read};

use frame_support::assert_ok;
use hp_verifiers::Verifier;
use rstest::*;
use sp_core::ConstU32;

use super::*;

struct ConfigTest;

impl Config for ConfigTest {
    type VkMaxBytes = ConstU32<750000000>;
    // most proofs would be capped at 4096 bytes, we double it to be safe
    type ProofMaxBytes = ConstU32<8192>;
}

struct ConfigTestSmall;

impl Config for ConfigTestSmall {
    type VkMaxBytes = ConstU32<100>;
    type ProofMaxBytes = ConstU32<1024>;
}

pub struct TestData {
    pub vk: Vec<u8>,
    pub proof: Vec<u8>,
    pub pubs: Vec<U256>,
}

#[fixture]
pub fn valid_test_data() -> TestData {
    let pubs_bytes = include_bytes!("resources/VALID_PUBS_21.bin").to_vec();
    let mut pubs = vec![];

    // using reader
    let mut reader = BufReader::new(pubs_bytes.as_slice());
    let mut buffer = [0u8; 32];
    while reader.read(&mut buffer).unwrap() > 0 {
        let fr = U256::from_little_endian(&buffer);
        pubs.push(fr);
    }

    TestData {
        vk: include_bytes!("resources/VALID_VK_21.bin").to_vec(),
        proof: include_bytes!("resources/VALID_PROOF_21.bin").to_vec(),
        pubs,
    }
}

#[rstest]
fn verify_valid_proof(valid_test_data: TestData) {
    assert_ok!(Halo2::<ConfigTest>::verify_proof(
        &valid_test_data.vk.into(),
        &valid_test_data.proof,
        &valid_test_data.pubs
    ));
}

mod reject {
    use frame_support::assert_err;
    use hp_verifiers::VerifyError;

    use super::*;

    #[rstest]
    fn invalid_proof(valid_test_data: TestData) {
        let mut invalid_pubs = valid_test_data.pubs.clone();
        let pubs_len = invalid_pubs.len();
        invalid_pubs[pubs_len - 1] += U256::one();

        assert_err!(
            Halo2::<ConfigTest>::verify_proof(
                &valid_test_data.vk.into(),
                &valid_test_data.proof,
                &invalid_pubs,
            ),
            VerifyError::VerifyError
        )
    }

    #[rstest]
    fn undeserializable_proof(valid_test_data: TestData) {
        let mut malformed_proof = valid_test_data.proof.clone();
        malformed_proof[0] = malformed_proof[0].wrapping_add(1);

        assert_err!(
            Halo2::<ConfigTest>::verify_proof(
                &valid_test_data.vk.into(),
                &malformed_proof,
                &valid_test_data.pubs,
            ),
            VerifyError::VerifyError
        )
    }

    #[rstest]
    fn undeserializable_vk(valid_test_data: TestData) {
        let mut malformed_vk = valid_test_data.vk.clone();
        malformed_vk[10] = malformed_vk[0].wrapping_add(1);

        assert_err!(
            Halo2::<ConfigTest>::verify_proof(
                &malformed_vk.into(),
                &valid_test_data.proof,
                &valid_test_data.pubs,
            ),
            VerifyError::InvalidVerificationKey
        )
    }

    #[rstest]
    fn too_big_vk(valid_test_data: TestData) {
        assert_err!(
            Halo2::<ConfigTestSmall>::validate_vk(&valid_test_data.vk.into()),
            VerifyError::InvalidVerificationKey
        )
    }

    #[rstest]
    fn too_big_proof(valid_test_data: TestData) {
        let proof = vec![1u8; ConfigTestSmall::max_proof_bytes() + 1];
        assert_err!(
            Halo2::<ConfigTestSmall>::verify_proof(
                &valid_test_data.vk.into(),
                &proof.into(),
                &valid_test_data.pubs
            ),
            VerifyError::InvalidProofData
        )
    }
}
