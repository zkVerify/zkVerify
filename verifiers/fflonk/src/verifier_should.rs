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

use sp_core::U256;

use super::*;
include!("resources.rs");

#[test]
fn verify_valid_proof() {
    let vk = cdk_key();

    assert!(Fflonk::verify_proof(&vk, &VALID_PROOF, &VALID_PUBS).is_ok());
}

#[test]
fn return_the_same_bytes_as_public_inputs() {
    // We use some other bytes to be sure that the pubs are not hardcoded
    let data: [u8; 32] = VALID_PROOF[0..32].try_into().unwrap();
    assert_eq!(Fflonk::pubs_bytes(&data).as_ref(), &data);
}

#[test]
fn validate_valid_vk() {
    let vk = cdk_key();
    assert!(Fflonk::validate_vk(&vk).is_ok())
}

#[test]
fn reject_malformed_vk() {
    let mut vk = cdk_key();

    *vk.mut_c0_x() = U256::zero();

    assert_eq!(
        Fflonk::validate_vk(&vk),
        Err(VerifyError::InvalidVerificationKey)
    );
}

mod reject {

    use super::*;

    #[test]
    fn invalid_pubs() {
        let vk = cdk_key();
        let mut invalid_pubs = VALID_PUBS;
        invalid_pubs[0] = invalid_pubs[0].wrapping_add(1);

        assert_eq!(
            Fflonk::verify_proof(&vk, &VALID_PROOF, &invalid_pubs),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn invalid_proof() {
        let vk = cdk_key();
        let mut invalid_proof: Proof = VALID_PROOF;
        // last byte changed from '0x06' to '0x00' (public inputs)
        invalid_proof[invalid_proof.len() - 1] = 0x00;

        assert_eq!(
            Fflonk::verify_proof(&vk, &invalid_proof, &VALID_PUBS),
            Err(VerifyError::VerifyError)
        );
    }

    #[test]
    fn reject_malformed_proof() {
        let vk = cdk_key();
        let mut malformed_proof: Proof = VALID_PROOF;
        // first byte changed from '0x17' to '0x07' (raw proof data)
        malformed_proof[0] = 0x07;

        assert_eq!(
            Fflonk::verify_proof(&vk, &malformed_proof, &VALID_PUBS),
            Err(VerifyError::InvalidProofData)
        );
    }
}
