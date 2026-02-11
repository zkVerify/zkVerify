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

use super::*;
use hex_literal::hex;

struct MockRuntime;

impl crate::Config for MockRuntime {
    type MaxPubs = sp_core::ConstU32<32>;
    type WeightInfo = ();
}

// Default (proof, vk, pubs) triplets for unit testing:

const VALID_ZK_PROOF: &[u8] = include_bytes!("resources/zk/log_25/proof");
const VALID_PLAIN_PROOF: &[u8] = include_bytes!("resources/plain/log_25/proof");

#[allow(dead_code)]
pub(crate) fn valid_vk() -> crate::Vk {
    let vk_bytes: &[u8] = include_bytes!("resources/zk/log_25/vk");
    let vk: [u8; crate::VK_SIZE] = vk_bytes
        .try_into()
        .expect("Benchmark file should always have the correct vk size");
    vk
}

#[allow(dead_code)]
pub(crate) fn valid_public_input() -> crate::Pubs {
    include_bytes!("resources/zk/log_25/pubs")
        .chunks_exact(crate::PUB_SIZE)
        .map(|c| c.try_into().unwrap())
        .collect()
}

#[test]
fn verify_valid_zk_proof() {
    let vk = valid_vk();
    let proof = Proof::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());
    let pi = valid_public_input();

    let res = Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pi);

    println!("{:?}", res);

    assert!(res.is_ok());
}

#[test]
fn verify_valid_plain_proof() {
    let vk = valid_vk();
    let proof = Proof::new(ProofType::Plain, VALID_PLAIN_PROOF.to_vec());
    let pi = valid_public_input();

    assert!(Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pi).is_ok());
}

#[test]
fn verify_vk_hash() {
    let vk = valid_vk();
    let vk_hash = Ultrahonk::<MockRuntime>::vk_hash(&vk);

    assert_eq!(
        vk_hash.as_bytes(),
        hex!("db7e16ea447c0514657eda8c51d3d1599e26a86bfe4608b67070b18f5b4cde96")
    );
}

mod reject {
    use super::*;

    #[test]
    fn invalid_public_values() {
        let vk = valid_vk();
        let proof = Proof::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());

        let mut invalid_pubs = valid_public_input();
        invalid_pubs[0][0] = 0x10;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn zk_proof_with_32_public_inputs_where_one_is_invalid() {
        let proof = Proof::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());
        let mut pubs: Vec<[u8; PUB_SIZE]> = valid_public_input();
        let vk = valid_vk();

        // Render first public input invalid
        pubs[0][0] += 1;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn plain_proof_with_32_public_inputs_where_one_is_invalid() {
        let proof = Proof::new(ProofType::Plain, VALID_PLAIN_PROOF.to_vec());
        let mut pubs: Vec<[u8; PUB_SIZE]> = valid_public_input();
        let vk = valid_vk();

        // Render first public input invalid
        pubs[0][0] += 1;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn if_provided_too_many_public_inputs_for_zk_proof() {
        let vk = valid_vk();
        let proof = Proof::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());

        let mut invalid_pubs = valid_public_input();
        while (invalid_pubs.len() as u32) <= <MockRuntime as Config>::MaxPubs::get() {
            invalid_pubs.push(valid_public_input()[0]);
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn if_provided_too_many_public_inputs_for_plain_proof() {
        let vk = valid_vk();
        let proof = Proof::new(ProofType::Plain, VALID_PLAIN_PROOF.to_vec());

        let mut invalid_pubs = valid_public_input();
        while (invalid_pubs.len() as u32) <= <MockRuntime as Config>::MaxPubs::get() {
            invalid_pubs.push(valid_public_input()[0]);
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn invalid_number_of_public_inputs() {
        let vk = valid_vk();
        let proof = Proof::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());

        let mut invalid_pubs = valid_public_input();
        invalid_pubs.pop();

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn invalid_zk_proof() {
        let vk = valid_vk();
        let pi = valid_public_input();

        let mut invalid_proof_bytes = VALID_ZK_PROOF.to_vec();
        invalid_proof_bytes[VALID_ZK_PROOF.len() - 1] = 0x00;

        let invalid_proof = Proof::new(ProofType::ZK, invalid_proof_bytes);

        assert!(matches!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::VerifyError) | Err(VerifyError::InvalidProofData)
        ));
    }

    #[test]
    fn invalid_plain_proof() {
        let vk = valid_vk();
        let pi = valid_public_input();

        let mut invalid_proof_bytes = VALID_PLAIN_PROOF.to_vec();
        invalid_proof_bytes[VALID_PLAIN_PROOF.len() - 1] = 0x00;

        let invalid_proof = Proof::new(ProofType::Plain, invalid_proof_bytes);

        assert!(matches!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::VerifyError) | Err(VerifyError::InvalidProofData)
        ));
    }

    #[test]
    fn big_zk_proof() {
        let vk = valid_vk();
        let pi = valid_public_input();

        let valid_proof = VALID_ZK_PROOF;
        let mut invalid_proof_bytes = [0u8; VALID_ZK_PROOF.len() + 1];

        invalid_proof_bytes[..VALID_ZK_PROOF.len()].copy_from_slice(&valid_proof);

        let invalid_proof = Proof::new(ProofType::ZK, invalid_proof_bytes.to_vec());

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn big_plain_proof() {
        let vk = valid_vk();
        let pi = valid_public_input();

        let valid_proof = VALID_PLAIN_PROOF;
        let mut invalid_proof_bytes = [0u8; VALID_PLAIN_PROOF.len() + 1];

        invalid_proof_bytes[..VALID_PLAIN_PROOF.len()].copy_from_slice(&valid_proof);

        let invalid_proof = Proof::new(ProofType::Plain, invalid_proof_bytes.to_vec());

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn small_zk_proof() {
        let vk = valid_vk();
        let pi = valid_public_input();

        let mut invalid_proof_bytes = VALID_ZK_PROOF.to_vec();
        invalid_proof_bytes.pop();
        let invalid_proof = Proof::new(ProofType::ZK, invalid_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn small_plain_proof() {
        let vk = valid_vk();
        let pi = valid_public_input();

        let mut invalid_proof_bytes = VALID_PLAIN_PROOF.to_vec();
        invalid_proof_bytes.pop();
        let invalid_proof = Proof::new(ProofType::Plain, invalid_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn invalid_vk() {
        let proof = Proof::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());
        let pi = valid_public_input();

        let mut invalid_vk = [0u8; VK_SIZE];
        invalid_vk[..VK_SIZE].copy_from_slice(&valid_vk());
        invalid_vk[0] = 0x10;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&invalid_vk, &proof, &pi),
            Err(VerifyError::InvalidVerificationKey)
        );
    }

    #[test]
    fn reject_malformed_zk_proof() {
        let vk = valid_vk();
        let pi = valid_public_input();

        let mut malformed_proof_bytes = VALID_ZK_PROOF.to_vec();
        malformed_proof_bytes[0] = 0x07;

        let malformed_proof = Proof::new(ProofType::ZK, malformed_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &malformed_proof, &pi),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn reject_malformed_plain_proof() {
        let vk = valid_vk();
        let pi = valid_public_input();

        let mut malformed_proof_bytes = VALID_PLAIN_PROOF.to_vec();
        malformed_proof_bytes[0] = 0x07;

        let malformed_proof = Proof::new(ProofType::Plain, malformed_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &malformed_proof, &pi),
            Err(VerifyError::VerifyError)
        );
    }
}
