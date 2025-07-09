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
use serial_test::serial;

use super::*;
include!("resources.rs");

struct MockRuntime;

impl crate::Config for MockRuntime {
    type MaxPubs = sp_core::ConstU32<10>;
}

#[test]
#[serial]
fn verify_valid_zk_proof() {
    let vk = VALID_VK;
    let proof = VALID_ZK_PROOF.to_vec();
    let pi = public_input();

    assert!(Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pi).is_ok());
}

#[test]
#[serial]
fn verify_valid_proof_with_8_public_inputs() {
    let proof = include_bytes!("resources/08/08_zk_proof").to_vec();
    let pubs: Vec<_> = include_bytes!("resources/08/08_pubs")
        .chunks_exact(crate::PUB_SIZE)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .collect();
    let vk = *include_bytes!("resources/08/08_vk");

    assert!(Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pubs).is_ok());
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
    #[serial]
    fn invalid_public_values() {
        let vk = VALID_VK;
        let proof = VALID_ZK_PROOF.to_vec();

        let mut invalid_pubs = public_input();
        invalid_pubs[0][0] = 0x10;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    #[serial]
    fn zk_proof_with_8_public_inputs_with_one_not_valid() {
        let proof = include_bytes!("resources/08/08_zk_proof").to_vec();
        let mut pubs: Vec<[u8; PUB_SIZE]> = include_bytes!("resources/08/08_pubs")
            .chunks_exact(crate::PUB_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        pubs[0][0] += 1;
        let vk = *include_bytes!("resources/08/08_vk");

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    #[serial]
    fn if_provided_too_many_public_inputs() {
        let vk = VALID_VK;
        let proof = VALID_ZK_PROOF.to_vec();

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
    #[serial]
    fn invalid_number_of_public_inputs() {
        let vk = VALID_VK;
        let proof = VALID_ZK_PROOF.to_vec();

        let invalid_pubs = vec![public_input()[0]];

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    #[serial]
    fn invalid_zk_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut invalid_proof = VALID_ZK_PROOF.to_vec();
        invalid_proof[ZK_PROOF_SIZE - 1] = 0x00;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::VerifyError)
        );
    }

    // #[test]
    // #[serial]
    // fn big_zk_proof() {
    //     let vk = VALID_VK;
    //     let pi = public_input();

    //     let valid_proof: [u8; ZK_PROOF_SIZE] = VALID_ZK_PROOF;
    //     let mut invalid_proof = [0u8; ZK_PROOF_SIZE + 1];

    //     invalid_proof[..ZK_PROOF_SIZE].copy_from_slice(&valid_proof);
    //     invalid_proof[ZK_PROOF_SIZE] = 0;

    //     assert_eq!(
    //         Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
    //         Err(VerifyError::InvalidInput)
    //     );
    // }

    // #[test]
    // #[serial]
    // fn small_zk_proof() {
    //     let vk = VALID_VK;
    //     let pi = public_input();

    //     let mut invalid_proof = VALID_ZK_PROOF.to_vec();
    //     invalid_proof.pop();

    //     assert_eq!(
    //         Ultrahonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
    //         Err(VerifyError::InvalidInput)
    //     );
    // }

    #[test]
    #[serial]
    fn invalid_vk() {
        let proof = VALID_ZK_PROOF.to_vec();
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
    #[serial]
    fn reject_malformed_zk_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut malformed_proof = VALID_ZK_PROOF.to_vec();
        malformed_proof[0] = 0x07;

        assert_eq!(
            Ultrahonk::<MockRuntime>::verify_proof(&vk, &malformed_proof, &pi),
            Err(VerifyError::VerifyError)
        );
    }
}
