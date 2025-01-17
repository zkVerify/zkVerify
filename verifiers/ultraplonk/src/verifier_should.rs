// Copyright 2024, Horizen Labs, Inc.

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

use serial_test::serial;

use super::*;
include!("resources.rs");

struct MockRuntime;

impl crate::Config for MockRuntime {
    type MaxPubs = sp_core::ConstU32<10>;
}

#[test]
#[serial]
fn verify_valid_proof() {
    let vk = VALID_VK;
    let proof = VALID_PROOF.to_vec();
    let pi = public_input();

    assert!(Ultraplonk::<MockRuntime>::verify_proof(&vk, &proof, &pi).is_ok());
}

#[test]
#[serial]
fn verify_valid_proof_with_8_public_inputs() {
    let proof = include_bytes!("resources/08_proof").to_vec();
    let pubs: Vec<_> = include_bytes!("resources/08_pubs")
        .chunks_exact(crate::PUBS_SIZE)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .collect();
    let vk = *include_bytes!("resources/08_vk");

    assert!(Ultraplonk::<MockRuntime>::verify_proof(&vk, &proof, &pubs).is_ok());
}

mod reject {
    use super::*;

    #[test]
    #[serial]
    fn invalid_public_values() {
        let vk = VALID_VK;
        let proof = VALID_PROOF.to_vec();

        let mut invalid_pubs = public_input();
        invalid_pubs[0][0] = 0x10;

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    #[serial]
    fn proof_with_8_public_inputs_with_one_not_valid() {
        let proof = include_bytes!("resources/08_proof").to_vec();
        let mut pubs: Vec<[u8; PUBS_SIZE]> = include_bytes!("resources/08_pubs")
            .chunks_exact(crate::PUBS_SIZE)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .collect();
        pubs[0][0] += 1;
        let vk = *include_bytes!("resources/08_vk");

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &proof, &pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    #[serial]
    fn if_provided_too_much_public_inputs() {
        let vk = VALID_VK;
        let proof = VALID_PROOF.to_vec();

        let mut invalid_pubs = public_input();
        while (invalid_pubs.len() as u32) < <MockRuntime as Config>::MaxPubs::get() {
            invalid_pubs.push(public_input()[0]);
        }

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    #[serial]
    fn invalid_number_of_public_inputs() {
        let vk = VALID_VK;
        let proof = VALID_PROOF.to_vec();

        let invalid_pubs = vec![public_input()[0]];

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    #[serial]
    fn invalid_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut invalid_proof: Proof = VALID_PROOF.to_vec();
        invalid_proof[PROOF_SIZE - 1] = 0x00;

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    #[serial]
    fn big_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut invalid_proof: Proof = VALID_PROOF.to_vec();
        invalid_proof.push(0);

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    #[serial]
    fn small_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut invalid_proof: Proof = VALID_PROOF.to_vec();
        invalid_proof.pop();

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    #[serial]
    fn invalid_vk() {
        let proof = VALID_PROOF.to_vec();
        let pi = public_input();

        let mut vk = VALID_VK;
        vk[0] = 0x10;

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &proof, &pi),
            Err(VerifyError::InvalidVerificationKey)
        );
    }

    #[test]
    #[serial]
    fn reject_malformed_proof() {
        let vk = VALID_VK;
        let pi = public_input();

        let mut malformed_proof: Proof = VALID_PROOF.to_vec();
        malformed_proof[0] = 0x07;

        assert_eq!(
            Ultraplonk::<MockRuntime>::verify_proof(&vk, &malformed_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }
}
