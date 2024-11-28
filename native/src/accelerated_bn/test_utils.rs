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

use ark_ec::{
    pairing::Pairing,
    short_weierstrass::{Affine, SWCurveConfig},
    AffineRepr, CurveGroup, Group,
};

pub fn multi_pairing<P: Pairing>() -> P::TargetField {
    let g = P::G1::generator();
    let h = P::G2::generator();
    P::multi_pairing([g], [h]).0
}

pub fn msm<G: SWCurveConfig>() -> (G::BaseField, G::BaseField, bool) {
    let scalars: Vec<G::ScalarField> = vec![1_u64, 2_u64, 3_u64, 4_u64, 5_u64]
        .into_iter()
        .map(Into::into)
        .collect();

    let points = vec![G::GENERATOR; 5];

    G::msm(&points, &scalars)
        .map(|result| result.into_affine())
        .map(|Affine { x, y, infinity }| (x, y, infinity))
        .unwrap()
}

pub fn mul_projective<G: SWCurveConfig>() -> (G::BaseField, G::BaseField, bool) {
    let scalar: u64 = 123456789;
    let point = G::GENERATOR.into_group();

    let Affine { x, y, infinity } = G::mul_projective(&point, &[scalar]).into_affine();
    (x, y, infinity)
}

pub fn mul_affine<G: SWCurveConfig>() -> (G::BaseField, G::BaseField, bool) {
    let scalar: u64 = 123456789;
    let point = G::GENERATOR;

    let Affine { x, y, infinity } = G::mul_affine(&point, &[scalar]).into_affine();
    (x, y, infinity)
}
