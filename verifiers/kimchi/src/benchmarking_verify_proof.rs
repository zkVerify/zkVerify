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

#![cfg(feature = "runtime-benchmarks")]

use crate::{
    Config as VerifierConfig, Fp, Kimchi as Verifier, KimchiProfileId, Proof, Pubs, Vesta16Proof,
    Vesta16VerifierIndex, Vk, PUB_SIZE,
};
use alloc::{vec, vec::Vec};
use frame_benchmarking::v2::*;
use pallet_verifiers::benchmarking_utils;
use pallet_verifiers::traits::Verifier as _;
use poly_commitment::SRS as _;
use rand_chacha::ChaCha20Rng;

pub trait Config: crate::Config {}
pub struct Pallet<T: Config>(crate::Pallet<T>);
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

const DOMAIN_4096_PROOF: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_lookup_runtime_pubs_64/proof.bin");
const DOMAIN_4096_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_4096_maxpoly_4096_lookup_runtime_pubs_64/verifier_index.bin"
);
const DOMAIN_4096_PUBS: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_lookup_runtime_pubs_64/pubs.bin");
const DOMAIN_4096_SIMPLE_PUBS_0_PROOF: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_0/proof.bin");
const DOMAIN_4096_SIMPLE_PUBS_0_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_0/verifier_index.bin");
const DOMAIN_4096_SIMPLE_PUBS_0_PUBS: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_0/pubs.bin");
const DOMAIN_4096_SIMPLE_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_64/proof.bin");
const DOMAIN_4096_SIMPLE_PUBS_64_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_64/verifier_index.bin");
const DOMAIN_4096_SIMPLE_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_64/pubs.bin");
const DOMAIN_4096_SIMPLE_PUBS_256_PROOF: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_256/proof.bin");
const DOMAIN_4096_SIMPLE_PUBS_256_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_256/verifier_index.bin");
const DOMAIN_4096_SIMPLE_PUBS_256_PUBS: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_256/pubs.bin");
const DOMAIN_4096_SIMPLE_PUBS_1024_PROOF: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_1024/proof.bin");
const DOMAIN_4096_SIMPLE_PUBS_1024_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_1024/verifier_index.bin");
const DOMAIN_4096_SIMPLE_PUBS_1024_PUBS: &[u8] =
    include_bytes!("resources/generated_4096_maxpoly_4096_simple_pubs_1024/pubs.bin");
const DOMAIN_16384_MAXPOLY_8192_PUBS_0_PROOF: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_0/proof.bin");
const DOMAIN_16384_MAXPOLY_8192_PUBS_0_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_0/verifier_index.bin"
);
const DOMAIN_16384_MAXPOLY_8192_PUBS_0_PUBS: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_0/pubs.bin");
const DOMAIN_16384_MAXPOLY_8192_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_64/proof.bin");
const DOMAIN_16384_MAXPOLY_8192_PUBS_64_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_64/verifier_index.bin"
);
const DOMAIN_16384_MAXPOLY_8192_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_64/pubs.bin");
const DOMAIN_16384_MAXPOLY_8192_PUBS_256_PROOF: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_256/proof.bin");
const DOMAIN_16384_MAXPOLY_8192_PUBS_256_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_256/verifier_index.bin"
);
const DOMAIN_16384_MAXPOLY_8192_PUBS_256_PUBS: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_256/pubs.bin");
const DOMAIN_16384_MAXPOLY_8192_PUBS_1024_PROOF: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_1024/proof.bin");
const DOMAIN_16384_MAXPOLY_8192_PUBS_1024_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_1024/verifier_index.bin"
);
const DOMAIN_16384_MAXPOLY_8192_PUBS_1024_PUBS: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_8192_lookup_runtime_pubs_1024/pubs.bin");
const DOMAIN_16384_MAXPOLY_4096_PUBS_0_PROOF: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_0/proof.bin");
const DOMAIN_16384_MAXPOLY_4096_PUBS_0_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_0/verifier_index.bin"
);
const DOMAIN_16384_MAXPOLY_4096_PUBS_0_PUBS: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_0/pubs.bin");
const DOMAIN_16384_MAXPOLY_4096_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_64/proof.bin");
const DOMAIN_16384_MAXPOLY_4096_PUBS_64_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_64/verifier_index.bin"
);
const DOMAIN_16384_MAXPOLY_4096_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_64/pubs.bin");
const DOMAIN_16384_MAXPOLY_4096_PUBS_256_PROOF: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_256/proof.bin");
const DOMAIN_16384_MAXPOLY_4096_PUBS_256_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_256/verifier_index.bin"
);
const DOMAIN_16384_MAXPOLY_4096_PUBS_256_PUBS: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_256/pubs.bin");
const DOMAIN_16384_MAXPOLY_4096_PUBS_1024_PROOF: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_1024/proof.bin");
const DOMAIN_16384_MAXPOLY_4096_PUBS_1024_VERIFIER_INDEX: &[u8] = include_bytes!(
    "resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_1024/verifier_index.bin"
);
const DOMAIN_16384_MAXPOLY_4096_PUBS_1024_PUBS: &[u8] =
    include_bytes!("resources/generated_16384_maxpoly_4096_lookup_runtime_pubs_1024/pubs.bin");
