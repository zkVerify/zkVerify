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
    type MaxNSegment = ConstU32<4>;
    type Segment20MaxSize = ConstU32<350_000>;
    type MaxPubsSize = ConstU32<100>;
    type WeightInfo = ();
}
include!("resources.rs");

#[rstest]
#[case(&v2_1::VK, Proof::V2_1(v2_1::PROOF_SUCCINCT.to_vec()), &v2_1::PUBS_SUCCINCT)]
#[case(&v2_1::VK, Proof::V2_1(v2_1::PROOF_POSEIDON2_16.to_vec()), &v2_1::PUBS_16)]
#[case(&v2_1::VK, Proof::V2_1(v2_1::PROOF_POSEIDON2_22.to_vec()), &v2_1::PUBS_22)]
#[case(&v2_2::VK, Proof::V2_2(v2_2::PROOF_SUCCINCT.to_vec()), &v2_2::PUBS_SUCCINCT)]
#[case(&v2_2::VK, Proof::V2_2(v2_2::PROOF_POSEIDON2_16.to_vec()), &v2_2::PUBS_16)]
#[case(&v2_2::VK, Proof::V2_2(v2_2::PROOF_POSEIDON2_22.to_vec()), &v2_2::PUBS_22)]
#[case::accept_and_verify_the_upper_bound_proof(&v2_1::VK, Proof::V2_1(v2_1::PROOF_UPPER_BOUND.to_vec()), &v2_1::PUBS_UPPER_BOUND
)]
fn verify_valid_proof(#[case] vk: &Vk, #[case] proof: Proof, #[case] pubs: &[u8]) {
    assert!(Risc0::<Mock>::verify_proof(vk, &proof, &pubs.to_vec()).is_ok());
}

#[rstest]
#[case(Proof::V1_0(any_mock::PROOF.to_vec()))]
#[case(Proof::V1_1(any_mock::PROOF.to_vec()))]
#[case(Proof::V1_2(any_mock::PROOF.to_vec()))]
#[case(Proof::V2_0(any_mock::PROOF.to_vec()))]
fn reject_unsupported_proof_version(#[case] proof: Proof) {
    assert_eq!(
        Risc0::<Mock>::verify_proof(&any_mock::VK, &proof, &any_mock::PUBS.to_vec()),
        Err(VerifyError::UnsupportedVersion)
    );
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
#[case::v2_0(
    Proof::V2_0(Default::default()),
    H256::from(sp_io::hashing::sha2_256(b"risc0:v2.0"))
)]
#[case::v2_1(
    Proof::V2_1(Default::default()),
    H256::from(sp_io::hashing::sha2_256(b"risc0:v2.1"))
)]
#[case::v2_2(
    Proof::V2_2(Default::default()),
    H256::from(sp_io::hashing::sha2_256(b"risc0:v2.2"))
)]
#[case::do_not_depend_on_proof_content(
    Proof::V2_1([0xde;16].to_vec()),
    H256::from(sp_io::hashing::sha2_256(b"risc0:v2.1"))
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
        let mut invalid_pubs = v2_1::PUBS.to_vec();
        invalid_pubs[0] = invalid_pubs[0].wrapping_add(1);
        let proof = Proof::V2_1(v2_1::VALID_PROOF.to_vec());
        assert_eq!(
            Risc0::<Mock>::verify_proof(&v2_1::VK, &proof, &invalid_pubs),
            Err(VerifyError::VerifyError)
        )
    }

    #[test]
    fn undeserializable_proof() {
        let mut malformed_proof = v2_1::VALID_PROOF.to_vec();
        malformed_proof[0] = malformed_proof[0].wrapping_add(1);
        let proof = Proof::V2_1(malformed_proof);
        assert_eq!(
            Risc0::<Mock>::verify_proof(&v2_1::VK, &proof, &v2_1::PUBS.to_vec()),
            Err(VerifyError::InvalidProofData)
        )
    }

    #[test]
    fn too_big_proof() {
        let too_big_proof = vec![0; Mock::max_proof_size() as usize + 1];
        let proof = Proof::V2_1(too_big_proof);
        assert_eq!(
            Risc0::<Mock>::verify_proof(&v2_1::VK, &proof, &v2_1::PUBS.to_vec()),
            Err(VerifyError::InvalidProofData)
        )
    }

    #[test]
    fn too_big_pubs() {
        let too_big_pubs = vec![0; Mock::max_pubs_size() as usize + 1];
        let proof = Proof::V2_1(v2_1::VALID_PROOF.to_vec());
        assert_eq!(
            Risc0::<Mock>::verify_proof(&v2_1::VK, &proof, &too_big_pubs),
            Err(VerifyError::InvalidInput)
        )
    }

    #[test]
    fn too_weight_proof() {
        let vk = v2_1::VK;
        let proof = Proof::V2_1(v2_1::PROOF_COMPOSITE_3_SLOTS.to_vec());
        let pubs = v2_1::PUBS_COMPOSITE_3_SLOTS.to_vec();
        assert!(Risc0::<Mock>::verify_proof(&vk, &proof, &pubs).is_ok());

        struct PassSizeButNotWeight;
        impl Config for PassSizeButNotWeight {
            type MaxNSegment = ConstU32<2>;
            type Segment20MaxSize = ConstU32<1_000_000>;
            type MaxPubsSize = ConstU32<100>;
            type WeightInfo = ();
        }

        assert_eq!(
            Risc0::<PassSizeButNotWeight>::verify_proof(
                &vk,
                &Proof::V2_1(v2_1::PROOF_COMPOSITE_3_SLOTS.to_vec()),
                &pubs
            ),
            Err(VerifyError::InvalidProofData)
        )
    }
}

