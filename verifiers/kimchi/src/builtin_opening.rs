// Copyright 2026, Horizen Labs, Inc.
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

//! IPA opening verification backed by the native fixed Vesta16 SRS MSM.

use alloc::{vec, vec::Vec};

use ark_ff::{One, UniformRand, Zero};
#[cfg(feature = "std")]
use ark_poly::EvaluationDomain;
use kimchi::groupmap::GroupMap;
use mina_curves::pasta::{Fp, Fq, Vesta};
use mina_poseidon::{pasta::FULL_ROUNDS, sponge::ScalarChallenge, FqSponge};
use poly_commitment::{
    commitment::{
        b_poly, b_poly_coefficients, combine_commitments, shift_scalar, BatchEvaluationProof,
        CommitmentCurve,
    },
    ipa::{endos, OpeningProof},
    OpenProof, SRS,
};
#[cfg(feature = "std")]
use poly_commitment::{utils::DensePolynomialOrEvaluations, PolyComm};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::builtin_srs::BuiltinSrs;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuiltinOpeningProof(OpeningProof<Vesta, FULL_ROUNDS>);

impl BuiltinOpeningProof {
    pub(crate) fn rounds(&self) -> usize {
        self.0.lr.len()
    }

    #[cfg(test)]
    pub(crate) fn clear_rounds_for_test(&mut self) {
        self.0.lr.clear();
    }

    #[cfg(test)]
    pub(crate) fn duplicate_round_for_test(&mut self) {
        if let Some(round) = self.0.lr.last().copied() {
            self.0.lr.push(round);
        }
    }

    #[cfg(test)]
    pub(crate) fn corrupt_z1_for_test(&mut self) {
        self.0.z1 += Fp::one();
    }
}

impl OpenProof<Vesta, FULL_ROUNDS> for BuiltinOpeningProof {
    type SRS = BuiltinSrs;

    #[cfg(feature = "std")]
    fn open<EFqSponge, RNG, D: EvaluationDomain<Fp>>(
        _srs: &Self::SRS,
        _group_map: &<Vesta as poly_commitment::commitment::CommitmentCurve>::Map,
        _plnms: &[(DensePolynomialOrEvaluations<'_, Fp, D>, PolyComm<Fp>)],
        _elm: &[Fp],
        _polyscale: Fp,
        _evalscale: Fp,
        _sponge: EFqSponge,
        _rng: &mut RNG,
    ) -> Self
    where
        EFqSponge: Clone + FqSponge<Fq, Vesta, Fp, FULL_ROUNDS>,
        RNG: RngCore + CryptoRng,
    {
        panic!("BuiltinOpeningProof cannot be used for proving")
    }

    fn verify<EFqSponge, RNG>(
        srs: &Self::SRS,
        group_map: &<Vesta as poly_commitment::commitment::CommitmentCurve>::Map,
        batch: &mut [BatchEvaluationProof<Vesta, EFqSponge, Self, FULL_ROUNDS>],
        rng: &mut RNG,
    ) -> bool
    where
        EFqSponge: FqSponge<Fq, Vesta, Fp, FULL_ROUNDS>,
        RNG: RngCore + CryptoRng,
    {
        verify_with_native_srs_msm(srs, group_map, batch, rng)
    }
}

#[allow(clippy::too_many_lines)]
fn verify_with_native_srs_msm<EFqSponge, RNG>(
    srs: &BuiltinSrs,
    group_map: &<Vesta as poly_commitment::commitment::CommitmentCurve>::Map,
    batch: &mut [BatchEvaluationProof<Vesta, EFqSponge, BuiltinOpeningProof, FULL_ROUNDS>],
    rng: &mut RNG,
) -> bool
where
    EFqSponge: FqSponge<Fq, Vesta, Fp, FULL_ROUNDS>,
    RNG: RngCore + CryptoRng,
{
    let srs_len = srs.max_poly_size();
    if srs_len == 0 || !srs_len.is_power_of_two() {
        return false;
    }
    let expected_rounds = srs_len.trailing_zeros() as usize;
    if batch
        .iter()
        .any(|proof| proof.opening.0.lr.len() != expected_rounds)
    {
        return false;
    }

    let (_, endo_r) = endos::<Vesta>();
    let mut h_scalar = Fp::zero();
    let mut g_scalars = vec![Fp::zero(); srs_len];
    let mut extra_bases = Vec::new();
    let mut extra_scalars = Vec::new();

    let rand_base = Fp::rand(rng);
    let sg_rand_base = Fp::rand(rng);
    let mut rand_base_i = Fp::one();
    let mut sg_rand_base_i = Fp::one();

    for BatchEvaluationProof {
        sponge,
        evaluation_points,
        polyscale,
        evalscale,
        evaluations,
        opening,
        combined_inner_product,
    } in batch.iter_mut()
    {
        let opening = &opening.0;
        sponge.absorb_fr(&[shift_scalar::<Vesta>(*combined_inner_product)]);

        let u_base = {
            let t = sponge.challenge_fq();
            let (x, y) = group_map.to_group(t);
            Vesta::of_coordinates(x, y)
        };

        let challenges = opening.challenges::<EFqSponge>(&endo_r, sponge);
        sponge.absorb_g(&[opening.delta]);
        let c = ScalarChallenge::new(sponge.challenge()).to_field(&endo_r);

        let b0 = {
            let mut scale = Fp::one();
            let mut result = Fp::zero();
            for &evaluation_point in evaluation_points.iter() {
                result += scale * b_poly(&challenges.chal, evaluation_point);
                scale *= *evalscale;
            }
            result
        };

        let s = b_poly_coefficients(&challenges.chal);
        if s.len() > g_scalars.len() {
            return false;
        }

        let neg_rand_base_i = -rand_base_i;
        extra_bases.push(opening.sg);
        extra_scalars.push(neg_rand_base_i * opening.z1 - sg_rand_base_i);

        for (scalar, term) in g_scalars.iter_mut().zip(s) {
            *scalar += sg_rand_base_i * term;
        }

        h_scalar -= rand_base_i * opening.z2;

        extra_scalars.push(neg_rand_base_i * (opening.z1 * b0));
        extra_bases.push(u_base);

        let rand_base_i_c_i = c * rand_base_i;
        for ((l, r), (u_inv, u)) in opening
            .lr
            .iter()
            .zip(challenges.chal_inv.iter().zip(challenges.chal.iter()))
        {
            extra_bases.push(*l);
            extra_scalars.push(rand_base_i_c_i * u_inv);
            extra_bases.push(*r);
            extra_scalars.push(rand_base_i_c_i * u);
        }

        combine_commitments(
            evaluations,
            &mut extra_scalars,
            &mut extra_bases,
            *polyscale,
            rand_base_i_c_i,
        );

        extra_scalars.push(rand_base_i_c_i * *combined_inner_product);
        extra_bases.push(u_base);
        extra_scalars.push(rand_base_i);
        extra_bases.push(opening.delta);

        rand_base_i *= rand_base;
        sg_rand_base_i *= sg_rand_base;
    }

    let Ok(srs_len) = u32::try_from(srs_len) else {
        return false;
    };
    let result = match srs.profile() {
        Some(crate::KimchiProfileId::Vesta16) => native::vesta::vesta16_srs_msm(
            srs_len,
            h_scalar,
            &g_scalars,
            &extra_bases,
            &extra_scalars,
        ),
        None => return false,
    };

    result.is_ok_and(|point| point.is_zero())
}
