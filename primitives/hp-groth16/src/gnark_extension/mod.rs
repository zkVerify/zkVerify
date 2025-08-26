mod data_structures;

#[cfg(feature = "implementation")]
pub use implementation::*;

#[cfg(feature = "implementation")]
mod implementation {
    use crate::{data_structures::Scalar, gnark_extension::data_structures::*, Groth16Error};

    use alloc::vec::Vec;
    use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
    use ark_ff::{
        field_hashers::{DefaultFieldHasher, HashToField},
        Field, PrimeField,
    };
    use ark_groth16::prepare_verifying_key;
    use ark_serialize::CanonicalSerialize;
    use ark_std::ops::AddAssign;

    const COMMITMENT_DST: &[u8] = b"bsb22-commitment";
    const CHALLENGE_DST: &[u8] = b"G16-BSB22";

    /// Verify a groth16 proof against the `E` elliptic curve using the provided verification key and inputs.
    pub fn verify_proof<E: Pairing>(
        vk: VerificationKey,
        proof: Proof,
        inputs: &[Scalar],
    ) -> Result<bool, Groth16Error> {
        // Initialize field hasher. This follows https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-5.2
        // Note: gnark leaves freedom in the choice of the hash_to_field function, but sets the above as default.
        // We need to double check if someone intends to use a different one.
        let h =
            <DefaultFieldHasher<sha2::Sha256> as HashToField<E::ScalarField>>::new(COMMITMENT_DST);
        let mut challenge = Vec::new();
        let mut inputs_committed = Vec::new();

        // Iterate over all the commitments
        vk.public_and_commitment_committed
            .iter()
            .enumerate()
            .try_for_each(|(i, comm)| {
                let mut to_hash = Vec::new();

                // Add commitment to the hasher
                to_hash.extend_from_slice(&proof.commitments[i].0);

                // Iterate over all the public inputs this commitment refers to
                comm.iter().for_each(|pub_index| {
                    // Add public input to the hasher
                    to_hash.extend_from_slice(&inputs[(pub_index - 1) as usize].0);
                });

                // Compute hash result
                let hash_result: E::ScalarField = h.hash_to_field(&to_hash, 1)[0];

                // Append result to public inputs
                inputs_committed.push(hash_result);

                let mut hash_result_raw = Vec::new();
                hash_result
                    .serialize_uncompressed(&mut hash_result_raw)
                    .map_err(|_| Groth16Error::VerifyError)?;
                // In gnark they are serialized in big-endian, while arkworks uses little-endian.
                // CHECK ENDIANESS ! AS WE KNOW SERIALIZATION CHANGES FOR DIFFERENT CURVES
                hash_result_raw.reverse();

                // Append result
                challenge.extend_from_slice(&hash_result_raw);

                Ok(())
            })?;
        
        // Parse proof into arkworks format
        let proof: ArkProof<E> = proof.try_into().map_err(|_| Groth16Error::InvalidProof)?;

        // Parse vk into arkworks format
        let vk: ArkVerificationKey<E> = vk
            .try_into_ark_unchecked()
            .map_err(|_| Groth16Error::InvalidVerificationKey)?;

        // Parse inputs into arkworks format
        let mut inputs = inputs
            .iter()
            .map(|v| v.clone().try_into_scalar::<E::ScalarField>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Groth16Error::InvalidInput)?;

        // Append committed inputs
        inputs.extend_from_slice(&inputs_committed);

        // Verify proof of knowledge of the commitments
        if !vk.commitment_keys.is_empty() {
            // In gnark this hasher is, instead, exactly https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-5.2
            let h = <DefaultFieldHasher<sha2::Sha256> as HashToField<E::ScalarField>>::new(
                CHALLENGE_DST,
            );
            let challenge = h.hash_to_field(&challenge, 1)[0];
            if !verify_pok::<E>(
                &vk.commitment_keys,
                &proof.commitments,
                proof.commitments_pok.clone(),
                challenge,
            )? {
                return Ok(false);
            }
        }

        // Like in Groth16
        let pvk = prepare_verifying_key::<E>(&vk.vk);
        let mut prepared_inputs = ark_groth16::Groth16::<E>::prepare_inputs(&pvk, &inputs)
            .map_err(|_| Groth16Error::VerifyError)?;

        // Add commitments to the prepared_inputs
        proof
            .commitments
            .iter()
            .for_each(|comm| prepared_inputs.add_assign(&comm.into_group()));

        ark_groth16::Groth16::<E>::verify_proof_with_prepared_inputs(
            &pvk,
            &proof.proof,
            &prepared_inputs,
        )
        .map_err(|_| Groth16Error::VerifyError)
    }

    // Verifies multiple separate proofs of knowledge using n+1 pairings instead of 2n pairings.
    // Note: the gnark function is called "BatchVerifyMultiVk" andtakes a Vec<pok>, specifying that the prover can fold the proofs by
    // itself and passing only a single element. The reality is that this is always done on the prover
    // side (as Proof data structure only contains a single element for pok), thus the code below
    // has been massively simplified compared to the gnark version.
    fn verify_pok<E: Pairing>(
        vk: &[ArkPedersenVerificationKey<E>],
        commitments: &[E::G1Affine],
        pok: E::G1Affine,
        challenge: E::ScalarField,
    ) -> Result<bool, Groth16Error> {
        if commitments.len() != vk.len() {
            return Err(Groth16Error::InvalidVerificationKey);
        }

        let mut pairing_g1: Vec<E::G1Prepared> = vec![commitments[0].into()];
        let mut pairing_g2: Vec<E::G2Prepared> = vec![vk[0].g_sigma_neg.into()];
        let mut r = challenge;

        vk.iter()
            .zip(commitments.iter())
            .skip(1)
            .for_each(|(vk, comm)| {
                pairing_g1.push(
                    comm.mul_bigint(&challenge.into_bigint())
                        .into_affine()
                        .into(),
                );
                pairing_g2.push(vk.g_sigma_neg.into());
                r *= challenge;
            });

        pairing_g1.push(pok.into());
        pairing_g2.push(vk[0].g.into());

        let test = E::final_exponentiation(E::multi_miller_loop(pairing_g1, pairing_g2))
            .ok_or(Groth16Error::VerifyError)?;

        Ok(test.0 == E::TargetField::ONE)
    }

