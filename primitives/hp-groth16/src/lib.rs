// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! Provide a base interface and the std lib implementation for groth16
//! verifier via arkworks library

mod data_structures;

/// Rexported Bls curve
pub use ark_bls12_381::Bls12_381;
/// Rexported Bn curve
pub use ark_bn254::Bn254;

pub mod dummy_circuit;
pub use data_structures::*;

#[cfg(feature = "implementation")]
pub use implementation::*;

/// Errors that can occur during groth16 verification.
#[derive(Debug, PartialEq)]
pub enum Groth16Error {
    /// Invalid proof provided.
    InvalidProof,
    /// Invalid verification key provided.
    InvalidVerificationKey,
    /// Invalid input provided.
    InvalidInput,
    /// Verification failed.
    VerifyError,
}

impl From<Groth16Error> for hp_verifiers::VerifyError {
    fn from(error: Groth16Error) -> Self {
        match error {
            Groth16Error::InvalidProof => hp_verifiers::VerifyError::InvalidProofData,
            Groth16Error::InvalidVerificationKey => {
                hp_verifiers::VerifyError::InvalidVerificationKey
            }
            Groth16Error::InvalidInput => hp_verifiers::VerifyError::InvalidInput,
            Groth16Error::VerifyError => hp_verifiers::VerifyError::VerifyError,
        }
    }
}

#[cfg(feature = "implementation")]
mod implementation {
    use crate::*;

    use ark_ec::pairing::Pairing;
    use ark_groth16::prepare_verifying_key;
    use sp_std::vec::Vec;

    /// Verify a groth16 proof against the `E` elliptic curve using the provided verification key and inputs.
    pub fn verify_proof<E: Pairing>(
        vk: VerificationKey,
        proof: Proof,
        inputs: &[Scalar],
    ) -> Result<bool, Groth16Error> {
        let proof: ark_groth16::Proof<E> =
            proof.try_into().map_err(|_| Groth16Error::InvalidProof)?;
        let vk: ark_groth16::VerifyingKey<E> = vk
            .try_into_ark_unchecked()
            .map_err(|_| Groth16Error::InvalidVerificationKey)?;
        let pvk = prepare_verifying_key::<E>(&vk);
        let inputs = inputs
            .iter()
            .map(|v| v.clone().try_into_scalar::<E::ScalarField>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Groth16Error::InvalidInput)?;
        ark_groth16::Groth16::<E>::verify_proof(&pvk, &proof, &inputs)
            .map_err(|_| Groth16Error::VerifyError)
    }

    /// Verify a groth16 verification key against the `E` elliptic curve.
    pub fn validate_key<E: Pairing>(vk: VerificationKey) -> Result<(), Groth16Error> {
        ark_groth16::VerifyingKey::<E>::try_from(vk)
            .map(|_| ())
            .map_err(|_| Groth16Error::InvalidVerificationKey)
    }
}

#[cfg(test)]
mod should {
    use core::marker::PhantomData;

    use super::*;
    use ark_bls12_381::Bls12_381;
    use ark_bn254::Bn254;
    use ark_ec::pairing::Pairing;
    use ark_ff::One;
    use rstest::rstest;
    use rstest_reuse::{apply, template};

    #[template]
    #[rstest]
    #[case::bn254(PhantomData::<Bn254>)]
    #[case::bls12_381(PhantomData::<Bls12_381>)]
    fn curves<P: Pairing>(#[case] _p: P) {}

    mod verify_proof {
        use super::*;

        #[apply(curves)]
        fn succeed<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (proof, vk, inputs) = dummy_circuit::get_instance::<E>(10, None);

            assert!(verify_proof::<E>(vk, proof, &inputs).unwrap())
        }

        #[apply(curves)]
        fn fail_with_wrong_vk<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (proof, _, inputs) = dummy_circuit::get_instance::<E>(10, Some(0));
            let (_, vk, _) = dummy_circuit::get_instance::<E>(10, Some(42));

            assert!(!verify_proof::<E>(vk, proof, &inputs).unwrap())
        }

        #[apply(curves)]
        fn fail_with_wrong_inputs<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (proof, vk, _) = dummy_circuit::get_instance::<E>(10, Some(0));
            let (_, _, inputs) = dummy_circuit::get_instance::<E>(10, Some(42));

            assert!(!verify_proof::<E>(vk, proof, &inputs).unwrap())
        }

        #[apply(curves)]
        fn fail_with_wrong_proof<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (_, vk, inputs) = dummy_circuit::get_instance::<E>(10, Some(0));
            let (proof, _, _) = dummy_circuit::get_instance::<E>(10, Some(42));

            assert!(!verify_proof::<E>(vk, proof, &inputs).unwrap())
        }

        #[apply(curves)]
        fn fail_with_malformed_proof<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (mut proof, vk, inputs) = dummy_circuit::get_instance::<E>(10, None);
            proof.a.0[0] += 1;

            assert_eq!(
                verify_proof::<E>(vk, proof, &inputs).err().unwrap(),
                Groth16Error::InvalidProof
            )
        }

        #[apply(curves)]
        fn fail_with_malformed_inputs<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (proof, vk, mut inputs) = dummy_circuit::get_instance::<E>(10, None);
            // tamper input so that it overflows scalar modulus
            for v in &mut inputs {
                for byte in &mut v.0 {
                    *byte = 0xff;
                }
            }

            assert_eq!(
                verify_proof::<E>(vk, proof, &inputs).err().unwrap(),
                Groth16Error::InvalidInput
            )
        }

        #[apply(curves)]
        fn fail_with_too_many_inputs<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (proof, vk, mut inputs) = dummy_circuit::get_instance::<E>(10, None);
            inputs.push(Scalar::try_from_scalar(E::ScalarField::one()).unwrap());

            assert_eq!(
                verify_proof::<E>(vk, proof, &inputs).err().unwrap(),
                Groth16Error::VerifyError
            )
        }

        #[apply(curves)]
        fn fail_with_too_few_inputs<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (proof, vk, mut inputs) = dummy_circuit::get_instance::<E>(10, None);
            inputs.pop();

            assert_eq!(
                verify_proof::<E>(vk, proof, &inputs).err().unwrap(),
                Groth16Error::VerifyError
            )
        }
    }

    mod validate_key {
        use super::*;

        #[apply(curves)]
        fn accept_valid_vk<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (_, vk, _) = dummy_circuit::get_instance::<E>(1, Some(0));

            assert!(validate_key::<E>(vk).is_ok());
        }

        #[apply(curves)]
        fn reject_malformed_vk<E: Pairing>(#[case] _p: PhantomData<E>) {
            let (_, mut vk, _) = dummy_circuit::get_instance::<E>(1, Some(0));
            vk.alpha_g1.0[0] += 1;

            assert_eq!(
                validate_key::<E>(vk),
                Err(Groth16Error::InvalidVerificationKey)
            );
        }
    }
}
