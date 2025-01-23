#![cfg(test)]

use std::io::{BufReader, Read};

use frame_support::assert_ok;
use rstest::*;
use sp_core::ConstU32;
use hp_verifiers::Verifier;

use super::*;

struct ConfigTest;

impl Config for ConfigTest {
    type FixedMax = ConstU32<50>;

    type ColumnsMax = ConstU32<100>;

    type PermutationMax = ConstU32<100>;
    
    type SelectorMax = ConstU32<100>;
    
    type LargestK = ConstU32<20>;
    
    type ChallengesMax = ConstU32<100>;
    
    type QueriesMax = ConstU32<100>;
    
    type ExpressionDegreeMax = ConstU32<100>;
    
    type ExpressionVarsMax = ConstU32<100>;
    
    type GatesMax = ConstU32<100>;
    
    type LookupsMax = ConstU32<100>;
}

pub struct TestData {
    pub vk: Vec<u8>,
    pub proof: Vec<u8>,
    pub pubs: Vec<U256>,
}

#[fixture]
pub fn valid_test_data() -> TestData {
    let pubs_bytes = include_bytes!("resources/VALID_PUBS.bin").to_vec();
    let mut pubs = vec![];

   // using reader
   let mut reader = BufReader::new(pubs_bytes.as_slice());
   let mut buffer = [0u8; 32];
   while reader.read(&mut buffer).unwrap() > 0 {
       let fr = U256::from_little_endian(&buffer);
       pubs.push(fr);
   }
    
    TestData {
        vk: include_bytes!("resources/VALID_VK.bin").to_vec(),
        proof: include_bytes!("resources/VALID_PROOF.bin").to_vec(),
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

    // #[rstest]
    // fn too_big_vk(valid_test_data: TestData) {
    //     assert_err!(
    //         Halo2::<ConfigWithMaxNuEqualTo3>::verify_proof(
    //             &valid_test_data.vk.into(),
    //             &valid_test_data.proof,
    //             &valid_test_data.pubs
    //         ),
    //         VerifyError::InvalidVerificationKey
    //     )
    // }

    // #[rstest]
    // fn too_big_proof(valid_test_data: TestData) {
    //     let proof = vec![1u8; crate::MAX_PROOF_SIZE as usize + 1];
    //     assert_err!(
    //         Halo2::<ConfigTest>::verify_proof(
    //             &valid_test_data.vk.into(),
    //             &proof.into(),
    //             &valid_test_data.pubs
    //         ),
    //         VerifyError::InvalidProofData
    //     )
    // }

    // #[rstest]
    // fn too_big_pubs(valid_test_data: TestData) {
    //     let pubs = vec![1u8; crate::MAX_PUBS_SIZE as usize + 1];
    //     assert_err!(
    //         Halo2::<ConfigTest>::verify_proof(
    //             &valid_test_data.vk.into(),
    //             &valid_test_data.proof,
    //             &pubs.into()
    //         ),
    //         VerifyError::InvalidInput
    //     )
    // }
}