const DOMAIN_65536_PUBS_0_PROOF: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_0/proof.bin");
const DOMAIN_65536_PUBS_0_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_0/verifier_index.bin");
const DOMAIN_65536_PUBS_0_PUBS: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_0/pubs.bin");
const DOMAIN_65536_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_64/proof.bin");
const DOMAIN_65536_PUBS_64_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_64/verifier_index.bin");
const DOMAIN_65536_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_64/pubs.bin");
const DOMAIN_65536_PUBS_256_PROOF: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_256/proof.bin");
const DOMAIN_65536_PUBS_256_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_256/verifier_index.bin");
const DOMAIN_65536_PUBS_256_PUBS: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_256/pubs.bin");
const DOMAIN_65536_PUBS_1024_PROOF: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_1024/proof.bin");
const DOMAIN_65536_PUBS_1024_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_1024/verifier_index.bin");
const DOMAIN_65536_PUBS_1024_PUBS: &[u8] =
    include_bytes!("resources/generated_65536_lookup_runtime_pubs_1024/pubs.bin");
const DOMAIN_131072_PUBS_0_PROOF: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_0/proof.bin");
const DOMAIN_131072_PUBS_0_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_0/verifier_index.bin");
const DOMAIN_131072_PUBS_0_PUBS: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_0/pubs.bin");
const DOMAIN_131072_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_64/proof.bin");
const DOMAIN_131072_PUBS_64_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_64/verifier_index.bin");
const DOMAIN_131072_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_64/pubs.bin");
const DOMAIN_131072_PUBS_256_PROOF: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_256/proof.bin");
const DOMAIN_131072_PUBS_256_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_256/verifier_index.bin");
const DOMAIN_131072_PUBS_256_PUBS: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_256/pubs.bin");
const DOMAIN_131072_PUBS_1024_PROOF: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_1024/proof.bin");
const DOMAIN_131072_PUBS_1024_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_1024/verifier_index.bin");
const DOMAIN_131072_PUBS_1024_PUBS: &[u8] =
    include_bytes!("resources/generated_131072_lookup_runtime_pubs_1024/pubs.bin");
const DOMAIN_262144_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_64/proof.bin");
const DOMAIN_262144_PUBS_64_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_64/verifier_index.bin");
const DOMAIN_262144_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_64/pubs.bin");
const DOMAIN_262144_PUBS_0_PROOF: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_0/proof.bin");
const DOMAIN_262144_PUBS_0_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_0/verifier_index.bin");
const DOMAIN_262144_PUBS_0_PUBS: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_0/pubs.bin");
const DOMAIN_262144_PUBS_1_PROOF: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1/proof.bin");
const DOMAIN_262144_PUBS_1_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1/verifier_index.bin");
const DOMAIN_262144_PUBS_1_PUBS: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1/pubs.bin");
const DOMAIN_262144_PUBS_16_PROOF: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_16/proof.bin");
const DOMAIN_262144_PUBS_16_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_16/verifier_index.bin");
const DOMAIN_262144_PUBS_16_PUBS: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_16/pubs.bin");
const DOMAIN_262144_PUBS_256_PROOF: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_256/proof.bin");
const DOMAIN_262144_PUBS_256_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_256/verifier_index.bin");
const DOMAIN_262144_PUBS_256_PUBS: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_256/pubs.bin");
const DOMAIN_262144_PUBS_1024_PROOF: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/proof.bin");
const DOMAIN_262144_PUBS_1024_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/verifier_index.bin");
const DOMAIN_262144_PUBS_1024_PUBS: &[u8] =
    include_bytes!("resources/generated_262144_lookup_runtime_pubs_1024/pubs.bin");
const DOMAIN_524288_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_524288_lookup_runtime_pubs_64/proof.bin");
const DOMAIN_524288_PUBS_64_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_524288_lookup_runtime_pubs_64/verifier_index.bin");
const DOMAIN_524288_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_524288_lookup_runtime_pubs_64/pubs.bin");
const DOMAIN_524288_PUBS_1024_PROOF: &[u8] =
    include_bytes!("resources/generated_524288_lookup_runtime_pubs_1024/proof.bin");
