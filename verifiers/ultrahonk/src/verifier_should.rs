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
use rstest::rstest;

struct MockRuntime;

impl crate::Config for MockRuntime {
    type MaxPubs = sp_core::ConstU32<32>;
    type WeightInfo = ();
}

fn load_test_data(proof_type: ProofType, version: ProtocolVersion) -> TestData {
    let test_params = match version {
        ProtocolVersion::V3_0 => {
            TestParams::new(MAX_BENCHMARKED_LOG_CIRCUIT_SIZE, proof_type, version)
        }
        ProtocolVersion::V0_84 => TestParams::new_v0_84(proof_type),
    };
    get_parameterized_test_data(test_params).expect("test data should be present")
}

/// Mutate the raw proof bytes within a VersionedProof, preserving the proof type.
fn mutate_proof_bytes(
    versioned_proof: VersionedProof,
    mutator: impl FnOnce(&mut Vec<u8>),
) -> VersionedProof {
    match versioned_proof {
        VersionedProof::V0_84(proof) => {
            let pt = ProofType::from(&proof);
            let mut raw: RawProof = proof.into();
            mutator(&mut raw);
            VersionedProof::V0_84(Proof::new(pt, raw))
        }
        VersionedProof::V3_0(proof) => {
            let pt = ProofType::from(&proof);
            let mut raw: RawProof = proof.into();
            mutator(&mut raw);
            VersionedProof::V3_0(Proof::new(pt, raw))
        }
    }
}

/// Mutate the raw proof bytes and override the proof type.
fn mutate_proof_bytes_with_type(
    versioned_proof: VersionedProof,
    new_type: ProofType,
    mutator: impl FnOnce(&mut Vec<u8>),
) -> VersionedProof {
    match versioned_proof {
        VersionedProof::V0_84(proof) => {
            let mut raw: RawProof = proof.into();
            mutator(&mut raw);
            VersionedProof::V0_84(Proof::new(new_type, raw))
        }
        VersionedProof::V3_0(proof) => {
            let mut raw: RawProof = proof.into();
            mutator(&mut raw);
            VersionedProof::V3_0(Proof::new(new_type, raw))
        }
    }
}

/// Mutate the raw VK bytes within a VersionedVk.
fn mutate_vk_bytes(versioned_vk: VersionedVk, mutator: impl FnOnce(&mut [u8])) -> VersionedVk {
    match versioned_vk {
        VersionedVk::V0_84(mut vk) => {
            mutator(&mut vk);
            VersionedVk::V0_84(vk)
        }
        VersionedVk::V3_0(mut vk) => {
            mutator(&mut vk);
            VersionedVk::V3_0(vk)
        }
    }
}

#[rstest]
fn verify_valid_proof(
    #[values(ProofType::ZK, ProofType::Plain)] proof_type: ProofType,
    #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
) {
    let TestData {
        versioned_vk,
        versioned_proof,
        pubs,
    } = load_test_data(proof_type, version);

    assert!(Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs).is_ok());
}

#[rstest]
#[case::v0_84(
    ProtocolVersion::V0_84,
    hex!("293060325b05b7f7ce08e13d028d791f598b4856e3523e1e3e7c8b7f341805d1"),
)]
#[case::v3_0(
    ProtocolVersion::V3_0,
    hex!("22af324f6deda316ee8b2d91cea110dbbf67715caed46b2f953a91725b8032bc"),
)]
fn verify_vk_hash(#[case] version: ProtocolVersion, #[case] expected: [u8; 32]) {
    let TestData { versioned_vk, .. } = load_test_data(ProofType::Plain, version);

    let vk_hash = Ultrahonk::<MockRuntime>::vk_hash(&versioned_vk);

    assert_eq!(vk_hash.as_bytes(), expected);
}

mod reject {
    use super::*;

    #[rstest]
    fn invalid_public_values(
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = load_test_data(ProofType::ZK, version);

        let mut invalid_pubs = pubs;
        invalid_pubs[0][0] = 0x10;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &invalid_pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[rstest]
    fn proof_with_one_invalid_public_input(
        #[values(ProofType::ZK, ProofType::Plain)] proof_type: ProofType,
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            mut pubs,
        } = load_test_data(proof_type, version);

        pubs[0][0] += 1;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[rstest]
    fn too_many_public_inputs(
        #[values(ProofType::ZK, ProofType::Plain)] proof_type: ProofType,
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            mut pubs,
        } = load_test_data(proof_type, version);

        while (pubs.len() as u32) <= <MockRuntime as Config>::MaxPubs::get() {
            pubs.push([0u8; PUB_SIZE]);
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[rstest]
    fn invalid_number_of_public_inputs(
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            mut pubs,
        } = load_test_data(ProofType::Plain, version);

        if pubs.is_empty() {
            pubs.push([0u8; PUB_SIZE]);
        } else {
            pubs.pop();
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &versioned_proof, &pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[rstest]
    fn invalid_zk_proof(
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = load_test_data(ProofType::ZK, version);

        let invalid_proof = mutate_proof_bytes(versioned_proof, |bytes| {
            let len = bytes.len();
            bytes[len - 1] = 0x00;
        });

        assert!(matches!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_proof, &pubs),
            Err(VerifyError::VerifyError) | Err(VerifyError::InvalidProofData)
        ));
    }

    #[rstest]
    fn invalid_plain_proof(
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = load_test_data(ProofType::Plain, version);

        let invalid_proof = mutate_proof_bytes_with_type(versioned_proof, ProofType::ZK, |bytes| {
            let len = bytes.len();
            bytes[len - 1] = 0x00;
        });

        assert!(matches!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_proof, &pubs),
            Err(VerifyError::VerifyError) | Err(VerifyError::InvalidProofData)
        ));
    }

    #[rstest]
    fn oversized_proof(
        #[values(ProofType::ZK, ProofType::Plain)] proof_type: ProofType,
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = load_test_data(proof_type, version);

        let invalid_proof = mutate_proof_bytes(versioned_proof, |bytes| bytes.push(0x00));

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_proof, &pubs),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[rstest]
    fn undersized_proof(
        #[values(ProofType::ZK, ProofType::Plain)] proof_type: ProofType,
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = load_test_data(proof_type, version);

        let invalid_proof = mutate_proof_bytes(versioned_proof, |bytes| {
            bytes.pop();
        });

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_proof, &pubs),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[rstest]
    fn invalid_vk(
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = load_test_data(ProofType::ZK, version);

        let invalid_vk = mutate_vk_bytes(versioned_vk, |bytes| bytes[0] = 0x10);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&invalid_vk, &versioned_proof, &pubs),
            Err(VerifyError::InvalidVerificationKey)
        );
    }

    #[test]
    fn proof_with_version_not_matching_the_vk_version() {
        let TestData {
            versioned_vk, pubs, ..
        } = load_test_data(ProofType::ZK, ProtocolVersion::V0_84);

        let v3_0_versioned_proof = VersionedProof::V3_0(Proof::default());

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &v3_0_versioned_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[rstest]
    fn malformed_proof(
        #[values(ProofType::ZK, ProofType::Plain)] proof_type: ProofType,
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84)] version: ProtocolVersion,
    ) {
        let TestData {
            versioned_vk,
            versioned_proof,
            pubs,
        } = load_test_data(proof_type, version);

        let invalid_proof = mutate_proof_bytes(versioned_proof, |bytes| bytes[0] = 0x07);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &invalid_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }
}
