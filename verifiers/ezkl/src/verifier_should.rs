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
include!("resources.rs");

struct MockRuntime;

impl crate::Config for MockRuntime {
    type MaxPubs = sp_core::ConstU32<10>;
}

#[test]
fn verify_valid_proof() {
    let vk = EzklVk::new(VALID_VKA.to_vec());
    let proof = VALID_PROOF.to_vec();
    let pi = valid_instances();

    assert!(Ezkl::<MockRuntime>::verify_proof(&vk, &proof, &pi).is_ok());
}

#[test]
fn verify_valid_proof_alt() {
    let vk = EzklVk::new(VALID_VKA_ALT.to_vec());
    let proof = VALID_PROOF_ALT.to_vec();
    let pi = valid_instances_alt();

    assert!(Ezkl::<MockRuntime>::verify_proof(&vk, &proof, &pi).is_ok());
}

#[test]
fn verify_vk_hash() {
    let vk = EzklVk::new(VALID_VKA.to_vec());
    let vk_hash = Ezkl::<MockRuntime>::vk_hash(&vk);

    assert_eq!(
        vk_hash.as_bytes(),
        hex_literal::hex!("27833207b215f9d57bcf0adedaa5353bacd21e90252110f46b60943b54bd9518")
    );
}

mod reject {
    use super::*;

    #[test]
    fn invalid_public_values() {
        let vk = EzklVk::new(VALID_VKA.to_vec());
        let proof = VALID_PROOF.to_vec();

        let mut invalid_pubs = valid_instances();
        invalid_pubs[0][0] = 0x10;

        assert_eq!(
            Ezkl::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn if_provided_too_many_public_inputs() {
        let vk = EzklVk::new(VALID_VKA.to_vec());
        let proof = VALID_PROOF.to_vec();

        let mut invalid_pubs = valid_instances();
        while (invalid_pubs.len() as u32) <= <MockRuntime as Config>::MaxPubs::get() {
            invalid_pubs.push(valid_instances()[0]);
        }

        assert_eq!(
            Ezkl::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn invalid_number_of_public_inputs() {
        let vk = EzklVk::new(VALID_VKA.to_vec());
        let proof = VALID_PROOF.to_vec();

        let invalid_pubs = vec![valid_instances()[0]];

        assert_eq!(
            Ezkl::<MockRuntime>::verify_proof(&vk, &proof, &invalid_pubs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[test]
    fn big_proof() {
        let vk = EzklVk::new(VALID_VKA.to_vec());
        let pi = valid_instances();

        let mut invalid_proof = VALID_PROOF.to_vec();
        invalid_proof.push(1);

        assert_eq!(
            Ezkl::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn small_proof() {
        let vk = EzklVk::new(VALID_VKA.to_vec());
        let pi = valid_instances();

        let mut invalid_proof = VALID_PROOF.to_vec();
        invalid_proof.pop();

        assert_eq!(
            Ezkl::<MockRuntime>::verify_proof(&vk, &invalid_proof, &pi),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[test]
    fn invalid_vk() {
        let proof = VALID_PROOF.to_vec();
        let pi = valid_instances();

        let mut invalid_vk_bytes = VALID_VKA.to_vec();
        invalid_vk_bytes[0] = 0x10;
        let invalid_vk = EzklVk::new(invalid_vk_bytes);

        assert_eq!(
            Ezkl::<MockRuntime>::verify_proof(&invalid_vk, &proof, &pi),
            Err(VerifyError::VerifyError) // note that this is not an `InvalidVerificationKey` variant since verification fails
        );
    }

    #[test]
    fn reject_malformed_proof() {
        let vk = EzklVk::new(VALID_VKA.to_vec());
        let pi = valid_instances();

        let mut malformed_proof = VALID_PROOF.to_vec();
        malformed_proof[0] = 0x07;

        assert_eq!(
            Ezkl::<MockRuntime>::verify_proof(&vk, &malformed_proof, &pi),
            Err(VerifyError::InvalidProofData) // note that this is not an `VerifyError` variant since the faulty bytes get intercepted earlier
        );
    }
}