    /// Verify a groth16 verification key against the `E` elliptic curve.
    pub fn validate_key<E: Pairing>(vk: VerificationKey) -> Result<(), Groth16Error> {
        ArkVerificationKey::<E>::try_from(vk)
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
    // #[case::bn254(PhantomData::<Bn254>)]
    #[case::bls12_381(PhantomData::<Bls12_381>)]
    fn curves<P: Pairing>(#[case] _p: P) {}

    mod verify_proof {
        use super::*;
        include!("resources.rs");

        #[apply(curves)]
        fn succeed<E: Pairing>(#[case] _p: PhantomData<E>) {

            let result = crate::gnark_extension::verify_proof::<E>(verification_key_bls12_381(), proof_bls12_381(), &pubs_bls12_381());
            println!("{:?}", result);
        }

        // #[apply(curves)]
        // fn fail_with_wrong_vk<E: Pairing>(#[case] _p: PhantomData<E>) {
        //     let (proof, _, inputs) = dummy_circuit::get_instance::<E>(10, Some(0));
        //     let (_, vk, _) = dummy_circuit::get_instance::<E>(10, Some(42));

        //     assert!(!verify_proof::<E>(vk, proof, &inputs).unwrap())
        // }

        // #[apply(curves)]
        // fn fail_with_wrong_inputs<E: Pairing>(#[case] _p: PhantomData<E>) {
        //     let (proof, vk, _) = dummy_circuit::get_instance::<E>(10, Some(0));
        //     let (_, _, inputs) = dummy_circuit::get_instance::<E>(10, Some(42));

        //     assert!(!verify_proof::<E>(vk, proof, &inputs).unwrap())
        // }

        // #[apply(curves)]
        // fn fail_with_wrong_proof<E: Pairing>(#[case] _p: PhantomData<E>) {
        //     let (_, vk, inputs) = dummy_circuit::get_instance::<E>(10, Some(0));
        //     let (proof, _, _) = dummy_circuit::get_instance::<E>(10, Some(42));

        //     assert!(!verify_proof::<E>(vk, proof, &inputs).unwrap())
        // }

        // #[apply(curves)]
        // fn fail_with_malformed_proof<E: Pairing>(#[case] _p: PhantomData<E>) {
        //     let (mut proof, vk, inputs) = dummy_circuit::get_instance::<E>(10, None);
        //     proof.a.0[0] += 1;

        //     assert_eq!(
        //         verify_proof::<E>(vk, proof, &inputs).err().unwrap(),
        //         Groth16Error::InvalidProof
        //     )
        // }

        // #[apply(curves)]
        // fn fail_with_malformed_inputs<E: Pairing>(#[case] _p: PhantomData<E>) {
        //     let (proof, vk, mut inputs) = dummy_circuit::get_instance::<E>(10, None);
        //     // tamper input so that it overflows scalar modulus
        //     for v in &mut inputs {
        //         for byte in &mut v.0 {
        //             *byte = 0xff;
        //         }
        //     }

        //     assert_eq!(
        //         verify_proof::<E>(vk, proof, &inputs).err().unwrap(),
        //         Groth16Error::InvalidInput
        //     )
        // }

        // #[apply(curves)]
        // fn fail_with_too_many_inputs<E: Pairing>(#[case] _p: PhantomData<E>) {
        //     let (proof, vk, mut inputs) = dummy_circuit::get_instance::<E>(10, None);
        //     inputs.push(Scalar::try_from_scalar(E::ScalarField::one()).unwrap());

        //     assert_eq!(
        //         verify_proof::<E>(vk, proof, &inputs).err().unwrap(),
        //         Groth16Error::VerifyError
        //     )
        // }

        // #[apply(curves)]
        // fn fail_with_too_few_inputs<E: Pairing>(#[case] _p: PhantomData<E>) {
        //     let (proof, vk, mut inputs) = dummy_circuit::get_instance::<E>(10, None);
        //     inputs.pop();

        //     assert_eq!(
        //         verify_proof::<E>(vk, proof, &inputs).err().unwrap(),
        //         Groth16Error::VerifyError
        //     )
        // }
    }

    // mod validate_key {
    //     use super::*;

    //     #[apply(curves)]
    //     fn accept_valid_vk<E: Pairing>(#[case] _p: PhantomData<E>) {
    //         let (_, vk, _) = dummy_circuit::get_instance::<E>(1, Some(0));

    //         assert!(validate_key::<E>(vk).is_ok());
    //     }

    //     #[apply(curves)]
    //     fn reject_malformed_vk<E: Pairing>(#[case] _p: PhantomData<E>) {
    //         let (_, mut vk, _) = dummy_circuit::get_instance::<E>(1, Some(0));
    //         vk.alpha_g1.0[0] += 1;

    //         assert_eq!(
    //             validate_key::<E>(vk),
    //             Err(Groth16Error::InvalidVerificationKey)
    //         );
    //     }
    // }
}