const DOMAIN_524288_PUBS_1024_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_524288_lookup_runtime_pubs_1024/verifier_index.bin");
const DOMAIN_524288_PUBS_1024_PUBS: &[u8] =
    include_bytes!("resources/generated_524288_lookup_runtime_pubs_1024/pubs.bin");
const DOMAIN_1048576_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_1048576_lookup_runtime_pubs_64/proof.bin");
const DOMAIN_1048576_PUBS_64_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_1048576_lookup_runtime_pubs_64/verifier_index.bin");
const DOMAIN_1048576_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_1048576_lookup_runtime_pubs_64/pubs.bin");
const FOREIGN_FIELD_MUL_PROOF: &[u8] = include_bytes!("resources/foreign_field_mul/proof.bin");
const FOREIGN_FIELD_MUL_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/foreign_field_mul/verifier_index.bin");
const FOREIGN_FIELD_MUL_PUBS: &[u8] = include_bytes!("resources/foreign_field_mul/pubs.bin");

fn benchmark_data<T: VerifierConfig>(
    proof: &[u8],
    verifier_index: &[u8],
    pubs: &[u8],
    profile: KimchiProfileId,
) -> (Proof, Vk<T>, Pubs) {
    (
        proof.to_vec(),
        Vk::new(verifier_index.to_vec(), profile),
        decode_pubs(pubs),
    )
}

