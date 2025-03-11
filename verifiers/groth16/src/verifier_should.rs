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

use super::*;
use rstest::rstest;
use rstest_reuse::{apply, template};

struct Mock;
impl Config for Mock {
    const MAX_NUM_INPUTS: u32 = 16;
}

#[template]
#[rstest]
fn curves(#[values(Curve::Bn254, Curve::Bls12_381)] curve: Curve) {}

#[apply(curves)]
fn validate_correct_proof(curve: Curve) {
    let (proof, vk, inputs) = groth16::Groth16::get_instance(10, None, curve);
    assert!(Groth16::<Mock>::verify_proof(&vk, &proof, &inputs).is_ok());
}

#[apply(curves)]
fn validate_correct_vk(curve: Curve) {
    let (_, vk, _) = groth16::Groth16::get_instance(10, None, curve);
    assert!(Groth16::<Mock>::validate_vk(&vk).is_ok());
}

#[apply(curves)]
#[case::no_inputs(0)]
#[case::max_number_of_inputs(Mock::MAX_NUM_INPUTS as usize)]
fn validate_proof(curve: Curve, #[case] n: usize) {
    let (proof, vk, inputs) = groth16::Groth16::get_instance(n, None, curve);
    assert!(Groth16::<Mock>::verify_proof(&vk, &proof, &inputs).is_ok());
}

mod reject {
    use hp_verifiers::VerifyError;

    use super::*;

    #[apply(curves)]
    fn incorrect_proof(curve: Curve) {
        let (proof, _, _) = groth16::Groth16::get_instance(10, Some(0), curve);
        let (_, vk, inputs) = groth16::Groth16::get_instance(10, Some(42), curve);

        assert_eq!(
            Groth16::<Mock>::verify_proof(&vk, &proof, &inputs),
            Err(VerifyError::VerifyError)
        );
    }

    #[apply(curves)]
    fn malformed_proof(curve: Curve) {
        let (mut proof, vk, inputs) = groth16::Groth16::get_instance(1, Some(0), curve);
        proof.proof.a.0[0] += 1;

        assert_eq!(
            Groth16::<Mock>::verify_proof(&vk, &proof, &inputs),
            Err(VerifyError::InvalidProofData)
        );
    }

    #[apply(curves)]
    fn malformed_inputs(curve: Curve) {
        let (proof, vk, mut inputs) = groth16::Groth16::get_instance(1, Some(0), curve);
        for byte in &mut inputs[0].0 {
            *byte = 0xff;
        }

        assert_eq!(
            Groth16::<Mock>::verify_proof(&vk, &proof, &inputs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[apply(curves)]
    fn too_many_inputs(curve: Curve) {
        let (proof, vk, inputs) =
            groth16::Groth16::get_instance(Mock::MAX_NUM_INPUTS as usize + 1, Some(0), curve);

        assert_eq!(
            Groth16::<Mock>::verify_proof(&vk, &proof, &inputs),
            Err(VerifyError::InvalidInput)
        );
    }

    #[apply(curves)]
    fn inconsistent_vk_and_num_inputs(curve: Curve) {
        let (proof, vk, _) = groth16::Groth16::get_instance(4, Some(0), curve);
        let (_, _, inputs) = groth16::Groth16::get_instance(5, Some(0), curve);

        assert_eq!(
            Groth16::<Mock>::verify_proof(&vk, &proof, &inputs),
            Err(VerifyError::InvalidInput)
        );
    }
}