#[test]
fn compute_correct_weight_for_composite_proof() {
    let vk = v2_1::VK;
    let proof = Proof::V2_1(v2_1::PROOF_COMPOSITE_3_SLOTS.to_vec());
    let pubs = v2_1::PUBS_COMPOSITE_3_SLOTS.to_vec();

    // This proof is composed by two 2^20 segments and one 2^17 segment
    let expected = Weight::from_parts(
        <() as crate::WeightInfoVerifyProof>::verify_proof_segment_poseidon2_20().ref_time() * 2
            + <() as crate::WeightInfoVerifyProof>::verify_proof_segment_poseidon2_17().ref_time(),
        0,
    );

    assert_eq!(
        Some(expected),
        Risc0::<Mock>::verify_proof(&vk, &proof, &pubs).unwrap()
    );
}

#[test]
fn compute_correct_weight_for_succinct_proof() {
    let vk = v2_1::VK;
    let proof = Proof::V2_1(v2_1::PROOF_SUCCINCT.to_vec());
    let pubs = v2_1::PUBS_SUCCINCT.to_vec();

    // This proof is composed by two 2^20 segments and one 2^17 segment
    let expected = Weight::from_parts(
        <() as crate::WeightInfoVerifyProof>::verify_proof_succinct().ref_time(),
        0,
    );

    assert_eq!(
        Some(expected),
        Risc0::<Mock>::verify_proof(&vk, &proof, &pubs).unwrap()
    );
}

#[test]
fn segment_weight_return_error_if_unsupported_size_or_hash() {
    assert!(Risc0::<Mock>::segment_weight(SegmentInfo::new("unknown".to_owned(), 20)).is_err());
    assert!(Risc0::<Mock>::segment_weight(SegmentInfo::new("sha-256".to_owned(), 20)).is_err());
    assert!(Risc0::<Mock>::segment_weight(SegmentInfo::new("poseidon2".to_owned(), 23)).is_err());

    //Sanity check
    Risc0::<Mock>::segment_weight(SegmentInfo::new("poseidon2".to_owned(), 21)).unwrap();
}

mod weight {
    use super::*;

    #[rstest]
    fn verify_proof_should_return_the_same_value_from_conf_for_all_proof_and_pubs(
        #[values(400_000, 1_000_000)] proof_size: usize,
        #[values(0, 1_000)] pubs_size: usize,
    ) {
        let proof = (0..proof_size).map(|_| 11).collect::<Vec<_>>();
        let pubs = (0..pubs_size).map(|_| 22).collect::<Vec<_>>();
        assert_eq!(
            <Risc0Weight::<()> as pallet_verifiers::WeightInfo<Risc0<Mock>>>::verify_proof(
                &Proof::V2_1(proof),
                &pubs.to_vec()
            ),
            Mock::max_verify_proof_weight()
        );
    }
}
