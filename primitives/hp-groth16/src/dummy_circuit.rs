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

#![cfg(any(test, feature = "dummy-circuit"))]

//! This module contains a dummy circuit for Groth16 verification.

use alloc::vec::Vec;
use ark_crypto_primitives::snark::SNARK;
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::{rand::rngs::StdRng, rand::SeedableRng, UniformRand};

use crate::{Proof, Scalar, VerificationKey};

/// A dummy circuit.
#[derive(Clone, Debug)]
pub struct DummyCircuit<F: PrimeField> {
    /// Public inputs
    pub inputs: Vec<F>,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for DummyCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        for input in self.inputs {
            let _ = cs.new_input_variable(|| Ok(input))?;
        }
        Ok(())
    }
}

/// Get an instance of a dummy circuit with the given number of inputs and random seed.
pub fn get_instance<E: Pairing>(
    num_inputs: usize,
    rng_seed: Option<u64>,
) -> (Proof, VerificationKey, Vec<Scalar>) {
    let rng = &mut StdRng::seed_from_u64(rng_seed.unwrap_or(0));

    let circuit = crate::dummy_circuit::DummyCircuit {
        inputs: (0..num_inputs).map(|_| E::ScalarField::rand(rng)).collect(),
    };

    let (pk, vk) = ark_groth16::Groth16::<E>::circuit_specific_setup(circuit.clone(), rng).unwrap();
    let proof = ark_groth16::Groth16::<E>::prove(&pk, circuit.clone(), rng).unwrap();

    let proof: Proof = proof.try_into().unwrap();
    let vk: VerificationKey = vk.try_into().unwrap();
    let inputs: Vec<Scalar> = circuit
        .inputs
        .into_iter()
        .map(Scalar::try_from_scalar)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    (proof, vk, inputs)
}