fn domain_4096_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_4096_PROOF,
        DOMAIN_4096_VERIFIER_INDEX,
        DOMAIN_4096_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_4096_simple_pubs_0_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_4096_SIMPLE_PUBS_0_PROOF,
        DOMAIN_4096_SIMPLE_PUBS_0_VERIFIER_INDEX,
        DOMAIN_4096_SIMPLE_PUBS_0_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_4096_simple_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_4096_SIMPLE_PUBS_64_PROOF,
        DOMAIN_4096_SIMPLE_PUBS_64_VERIFIER_INDEX,
        DOMAIN_4096_SIMPLE_PUBS_64_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_4096_simple_pubs_256_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_4096_SIMPLE_PUBS_256_PROOF,
        DOMAIN_4096_SIMPLE_PUBS_256_VERIFIER_INDEX,
        DOMAIN_4096_SIMPLE_PUBS_256_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_4096_simple_pubs_1024_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_4096_SIMPLE_PUBS_1024_PROOF,
        DOMAIN_4096_SIMPLE_PUBS_1024_VERIFIER_INDEX,
        DOMAIN_4096_SIMPLE_PUBS_1024_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_16384_maxpoly_8192_pubs_0_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_16384_MAXPOLY_8192_PUBS_0_PROOF,
        DOMAIN_16384_MAXPOLY_8192_PUBS_0_VERIFIER_INDEX,
        DOMAIN_16384_MAXPOLY_8192_PUBS_0_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_16384_maxpoly_8192_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_16384_MAXPOLY_8192_PUBS_64_PROOF,
        DOMAIN_16384_MAXPOLY_8192_PUBS_64_VERIFIER_INDEX,
        DOMAIN_16384_MAXPOLY_8192_PUBS_64_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_16384_maxpoly_8192_pubs_256_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_16384_MAXPOLY_8192_PUBS_256_PROOF,
        DOMAIN_16384_MAXPOLY_8192_PUBS_256_VERIFIER_INDEX,
        DOMAIN_16384_MAXPOLY_8192_PUBS_256_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_16384_maxpoly_8192_pubs_1024_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_16384_MAXPOLY_8192_PUBS_1024_PROOF,
        DOMAIN_16384_MAXPOLY_8192_PUBS_1024_VERIFIER_INDEX,
        DOMAIN_16384_MAXPOLY_8192_PUBS_1024_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_16384_maxpoly_4096_pubs_0_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_16384_MAXPOLY_4096_PUBS_0_PROOF,
        DOMAIN_16384_MAXPOLY_4096_PUBS_0_VERIFIER_INDEX,
        DOMAIN_16384_MAXPOLY_4096_PUBS_0_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_16384_maxpoly_4096_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_16384_MAXPOLY_4096_PUBS_64_PROOF,
        DOMAIN_16384_MAXPOLY_4096_PUBS_64_VERIFIER_INDEX,
        DOMAIN_16384_MAXPOLY_4096_PUBS_64_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_16384_maxpoly_4096_pubs_256_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_16384_MAXPOLY_4096_PUBS_256_PROOF,
        DOMAIN_16384_MAXPOLY_4096_PUBS_256_VERIFIER_INDEX,
        DOMAIN_16384_MAXPOLY_4096_PUBS_256_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_16384_maxpoly_4096_pubs_1024_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_16384_MAXPOLY_4096_PUBS_1024_PROOF,
        DOMAIN_16384_MAXPOLY_4096_PUBS_1024_VERIFIER_INDEX,
        DOMAIN_16384_MAXPOLY_4096_PUBS_1024_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_65536_pubs_0_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_65536_PUBS_0_PROOF,
        DOMAIN_65536_PUBS_0_VERIFIER_INDEX,
        DOMAIN_65536_PUBS_0_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_65536_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_65536_PUBS_64_PROOF,
        DOMAIN_65536_PUBS_64_VERIFIER_INDEX,
        DOMAIN_65536_PUBS_64_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_65536_pubs_256_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_65536_PUBS_256_PROOF,
        DOMAIN_65536_PUBS_256_VERIFIER_INDEX,
        DOMAIN_65536_PUBS_256_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_65536_pubs_1024_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_65536_PUBS_1024_PROOF,
        DOMAIN_65536_PUBS_1024_VERIFIER_INDEX,
        DOMAIN_65536_PUBS_1024_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_131072_pubs_0_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_131072_PUBS_0_PROOF,
        DOMAIN_131072_PUBS_0_VERIFIER_INDEX,
        DOMAIN_131072_PUBS_0_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_131072_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_131072_PUBS_64_PROOF,
        DOMAIN_131072_PUBS_64_VERIFIER_INDEX,
        DOMAIN_131072_PUBS_64_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_131072_pubs_256_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_131072_PUBS_256_PROOF,
        DOMAIN_131072_PUBS_256_VERIFIER_INDEX,
        DOMAIN_131072_PUBS_256_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_131072_pubs_1024_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_131072_PUBS_1024_PROOF,
        DOMAIN_131072_PUBS_1024_VERIFIER_INDEX,
        DOMAIN_131072_PUBS_1024_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_262144_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_262144_PUBS_64_PROOF,
        DOMAIN_262144_PUBS_64_VERIFIER_INDEX,
        DOMAIN_262144_PUBS_64_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_262144_pubs_0_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_262144_PUBS_0_PROOF,
        DOMAIN_262144_PUBS_0_VERIFIER_INDEX,
        DOMAIN_262144_PUBS_0_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_262144_pubs_1_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_262144_PUBS_1_PROOF,
        DOMAIN_262144_PUBS_1_VERIFIER_INDEX,
        DOMAIN_262144_PUBS_1_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_262144_pubs_16_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_262144_PUBS_16_PROOF,
        DOMAIN_262144_PUBS_16_VERIFIER_INDEX,
        DOMAIN_262144_PUBS_16_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_262144_pubs_256_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_262144_PUBS_256_PROOF,
        DOMAIN_262144_PUBS_256_VERIFIER_INDEX,
        DOMAIN_262144_PUBS_256_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_262144_pubs_1024_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_262144_PUBS_1024_PROOF,
        DOMAIN_262144_PUBS_1024_VERIFIER_INDEX,
        DOMAIN_262144_PUBS_1024_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_524288_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_524288_PUBS_64_PROOF,
        DOMAIN_524288_PUBS_64_VERIFIER_INDEX,
        DOMAIN_524288_PUBS_64_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_524288_pubs_1024_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_524288_PUBS_1024_PROOF,
        DOMAIN_524288_PUBS_1024_VERIFIER_INDEX,
        DOMAIN_524288_PUBS_1024_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn domain_1048576_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_1048576_PUBS_64_PROOF,
        DOMAIN_1048576_PUBS_64_VERIFIER_INDEX,
        DOMAIN_1048576_PUBS_64_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn foreign_field_mul_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        FOREIGN_FIELD_MUL_PROOF,
        FOREIGN_FIELD_MUL_VERIFIER_INDEX,
        FOREIGN_FIELD_MUL_PUBS,
        KimchiProfileId::Vesta16,
    )
}

fn decode_pubs(bytes: &[u8]) -> Pubs {
    assert!(
        bytes.len().is_multiple_of(PUB_SIZE),
        "Kimchi public input fixture must be a sequence of {PUB_SIZE}-byte fields"
    );
    bytes
        .chunks_exact(PUB_SIZE)
        .map(|chunk| {
            chunk
                .try_into()
                .expect("chunks_exact always returns PUB_SIZE bytes")
        })
        .collect()
}

