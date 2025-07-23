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

use hex_literal::hex;

use super::*;
include!("resources.rs");

struct MockRuntime;

impl crate::Config for MockRuntime {
    type MaxPubs = sp_core::ConstU32<10>;
}

#[test]
fn verify_valid_zk_proof() {
    let vk = VALID_VK;
    let proof = RawProofWithType::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());
    let pi = public_input();

    assert!(Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pi).is_ok());
}

#[test]
fn verify_valid_plain_proof() {
    let vk = VALID_VK;
    let proof = RawProofWithType::new(ProofType::Plain, VALID_PLAIN_PROOF.to_vec());
    let pi = public_input();

    assert!(Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pi).is_ok());
}

#[test]
fn verify_valid_zk_proof_with_8_public_inputs() {
    let proof = RawProofWithType::new(
        ProofType::ZK,
        include_bytes!("resources/08/zk/zk_proof").to_vec(),
    );
    let pubs: Vec<_> = include_bytes!("resources/08/zk/pubs")
        .chunks_exact(crate::PUB_SIZE)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .collect();
    let vk = include_bytes!("resources/08/zk/vk");

    assert!(Ultrahonk::<MockRuntime>::verify_proof(vk, &proof, &pubs).is_ok());
}

#[test]
fn verify_valid_plain_proof_with_8_public_inputs() {
    let proof = RawProofWithType::new(
        ProofType::Plain,
        include_bytes!("resources/08/plain/plain_proof").to_vec(),
    );
    let pubs: Vec<_> = include_bytes!("resources/08/plain/pubs")
        .chunks_exact(crate::PUB_SIZE)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .collect();
    let vk = include_bytes!("resources/08/plain/vk");

    assert!(Ultrahonk::<MockRuntime>::verify_proof(vk, &proof, &pubs).is_ok());
}

#[test]
fn verify_vk_hash() {
    let vk = VALID_VK;
    let vk_hash = Ultrahonk::<MockRuntime>::vk_hash(&vk);

    assert_eq!(
        vk_hash.as_bytes(),
        hex!("862d96a25fb215e349e53f808e5cc5fff65c789f7a751c07857fdded9e3cbece")
    );
}

mod reject {
    use super::*;

    #[test]
    fn invalid_public_values() {
        let vk = VALID_VK;
        let proof = RawProofWithType::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());

        let mut invalid_pubs = public_input();
        invalid_pubs[0][0] = 0x10;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn zk_proof_with_8_public_inputs_with_one_not_valid() {
        let proof = RawProofWithType::new(
            ProofType::ZK,
            include_bytes!("resources/08/zk/zk_proof").to_vec(),
        );
        let mut pubs: Vec<[u8; PUB_SIZE]> = include_bytes!("resources/08/zk/pubs")
            .chunks_exact(crate::PUB_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        pubs[0][0] += 1;
        let vk = include_bytes!("resources/08/zk/vk");

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(vk, &proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn plain_proof_with_8_public_inputs_with_one_not_valid() {
        let proof = RawProofWithType::new(
            ProofType::Plain,
            include_bytes!("resources/08/plain/plain_proof").to_vec(),
        );
        let mut pubs: Vec<[u8; PUB_SIZE]> = include_bytes!("resources/08/plain/pubs")
            .chunks_exact(crate::PUB_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        pubs[0][0] += 1;
        let vk = include_bytes!("resources/08/plain/vk");

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(vk, &proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn if_provided_too_many_public_inputs_for_zk_proof() {
        let vk = VALID_VK;
        let proof = RawProofWithType::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());

        let mut invalid_pubs = public_input();
        while (invalid_pubs.len() as u32) < <MockRuntime as Config>::MaxPubs::get() {
            invalid_pubs.push(public_input()[0]);
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn if_provided_too_many_public_inputs_for_plain_proof() {
        let vk = VALID_VK;
        let proof = RawProofWithType::new(ProofType::Plain, VALID_PLAIN_PROOF.to_vec());

        let mut invalid_pubs = public_input();
        while (invalid_pubs.len() as u32) <= <MockRuntime as Config>::MaxPubs::get() {
            invalid_pubs.push(public_input()[0]);
        }

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn invalid_number_of_public_inputs() {
        let vk = VALID_VK;
        let proof = RawProofWithType::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());

        let invalid_pubs = vec![public_input()[0]];

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn invalid_zk_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut invalid_proof_bytes = VALID_ZK_PROOF.to_vec();
        invalid_proof_bytes[ZK_PROOF_SIZE - 1] = 0x00;

        let invalid_proof = RawProofWithType::new(ProofType::ZK, invalid_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn invalid_plain_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut invalid_proof_bytes = VALID_PLAIN_PROOF.to_vec();
        invalid_proof_bytes[PLAIN_PROOF_SIZE - 1] = 0x00;

        let invalid_proof = RawProofWithType::new(ProofType::Plain, invalid_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn big_zk_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let valid_proof: [u8; ZK_PROOF_SIZE] = VALID_ZK_PROOF;
        let mut invalid_proof_bytes = [0u8; ZK_PROOF_SIZE + 1];

        invalid_proof_bytes[..ZK_PROOF_SIZE].copy_from_slice(&valid_proof);
        invalid_proof_bytes[ZK_PROOF_SIZE] = 0;

        let invalid_proof = RawProofWithType::new(ProofType::ZK, invalid_proof_bytes.to_vec());

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn big_plain_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let valid_proof: [u8; PLAIN_PROOF_SIZE] = VALID_PLAIN_PROOF;
        let mut invalid_proof_bytes = [0u8; PLAIN_PROOF_SIZE + 1];

        invalid_proof_bytes[..PLAIN_PROOF_SIZE].copy_from_slice(&valid_proof);
        invalid_proof_bytes[PLAIN_PROOF_SIZE] = 0;

        let invalid_proof = RawProofWithType::new(ProofType::Plain, invalid_proof_bytes.to_vec());

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn small_zk_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut invalid_proof_bytes = VALID_ZK_PROOF.to_vec();
        invalid_proof_bytes.pop();
        let invalid_proof = RawProofWithType::new(ProofType::ZK, invalid_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn small_plain_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut invalid_proof_bytes = VALID_PLAIN_PROOF.to_vec();
        invalid_proof_bytes.pop();
        let invalid_proof = RawProofWithType::new(ProofType::Plain, invalid_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn invalid_vk() {
        let proof = RawProofWithType::new(ProofType::ZK, VALID_ZK_PROOF.to_vec());
        let pi = public_input();

        let mut invalid_vk = [0u8; VK_SIZE];
        invalid_vk[..VK_SIZE].copy_from_slice(&VALID_VK);
        invalid_vk[0] = 0x10;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&invalid_vk, &proof, &pi),
            Err(VerifyError::InvalidVerificationKey)
        );
    }

    #[test]
    fn reject_malformed_zk_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut malformed_proof_bytes = VALID_ZK_PROOF.to_vec();
        malformed_proof_bytes[0] = 0x07;

        let malformed_proof = RawProofWithType::new(ProofType::ZK, malformed_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &malformed_proof, &pi),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn reject_malformed_plain_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut malformed_proof_bytes = VALID_PLAIN_PROOF.to_vec();
        malformed_proof_bytes[0] = 0x07;

        let malformed_proof = RawProofWithType::new(ProofType::Plain, malformed_proof_bytes);

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &malformed_proof, &pi),
            Err(VerifyError::VerifyError)
        );
    }
}
