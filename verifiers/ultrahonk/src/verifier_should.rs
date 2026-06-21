// Copyright 2025-2026, Horizen Labs, Inc.

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

impl Default for Proof {
    fn default() -> Self {
        Self::ZK(Vec::new()) // mirrors Noir's default
    }
}

impl Default for VersionedProof {
    fn default() -> Self {
        Self::V3_0(Proof::default())
    }
}

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
        ProtocolVersion::Legacy => TestParams::new_legacy(proof_type),
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
        VersionedProof::Legacy(proof) => {
            let pt = ProofType::from(&proof);
            let mut raw: RawProof = proof.into();
            mutator(&mut raw);
            VersionedProof::Legacy(Proof::new(pt, raw))
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
        VersionedProof::Legacy(proof) => {
            let mut raw: RawProof = proof.into();
            mutator(&mut raw);
            VersionedProof::Legacy(Proof::new(new_type, raw))
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
        VersionedVk::Legacy(mut vk) => {
            mutator(&mut vk);
            VersionedVk::Legacy(vk)
        }
    }
}

#[rstest]
fn verify_valid_proof(
    #[values(ProofType::ZK, ProofType::Plain)] proof_type: ProofType,
    #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
    version: ProtocolVersion,
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
#[case::legacy(
    ProtocolVersion::Legacy,
    // Legacy VK hash: SHA2-256 of raw VK bytes (matching pre-versioning code at
    // commit 113a728c, verifiers/ultrahonk/src/lib.rs:183-184)
    hex!("f28c8c634872785e7c6ae10684b7952f9d4822d065cee182489d5e49c995080d"),
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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

    #[test]
    fn legacy_proof_with_v0_84_vk() {
        let TestData {
            versioned_vk, pubs, ..
        } = load_test_data(ProofType::ZK, ProtocolVersion::V0_84);

        let TestData {
            versioned_proof: legacy_proof,
            ..
        } = load_test_data(ProofType::ZK, ProtocolVersion::Legacy);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&versioned_vk, &legacy_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn v0_84_proof_with_legacy_vk() {
        let TestData {
            versioned_vk: legacy_vk,
            pubs,
            ..
        } = load_test_data(ProofType::ZK, ProtocolVersion::Legacy);

        let TestData {
            versioned_proof: v0_84_proof,
            ..
        } = load_test_data(ProofType::ZK, ProtocolVersion::V0_84);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&legacy_vk, &v0_84_proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[rstest]
    fn malformed_proof(
        #[values(ProofType::ZK, ProofType::Plain)] proof_type: ProofType,
        #[values(ProtocolVersion::V3_0, ProtocolVersion::V0_84, ProtocolVersion::Legacy)]
        version: ProtocolVersion,
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

/// Regression test: verify that the Legacy variant produces the same statement hash as the
/// pre-versioning code. Uses the exact same test data from before commit 83e40f29:
///   - VALID_VK: commit 113a728c, verifiers/ultrahonk/src/resources.rs:968
///   - public_input(): commit 113a728c, verifiers/ultrahonk/src/resources.rs:17-21
///
/// The pre-versioning code computed:
///   - vk_hash: SHA2-256(raw_vk_bytes) (commit 113a728c, verifiers/ultrahonk/src/lib.rs:183-184)
///   - vk_bytes: raw bytes via to_vec() (commit 113a728c, verifiers/ultrahonk/src/lib.rs:202-204)
///   - verifier_version_hash: NO_VERSION_HASH (default trait impl)
///   - hash_context_data: b"ultrahonk"
///   - pubs_bytes: flat concatenation of public inputs
///
/// Statement hash formula (pallets/verifiers/src/lib.rs:192-211):
///   keccak256(keccak256(context) || vk_hash || verifier_version_hash || keccak256(pubs_bytes))
#[test]
fn legacy_produces_old_statement_hash() {
    // VALID_VK from commit 113a728c, verifiers/ultrahonk/src/resources.rs:968
    let raw_vk = hex!(
        "
        0000000000000020000000000000000500000000000000020000000000000001
        1d4e2b662cf75598ae75c80cb6190d6d86bc92fd69f1420fc9e6d5be8ba09e2c
        30210ded34398f54e3048f65c3f1dac749cc5022828668a6b345712af7369cbb
        1c3736f27bc34afe8eb1021704555717e76024100c144933330df5d9a6fb7e7f
        215612b168ecf42291b6df40da24069d5a0d5f2599d8be1ec34c5095e0922151
        059aecd0bba76edd4de929d587575b50c50f4be99a4615bfbd4ece89cb1442f1
        121b12b8bfa67425811621a1be826bcc5add41edb51fdce6c134c8e3ff5b1578
        2ad6f88dd8a25590c065ad43adb6f3d4ccba5a7312f27dd564b12325a2594ae5
        038c0c60a3dfed43a24eefcc0331f08074bea7bb5c7f65191ec2c3fe59a239cc
        17bebc96661564acc3f5c59647e9270570e0c238916df6390c8590445f256d1d
        0bf23741444a9bf150d33f19d70a31863256e71d2bb1adf96b04d61f2c95a2c4
        1b8058db3a5b9890b24d2545b7dd4aca37844bb0964691811a3dfe7b9fd24f8f
        28362861904e4b69161d7f43201c9213ede6e74eb63800123b82c73ad0156c40
        3058b7f62cbcbdc8763b05935e9965bea86cd205281d331fb426ef4232ffe5c5
        2b312f13fea65176bc0fe06aef8724f256898d215c78835f40bfe56fbf3f0de3
        0ac6c48b063b744bbeecb29c8962cf27853ae788601a92a0420ba047a7f7a643
        265a8af9070f8bd5e18bc97a13c985d35a59c188d3d5ee626bbc4589bba9ff9f
        024236bda126650fb5228cf424a0878775499e69e8bd2c39af33bd5fa0b4079a
        233cda9292be02cfa2da9d0fc7b0eab0eb1a867b06854066589b967455259b32
        0ca0bc4b1cd9eadbbf49eae56a99a4502ef13d965226a634d0981555e4a4da56
        1a8a818e6c61f68cefa329f2fabc95c80ad56a538d852f75eda858ed1a616c74
        09dfd2992ac1708f0dd1d28c2ad910d9cf21a1510948580f406bc9416113d620
        205f76eebda12f565c98c775c4e4f3534b5dcc29e57eed899b1a1a880534dcb9
        1b8afad764d2cbe67c94249535bba7fcbd3f412f868487222aa54f3268ab64a2
        01b70a90a334c9bd5096aad8a0cc5d4c1d1cdb0fe415445bd0c84309caaf213e
        13240f97a584b45184c8ec31319b5f6c04ee19ec1dfec87ed47d6d04aa158de2
        2dad22022121d689f57fb38ca21349cefb5d240b07ceb4be26ea429b6dc9d9e0
        2dbea5caeded6749d2ef2e2074dbea56c8d54fa043a54c6e6a40238fb0a52c8e
        1f299b74e3867e8c8bc149ef3a308007a3bd6f9935088ec247cce992c33a5336
        06652c2a72cb81284b190e235ee029a9463f36b2e29a1775c984b9d9b2714bab
        268e8d1e619fde85a71e430b77974326d790cb64c87558085332df639b8ce410
        2849ce9f77669190ed63388b3cc4a6d4e0d895c683ae0057f36a00e62416de5e
        2f8d58d08d4b4bb3a63e23e091e7a1f13c581c8a98c75014d5ec8a20890c62a5
        0fff3b4e49a2e6e05bc63d8438368182639ef435c89f30e3a3a9053d97bea5f2
        1820cafe7ffbef14880565ed976d53ed31c844187447d21f09100e8e569d3aec
        2e89eeb660cac820de50be4c53b608dd67c6977f5f1746fcf0fb6475d81ccd93
        18ca593957d2677420236138b3659a6b95b580bcc09a3dfbdadfa58a38222c15
        0c756ba6a0c66b05655349f04c61dff94dddf3a4d0117fafda741f9518c42f00
        0f87a1201ebad9bd23fed33824ae4ba2a1a307a45fb15594f8d553d2ebf9c285
        248460656ec9bc0ad940051e3b0751d25bb97885d8bc362eb06b96ea78d82f84
        0a5eebc538dc40185864706e22d850e3c02ce38e325761a59132bdb9e9d795be
        161edd8773a3b74c0553b690b4b80b2a5cbd4a1a25fda097bef23e349531b43e
        287139da895215c216aebe8cce7d3b944f4a3b051bd407126007921cb1fbc5fc
        20d671263cad88c119d0a5d172679309087e385f8e76d4cfa834fab61ebd6603
        0f9e6dfd3e6f4584b28e2cb00483dc2ffd9bf5f7ae2cc3f1ea0869c5ae71d9a1
        101e267b586089a8bb447e83ab3b7029ed788cc214e0be44485e2f39afbb7ae6
        13410d68bce429dc36e23023cfe21c5f2ced7e136529a4bcd4317232f2fc16b6
        1054a26ae3aeeeedc653cf5c5e3c09e2258141e67f4a5a48b50cbf48958b40bd
        2d14190edcf9b2aa697b677c779083aaf0151cc4f673dcf4bdba392d6280e376
        2e9e762a66fed77eb0e72645e5ba54f32c1d1bfbc4bd862361dafd7ebd6c68dd
        0b4a012fbc876f57da669215383f3595383f787bca153e972e6cfb9dfebeaa1b
        0000000000000000000000000000000000000000000000000000000000000001
        0000000000000000000000000000000000000000000000000000000000000002
        0af3884ecad3331429af995779c2602e93ca1ea976e9e1bc64bbcdbb9fe79212
        1f18803add8ad686e13dc2a989dcfb010cb69b0b38200df51787b7104bc74fb6
        "
    );
    let versioned_vk = VersionedVk::Legacy(raw_vk);

    // VK hash must match verify_vk_hash() from commit 113a728c,
    // verifiers/ultrahonk/src/verifier_should.rs:80-88
    let vk_hash = Ultrahonk::<MockRuntime>::vk_hash(&versioned_vk);
    assert_eq!(
        vk_hash.as_bytes(),
        hex!("862d96a25fb215e349e53f808e5cc5fff65c789f7a751c07857fdded9e3cbece"),
    );

    // public_input() from commit 113a728c, verifiers/ultrahonk/src/resources.rs:17-21
    let pubs: Pubs = alloc::vec![
        hex!("0000000000000000000000000000000000000000000000000000000000000002"),
        hex!("0000000000000000000000000000000000000000000000000000000000000003"),
    ];

    // Proof bytes don't affect statement hash, only the Legacy variant matters
    // (verifier_version_hash returns NO_VERSION_HASH for Legacy).
    let versioned_proof = VersionedProof::Legacy(Proof::default());

    let vk_or_hash = pallet_verifiers::VkOrHash::Vk(versioned_vk.into());
    let statement_hash = pallet_verifiers::compute_statement_hash::<Ultrahonk<MockRuntime>>(
        &vk_or_hash,
        &versioned_proof,
        &pubs,
    );

    // This is the same statement hash that the pre-versioning code would have produced
    // with VALID_VK and public_input() test data.
    assert_eq!(
        statement_hash.as_bytes(),
        &hex!("f0350931fcf99dcfb278b0b4e9752bef7bc22de05a822e220a3bc7013a90be72"),
        "Legacy statement hash should match the pre-versioning statement hash"
    );
}

#[test]
fn legacy_verifier_version_hash_is_no_version_hash() {
    let TestData {
        versioned_proof, ..
    } = load_test_data(ProofType::Plain, ProtocolVersion::Legacy);

    assert_eq!(
        Ultrahonk::<MockRuntime>::verifier_version_hash(&versioned_proof),
        pallet_verifiers::traits::NO_VERSION_HASH,
    );
}