fn prepared_data<T: VerifierConfig>(
    data: impl FnOnce() -> (Proof, Vk<T>, Pubs),
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    let (raw_proof, vk, raw_pubs) = data();
    let proof = crate::decode_proof(&raw_proof).expect("benchmark proof should decode");
    let public_input =
        crate::decode_public_input(&raw_pubs).expect("benchmark public input should decode");
    let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
    crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        .expect("benchmark verifier index should prepare");
    let rng = crate::make_rng(&vk, &raw_proof, &raw_pubs);

    (verifier_index, proof, public_input, rng)
}

fn decoded_data<T: VerifierConfig>(
    data: impl FnOnce() -> (Proof, Vk<T>, Pubs),
) -> (KimchiProfileId, Vesta16VerifierIndex, Vesta16Proof) {
    let (raw_proof, vk, _) = data();
    let proof = crate::decode_proof(&raw_proof).expect("benchmark proof should decode");
    let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

    (vk.profile, verifier_index, proof)
}

fn prepared_domain_4096_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_4096_data::<T>)
}

fn prepared_domain_65536_pubs_64_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_65536_pubs_64_data::<T>)
}

fn prepared_domain_131072_pubs_64_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_131072_pubs_64_data::<T>)
}

fn prepared_domain_262144_pubs_64_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_262144_pubs_64_data::<T>)
}

fn prepared_domain_262144_pubs_0_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_262144_pubs_0_data::<T>)
}

fn prepared_domain_262144_pubs_1_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_262144_pubs_1_data::<T>)
}

fn prepared_domain_262144_pubs_16_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_262144_pubs_16_data::<T>)
}

fn prepared_domain_262144_pubs_256_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_262144_pubs_256_data::<T>)
}

fn prepared_domain_262144_pubs_1024_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_262144_pubs_1024_data::<T>)
}

fn prepared_domain_524288_pubs_64_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_524288_pubs_64_data::<T>)
}

fn prepared_domain_524288_pubs_1024_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_524288_pubs_1024_data::<T>)
}

fn prepared_domain_1048576_pubs_64_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_1048576_pubs_64_data::<T>)
}

