// Copyright 2025, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg(test)]

use crate::resources::{get_parameterized_test_data, TestData, TestParams};

use super::*;
use hex_literal::hex;
// use pallet_verifiers::mock::Test;

struct MockRuntime;

impl crate::Config for MockRuntime {
    type MaxPubs = sp_core::ConstU32<32>;
    type WeightInfo = ();
}

// Default (proof, vk, pubs) triplets for unit testing:

#[allow(dead_code)]
pub(crate) fn valid_zk_test_data() -> TestData {
    let test_params = TestParams::new(25, ProofType::ZK, ProtocolVersion::V3_0);
    get_parameterized_test_data(test_params).expect("zk test data should be present")
}

#[allow(dead_code)]
pub(crate) fn valid_plain_test_data() -> TestData {
    let test_params = TestParams::new(25, ProofType::Plain, ProtocolVersion::V3_0);
    get_parameterized_test_data(test_params).expect("zk test data should be present")
}

#[test]
fn verify_valid_zk_proof() {
    let TestData {
        versioned_vk,
        versioned_proof,
        pubs,
    } = valid_zk_test_data();

    assert!(Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs).is_ok());
}

#[test]
fn verify_valid_plain_proof() {
    let TestData {
        versioned_vk,
        versioned_proof,
        pubs,
    } = valid_plain_test_data();

    assert!(Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs).is_ok());
}

#[test]
fn verify_vk_hash() {
    let TestData { versioned_vk, .. } = valid_plain_test_data();

    let vk_hash = Ultrahonk::<MockRuntime>::vk_hash(&versioned_vk);

    assert_eq!(
        vk_hash.as_bytes(),
        hex!("db7e16ea447c0514657eda8c51d3d1599e26a86bfe4608b67070b18f5b4cde96")
    );
}

mod reject {
    use super::*;

    #[test]
    fn invalid_public_values() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_zk_test_data();

