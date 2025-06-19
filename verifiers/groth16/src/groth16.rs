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

use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use core::fmt::Debug;
use hp_verifiers::VerifyError;
use scale_info::TypeInfo;

pub use hp_groth16::{vec_max_encoded_len, Proof, Scalar, VerificationKey, G1, G2};

#[derive(Copy, Clone, Debug, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Curve {
    Bn254,
    Bls12_381,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub struct VerificationKeyWithCurve {
    pub curve: Curve,
    pub alpha_g1: G1,
    pub beta_g2: G2,
    pub gamma_g2: G2,
    pub delta_g2: G2,
    pub gamma_abc_g1: Vec<G1>,
}

impl MaxEncodedLen for VerificationKeyWithCurve {
    fn max_encoded_len() -> usize {
        Curve::max_encoded_len()
            + G1::max_encoded_len()
            + 3 * G2::max_encoded_len()
            + vec_max_encoded_len(G1::max_encoded_len(), crate::MAX_NUM_INPUTS + 1)
    }
}

impl VerificationKeyWithCurve {
    pub fn from_curve_and_vk(curve: Curve, vk: VerificationKey) -> Self {
        Self {
            curve,
            alpha_g1: vk.alpha_g1,
            beta_g2: vk.beta_g2,
            gamma_g2: vk.gamma_g2,
            delta_g2: vk.delta_g2,
            gamma_abc_g1: vk.gamma_abc_g1,
        }
    }

    pub fn vk(self) -> VerificationKey {
        VerificationKey {
            alpha_g1: self.alpha_g1,
            beta_g2: self.beta_g2,
            gamma_g2: self.gamma_g2,
            delta_g2: self.delta_g2,
            gamma_abc_g1: self.gamma_abc_g1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub struct ProofWithCurve {
    pub curve: Curve,
    pub proof: Proof,
}

impl Default for ProofWithCurve {
    fn default() -> Self {
        Self {
            curve: Curve::Bn254,
            proof: Proof {
                a: G1(Vec::default()),
                b: G2(Vec::default()),
                c: G1(Vec::default()),
            },
        }
    }
}

impl ProofWithCurve {
    pub fn new(curve: Curve, proof: Proof) -> Self {
        Self { curve, proof }
    }
}

impl From<ProofWithCurve> for Proof {
    fn from(value: ProofWithCurve) -> Self {
        value.proof
    }
}

pub struct Groth16;

impl Groth16 {
    pub fn verify_proof(
        proof: Proof,
        vk: VerificationKeyWithCurve,
        inputs: &[Scalar],
    ) -> Result<bool, VerifyError> {
        let curve = vk.curve;
        let vk = vk.vk();
        match curve {
            Curve::Bn254 => {
                native::groth_16_bn_254_verify::verify(vk, proof, inputs).map_err(Into::into)
            }
            Curve::Bls12_381 => {
                native::groth_16_bls_12_381_verify::verify(vk, proof, inputs).map_err(Into::into)
            }
        }
    }

    #[cfg(any(test, feature = "runtime-benchmarks"))]
    pub fn get_instance(
        num_inputs: usize,
        rng_seed: Option<u64>,
        curve: Curve,
    ) -> (ProofWithCurve, VerificationKeyWithCurve, Vec<Scalar>) {
        let (proof, vk, inputs) = match curve {
            Curve::Bn254 => {
                hp_groth16::dummy_circuit::get_instance::<hp_groth16::Bn254>(num_inputs, rng_seed)
            }
            Curve::Bls12_381 => hp_groth16::dummy_circuit::get_instance::<hp_groth16::Bls12_381>(
                num_inputs, rng_seed,
            ),
        };

        (
            ProofWithCurve::new(curve, proof),
            VerificationKeyWithCurve::from_curve_and_vk(curve, vk),
            inputs,
        )
    }
}