fn prepared_foreign_field_mul_data<T: VerifierConfig>(
) -> (Vesta16VerifierIndex, Vesta16Proof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(foreign_field_mul_data::<T>)
}

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>)]
mod benchmarks {
    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    #[benchmark(extra)]
    fn verify_proof_domain_4096() {
        let (proof, vk, pubs) = domain_4096_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_domain_4096_pubs_0() {
        let (proof, vk, pubs) = domain_4096_simple_pubs_0_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // Public-input variants are coefficient probes. The production WeightInfo
    // exposes a single per-input coefficient, so these must not be emitted by the
    // default weight-generation pass.
    #[benchmark(extra)]
    fn verify_proof_domain_4096_pubs_64() {
        let (proof, vk, pubs) = domain_4096_simple_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_4096_pubs_256() {
        let (proof, vk, pubs) = domain_4096_simple_pubs_256_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_4096_pubs_1024() {
        let (proof, vk, pubs) = domain_4096_simple_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_16384_maxpoly_8192_pubs_0() {
        let (proof, vk, pubs) = domain_16384_maxpoly_8192_pubs_0_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_16384_maxpoly_8192_pubs_64() {
        let (proof, vk, pubs) = domain_16384_maxpoly_8192_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_16384_maxpoly_8192_pubs_256() {
        let (proof, vk, pubs) = domain_16384_maxpoly_8192_pubs_256_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_16384_maxpoly_8192_pubs_1024() {
        let (proof, vk, pubs) = domain_16384_maxpoly_8192_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_16384_maxpoly_4096_pubs_0() {
        let (proof, vk, pubs) = domain_16384_maxpoly_4096_pubs_0_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_16384_maxpoly_4096_pubs_64() {
        let (proof, vk, pubs) = domain_16384_maxpoly_4096_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_16384_maxpoly_4096_pubs_256() {
        let (proof, vk, pubs) = domain_16384_maxpoly_4096_pubs_256_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_16384_maxpoly_4096_pubs_1024() {
        let (proof, vk, pubs) = domain_16384_maxpoly_4096_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_domain_65536_pubs_0() {
        let (proof, vk, pubs) = domain_65536_pubs_0_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_65536_pubs_64() {
        let (proof, vk, pubs) = domain_65536_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_65536_pubs_256() {
        let (proof, vk, pubs) = domain_65536_pubs_256_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_65536_pubs_1024() {
        let (proof, vk, pubs) = domain_65536_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_domain_131072_pubs_0() {
        let (proof, vk, pubs) = domain_131072_pubs_0_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_131072_pubs_64() {
        let (proof, vk, pubs) = domain_131072_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_131072_pubs_256() {
        let (proof, vk, pubs) = domain_131072_pubs_256_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_131072_pubs_1024() {
        let (proof, vk, pubs) = domain_131072_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_262144_pubs_64() {
        let (proof, vk, pubs) = domain_262144_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_domain_262144_pubs_0() {
        let (proof, vk, pubs) = domain_262144_pubs_0_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_262144_pubs_1() {
        let (proof, vk, pubs) = domain_262144_pubs_1_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_262144_pubs_16() {
        let (proof, vk, pubs) = domain_262144_pubs_16_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_262144_pubs_256() {
        let (proof, vk, pubs) = domain_262144_pubs_256_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_262144_pubs_1024() {
        let (proof, vk, pubs) = domain_262144_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_524288_pubs_64() {
        let (proof, vk, pubs) = domain_524288_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_524288_pubs_1024() {
        let (proof, vk, pubs) = domain_524288_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_domain_1048576_pubs_64() {
        let (proof, vk, pubs) = domain_1048576_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_proof_foreign_field_mul() {
        let (proof, vk, pubs) = foreign_field_mul_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn reject_oversized_proof() {
        let (_, vk, pubs) = domain_262144_pubs_64_data::<T>();
        let proof = vec![0_u8; T::max_proof_size() as usize + 1];

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_err());
    }

    #[benchmark(extra)]
    fn reject_max_size_malformed_proof() {
        let (_, vk, pubs) = domain_262144_pubs_64_data::<T>();
        let proof = vec![0_u8; T::max_proof_size() as usize];

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_err());
    }

    #[benchmark(extra)]
    fn reject_oversized_public_inputs() {
        let (proof, vk, _) = domain_262144_pubs_64_data::<T>();
        let pubs = vec![[0_u8; PUB_SIZE]; T::max_pubs() as usize + 1];

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_err());
    }

    #[benchmark(extra)]
    fn reject_65536_public_inputs() {
        let (proof, vk, _) = domain_262144_pubs_64_data::<T>();
        let pubs = vec![[0_u8; PUB_SIZE]; 1 << 16];

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_err());
    }

    #[benchmark(extra)]
    fn reject_public_input_count_mismatch_before_prepare() {
        let (proof, vk, _) = domain_262144_pubs_64_data::<T>();
        let pubs = Vec::new();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_err());
    }

    #[benchmark(extra)]
    fn reject_verifier_index_with_trailing_bytes() {
        let (proof, mut vk, pubs) = domain_262144_pubs_64_data::<T>();
        vk.verifier_index_bytes.push(0);

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_err());
    }

    #[benchmark(extra)]
    fn decode_proof_domain_4096() {
        let proof = DOMAIN_4096_PROOF;

        let decoded;
        #[block]
        {
            decoded = crate::decode_proof(proof)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark(extra)]
    fn decode_proof_domain_65536_pubs_64() {
        let proof = DOMAIN_65536_PUBS_64_PROOF;

        let decoded;
        #[block]
        {
            decoded = crate::decode_proof(proof)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark(extra)]
    fn decode_proof_domain_262144_pubs_64() {
        let proof = DOMAIN_262144_PUBS_64_PROOF;

        let decoded;
        #[block]
        {
            decoded = crate::decode_proof(proof)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark(extra)]
    fn decode_proof_foreign_field_mul() {
        let proof = FOREIGN_FIELD_MUL_PROOF;

        let decoded;
        #[block]
        {
            decoded = crate::decode_proof(proof)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark(extra)]
    fn decode_vk_domain_4096() {
        let (_, vk, _) = domain_4096_data::<T>();

        let decoded;
        #[block]
        {
            decoded = crate::decode_vk(&vk)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark(extra)]
    fn decode_vk_domain_65536_pubs_64() {
        let (_, vk, _) = domain_65536_pubs_64_data::<T>();

        let decoded;
        #[block]
        {
            decoded = crate::decode_vk(&vk)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark(extra)]
    fn decode_vk_domain_262144_pubs_64() {
        let (_, vk, _) = domain_262144_pubs_64_data::<T>();

        let decoded;
        #[block]
        {
            decoded = crate::decode_vk(&vk)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark(extra)]
    fn decode_vk_foreign_field_mul() {
        let (_, vk, _) = foreign_field_mul_data::<T>();

        let decoded;
        #[block]
        {
            decoded = crate::decode_vk(&vk)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_4096() {
        let (_, vk, _) = domain_4096_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_65536_pubs_64() {
        let (_, vk, _) = domain_65536_pubs_64_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_262144_pubs_64() {
        let (_, vk, _) = domain_262144_pubs_64_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_262144_pubs_0() {
        let (_, vk, _) = domain_262144_pubs_0_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_262144_pubs_1() {
        let (_, vk, _) = domain_262144_pubs_1_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_262144_pubs_16() {
        let (_, vk, _) = domain_262144_pubs_16_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_262144_pubs_1024() {
        let (_, vk, _) = domain_262144_pubs_1024_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_524288_pubs_64() {
        let (_, vk, _) = domain_524288_pubs_64_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_524288_pubs_1024() {
        let (_, vk, _) = domain_524288_pubs_1024_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_domain_1048576_pubs_64() {
        let (_, vk, _) = domain_1048576_pubs_64_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn builtin_srs_foreign_field_mul() {
        let (_, vk, _) = foreign_field_mul_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.profile,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_4096() {
        let (_, vk, _) = domain_4096_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_65536_pubs_64() {
        let (_, vk, _) = domain_65536_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_262144_pubs_64() {
        let (_, vk, _) = domain_262144_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_262144_pubs_0() {
        let (_, vk, _) = domain_262144_pubs_0_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_262144_pubs_1() {
        let (_, vk, _) = domain_262144_pubs_1_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_262144_pubs_16() {
        let (_, vk, _) = domain_262144_pubs_16_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_262144_pubs_1024() {
        let (_, vk, _) = domain_262144_pubs_1024_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_524288_pubs_64() {
        let (_, vk, _) = domain_524288_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_524288_pubs_1024() {
        let (_, vk, _) = domain_524288_pubs_1024_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_domain_1048576_pubs_64() {
        let (_, vk, _) = domain_1048576_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn prepare_verifier_index_foreign_field_mul() {
        let (_, vk, _) = foreign_field_mul_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.profile)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark(extra)]
    fn validate_profile_domain_4096() {
        let (profile, verifier_index, proof) = decoded_data(domain_4096_data::<T>);

        let valid;
        #[block]
        {
            valid = crate::profile::validate_verifier_index(profile, &verifier_index)
                .and_then(|()| crate::profile::validate_proof(profile, &proof, &verifier_index))
        };
        assert!(valid.is_ok());
    }

    #[benchmark(extra)]
    fn validate_profile_domain_65536_pubs_64() {
        let (profile, verifier_index, proof) = decoded_data(domain_65536_pubs_64_data::<T>);

        let valid;
        #[block]
        {
            valid = crate::profile::validate_verifier_index(profile, &verifier_index)
                .and_then(|()| crate::profile::validate_proof(profile, &proof, &verifier_index))
        };
        assert!(valid.is_ok());
    }

    #[benchmark(extra)]
    fn validate_profile_domain_262144_pubs_64() {
        let (profile, verifier_index, proof) = decoded_data(domain_262144_pubs_64_data::<T>);

        let valid;
        #[block]
        {
            valid = crate::profile::validate_verifier_index(profile, &verifier_index)
                .and_then(|()| crate::profile::validate_proof(profile, &proof, &verifier_index))
        };
        assert!(valid.is_ok());
    }

    #[benchmark(extra)]
    fn validate_profile_domain_1048576_pubs_64() {
        let (profile, verifier_index, proof) = decoded_data(domain_1048576_pubs_64_data::<T>);

        let valid;
        #[block]
        {
            valid = crate::profile::validate_verifier_index(profile, &verifier_index)
                .and_then(|()| crate::profile::validate_proof(profile, &proof, &verifier_index))
        };
        assert!(valid.is_ok());
    }

    #[benchmark(extra)]
    fn validate_profile_foreign_field_mul() {
        let (profile, verifier_index, proof) = decoded_data(foreign_field_mul_data::<T>);

        let valid;
        #[block]
        {
            valid = crate::profile::validate_verifier_index(profile, &verifier_index)
                .and_then(|()| crate::profile::validate_proof(profile, &proof, &verifier_index))
        };
        assert!(valid.is_ok());
    }

    #[benchmark(extra)]
    fn lagrange_basis_domain_4096() {
        let (_, vk, _) = domain_4096_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        crate::prepare_verifier_index(&mut verifier_index, vk.profile)
            .expect("benchmark verifier index should prepare");

        let basis_len;
        #[block]
        {
            basis_len = verifier_index
                .srs()
                .get_lagrange_basis(verifier_index.domain)
                .len()
        };
        assert_eq!(basis_len, verifier_index.public);
    }

    #[benchmark(extra)]
    fn lagrange_basis_domain_65536() {
        let (_, vk, _) = domain_65536_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        crate::prepare_verifier_index(&mut verifier_index, vk.profile)
            .expect("benchmark verifier index should prepare");

        let basis_len;
        #[block]
        {
            basis_len = verifier_index
                .srs()
                .get_lagrange_basis(verifier_index.domain)
                .len()
        };
        assert_eq!(basis_len, verifier_index.public);
    }

    #[benchmark(extra)]
    fn lagrange_basis_domain_262144() {
        let (_, vk, _) = domain_262144_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        crate::prepare_verifier_index(&mut verifier_index, vk.profile)
            .expect("benchmark verifier index should prepare");

        let basis_len;
        #[block]
        {
            basis_len = verifier_index
                .srs()
                .get_lagrange_basis(verifier_index.domain)
                .len()
        };
        assert_eq!(basis_len, verifier_index.public);
    }

    #[benchmark(extra)]
    fn lagrange_basis_domain_1048576() {
        let (_, vk, _) = domain_1048576_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        crate::prepare_verifier_index(&mut verifier_index, vk.profile)
            .expect("benchmark verifier index should prepare");

        let basis_len;
        #[block]
        {
            basis_len = verifier_index
                .srs()
                .get_lagrange_basis(verifier_index.domain)
                .len()
        };
        assert_eq!(basis_len, verifier_index.public);
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_4096() {
        let (verifier_index, proof, public_input, mut rng) = prepared_domain_4096_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_65536_pubs_64() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_65536_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_131072_pubs_64() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_131072_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_262144_pubs_64() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_262144_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_262144_pubs_0() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_262144_pubs_0_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_262144_pubs_1() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_262144_pubs_1_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_262144_pubs_16() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_262144_pubs_16_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_262144_pubs_256() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_262144_pubs_256_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_262144_pubs_1024() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_262144_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_524288_pubs_64() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_524288_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_524288_pubs_1024() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_524288_pubs_1024_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_domain_1048576_pubs_64() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_1048576_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_foreign_field_mul() {
        let (verifier_index, proof, public_input, mut rng) = prepared_foreign_field_mul_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_warmed_domain_4096() {
        let (verifier_index, proof, public_input, mut rng) = prepared_domain_4096_data::<T>();

        let basis_len = verifier_index
            .srs()
            .get_lagrange_basis(verifier_index.domain)
            .len();
        assert_eq!(basis_len, verifier_index.public);

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark(extra)]
    fn verify_prepared_warmed_domain_65536_pubs_64() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_65536_pubs_64_data::<T>();

        let basis_len = verifier_index
            .srs()
            .get_lagrange_basis(verifier_index.domain)
            .len();
        assert_eq!(basis_len, verifier_index.public);

        let r;
        #[block]
        {
            r = crate::verify_vesta16_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
}

#[cfg(test)]
mod mock {
    use frame_support::{
        derive_impl, parameter_types,
        sp_runtime::{traits::IdentityLookup, BuildStorage},
        traits::{fungible::HoldConsideration, LinearStoragePrice},
    };
    use sp_core::{ConstU128, ConstU32};

    type Balance = u128;
    type AccountId = u64;

    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            Balances: pallet_balances,
            CommonVerifiersPallet: pallet_verifiers::common,
            VerifierPallet: crate,
        }
    );

    impl crate::Config for Test {
        type MaxProofSize = ConstU32<262144>;
        type MaxPubs = ConstU32<1024>;
        type MaxVkSize = ConstU32<65536>;
        type WeightInfo = ();
    }

    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlockU32<Test>;
        type AccountId = AccountId;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    parameter_types! {
        pub const BaseDeposit: Balance = 1;
        pub const PerByteDeposit: Balance = 2;
        pub const HoldReasonVkRegistration: RuntimeHoldReason = RuntimeHoldReason::CommonVerifiersPallet(pallet_verifiers::common::HoldReason::VkRegistration);
    }

    impl pallet_verifiers::Config<crate::Kimchi<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::KimchiWeight<()>;
        type Ticket = HoldConsideration<
            AccountId,
            Balances,
            HoldReasonVkRegistration,
            LinearStoragePrice<BaseDeposit, PerByteDeposit, Balance>,
        >;
        type Currency = Balances;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type WeightInfo = ();
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxLocks = ConstU32<10>;
        type MaxReserves = ConstU32<10>;
        type MaxFreezes = ConstU32<10>;
        type DoneSlashHandler = ();
    }

    impl pallet_verifiers::common::Config for Test {
        type CommonWeightInfo = Test;
    }

    pub fn test_ext() -> sp_io::TestExternalities {
        let mut ext = sp_io::TestExternalities::from(
            frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .expect("mock genesis storage should build"),
        );
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