        let mut invalid_pubs = pubs;
        invalid_pubs[0][0] = 0x10;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &invalid_pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn zk_proof_with_32_public_inputs_where_one_is_invalid() {
        let TestData {
            versioned_vk,
            versioned_proof,
            mut pubs,
        } = valid_zk_test_data();

        // Render first public input invalid
        pubs[0][0] += 1;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn plain_proof_with_32_public_inputs_where_one_is_invalid() {
        let TestData {
            versioned_vk,
            versioned_proof,
            mut pubs,
        } = valid_plain_test_data();

        // Render first public input invalid
        pubs[0][0] += 1;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn if_provided_too_many_public_inputs_for_zk_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            mut pubs,
        } = valid_zk_test_data();

        // Keep adding bogus public inputs until the limit is exceeded
        while (pubs.len() as u32) <= <MockRuntime as Config>::MaxPubs::get() {
            pubs.push([0u8; PUB_SIZE]);
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn if_provided_too_many_public_inputs_for_plain_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            mut pubs,
        } = valid_plain_test_data();

        // Keep adding bogus public inputs until the limit is exceeded
        while (pubs.len() as u32) <= <MockRuntime as Config>::MaxPubs::get() {
            pubs.push([0u8; PUB_SIZE]);
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn invalid_number_of_public_inputs() {
        let TestData {
            versioned_vk,
            versioned_proof,
            mut pubs,
        } = valid_plain_test_data();

        if pubs.is_empty() {
            // add one bogus public input
            pubs.push([0u8; PUB_SIZE]);
        } else {
            // drop one public input
            pubs.pop();
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn invalid_zk_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_zk_test_data();

        let invalid_versioned_proof = match versioned_proof {
            VersionedProof::V3_0(Proof::ZK(mut proof_bytes)) => {
                let proof_bytes_len = proof_bytes.len();
                proof_bytes[proof_bytes_len - 1] = 0x00;
                VersionedProof::V3_0(Proof::new(ProofType::ZK, proof_bytes))
            }
            VersionedProof::V3_0(Proof::Plain(_)) => unreachable!(),
        };

        assert!(matches!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_versioned_proof, &pubs),
            Err(VerifyError::VerifyError) | Err(VerifyError::InvalidProofData)
        ));
    }

    #[test]
    fn invalid_plain_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_plain_test_data();

        let invalid_versioned_proof = match versioned_proof {
            VersionedProof::V3_0(Proof::Plain(mut proof_bytes)) => {
                let proof_bytes_len = proof_bytes.len();
                proof_bytes[proof_bytes_len - 1] = 0x00;
                VersionedProof::V3_0(Proof::new(ProofType::ZK, proof_bytes))
            }
            VersionedProof::V3_0(Proof::ZK(_)) => unreachable!(),
        };

        assert!(matches!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_versioned_proof, &pubs),
            Err(VerifyError::VerifyError) | Err(VerifyError::InvalidProofData)
        ));
    }

    #[test]
    fn big_zk_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_zk_test_data();

        let invalid_versioned_proof = match versioned_proof {
            VersionedProof::V3_0(Proof::ZK(mut proof_bytes)) => {
                proof_bytes.push(0x00);
                VersionedProof::V3_0(Proof::new(ProofType::ZK, proof_bytes))
            }
            VersionedProof::V3_0(Proof::Plain(_)) => unreachable!(),
        };

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_versioned_proof, &pubs),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn big_plain_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_plain_test_data();

        let invalid_versioned_proof = match versioned_proof {
            VersionedProof::V3_0(Proof::Plain(mut proof_bytes)) => {
                proof_bytes.push(0x00);
                VersionedProof::V3_0(Proof::new(ProofType::Plain, proof_bytes))
            }
            VersionedProof::V3_0(Proof::ZK(_)) => unreachable!(),
        };

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_versioned_proof, &pubs),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn small_zk_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_zk_test_data();

        let invalid_versioned_proof = match versioned_proof {
            VersionedProof::V3_0(Proof::ZK(mut proof_bytes)) => {
                proof_bytes.pop();
                VersionedProof::V3_0(Proof::new(ProofType::ZK, proof_bytes))
            }
            VersionedProof::V3_0(Proof::Plain(_)) => unreachable!(),
        };

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_versioned_proof, &pubs),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn small_plain_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_plain_test_data();

        let invalid_versioned_proof = match versioned_proof {
            VersionedProof::V3_0(Proof::Plain(mut proof_bytes)) => {
                proof_bytes.pop();
                VersionedProof::V3_0(Proof::new(ProofType::Plain, proof_bytes))
            }
            VersionedProof::V3_0(Proof::ZK(_)) => unreachable!(),
        };

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_versioned_proof, &pubs),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn invalid_vk() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_zk_test_data();

        let invalid_versioned_vk = match versioned_vk {
            VersionedVk::V3_0(mut vk_bytes) => {
                vk_bytes[0] = 0x10;
                VersionedVk::V3_0(vk_bytes)
            }
            _ => unreachable!(),
        };

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&invalid_versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::InvalidVerificationKey)
        );
    }

    // TODO: Once a second version variant is introduced to the enum
    // #[test]
    // fn reject_proof_with_version_not_matching_the_vk_version() { }

    #[test]
    fn reject_malformed_zk_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_zk_test_data();

        let invalid_versioned_proof = match versioned_proof {
            VersionedProof::V3_0(Proof::ZK(mut proof_bytes)) => {
                proof_bytes[0] = 0x07;
                VersionedProof::V3_0(Proof::new(ProofType::ZK, proof_bytes))
            }
            VersionedProof::V3_0(Proof::Plain(_)) => unreachable!(),
        };

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_versioned_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn reject_malformed_plain_proof() {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = valid_plain_test_data();

        let invalid_versioned_proof = match versioned_proof {
            VersionedProof::V3_0(Proof::Plain(mut proof_bytes)) => {
                proof_bytes[0] = 0x07;
                VersionedProof::V3_0(Proof::new(ProofType::Plain, proof_bytes))
            }
            VersionedProof::V3_0(Proof::ZK(_)) => unreachable!(),
        };

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_versioned_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }
}
