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

//! Vesta verifier-parameter host functions.

extern crate alloc;

use alloc::vec::Vec;
use mina_curves::pasta::{Fp, ProjectiveVesta, Vesta};
use sp_runtime_interface::runtime_interface;

use crate::accelerated_bn::utils;
#[cfg(feature = "std")]
use crate::VerifyError;

const VESTA16_SRS_SIZE: usize = 1 << 16;

/// Returns the built-in Vesta16 SRS blinding commitment.
#[allow(clippy::result_unit_err)]
pub fn vesta16_blinding_commitment() -> Result<Vesta, ()> {
    let result = host_calls::vesta16_blinding_commitment()?;
    utils::decode(&result)
}

/// Returns the first `count` Lagrange-basis commitments for the built-in
/// Vesta16 SRS and the given radix-2 domain.
#[allow(clippy::result_unit_err)]
pub fn vesta16_lagrange_basis_prefix(
    max_poly_size: u32,
    domain_log2: u8,
    count: u32,
) -> Result<Vec<Vec<Vesta>>, ()> {
    let result = host_calls::vesta16_lagrange_basis_prefix(max_poly_size, domain_log2, count)?;
    utils::decode(&result)
}

/// Computes a variable-base MSM where the first terms are the built-in Vesta16
/// SRS bases: `[h, g[0], ..., g[srs_len - 1]]`.
#[allow(clippy::result_unit_err)]
pub fn vesta16_srs_msm(
    srs_len: u32,
    h_scalar: Fp,
    g_scalars: &[Fp],
    extra_bases: &[Vesta],
    extra_scalars: &[Fp],
) -> Result<ProjectiveVesta, ()> {
    let srs_len_u32 = srs_len;
    let srs_len = usize::try_from(srs_len).map_err(|_| ())?;
    if srs_len == 0
        || !srs_len.is_power_of_two()
        || srs_len > VESTA16_SRS_SIZE
        || g_scalars.len() != srs_len
        || extra_bases.len() != extra_scalars.len()
    {
        return Err(());
    }

    let h_scalar = utils::encode(h_scalar);
    let g_scalars = utils::encode(g_scalars);
    let extra_bases = utils::encode(extra_bases);
    let extra_scalars = utils::encode(extra_scalars);
    let result = host_calls::vesta16_srs_msm(
        srs_len_u32,
        &h_scalar,
        &g_scalars,
        &extra_bases,
        &extra_scalars,
    )?;
    utils::decode_proj_sw(&result)
}

/// Builds and validates the built-in Vesta16 SRS and Lagrange bases before
/// proof verification can encounter the first-use cache cost.
#[cfg(feature = "std")]
pub fn prewarm_vesta16_srs(domain_log2_sizes: &[u8]) -> Result<(), VerifyError> {
    native_builtin_srs::prewarm(domain_log2_sizes)
}

/// Native interfaces for Vesta verifier parameters.
#[runtime_interface]
pub trait HostCalls {
    /// Built-in Vesta16 SRS blinding commitment.
    ///
    /// - returns: `ArkScale<Vesta>`
    #[allow(clippy::result_unit_err)]
    fn vesta16_blinding_commitment() -> Result<Vec<u8>, ()> {
        native_builtin_srs::blinding_commitment()
    }

    /// Built-in Vesta16 Lagrange-basis commitment prefix.
    ///
    /// - returns: `ArkScale<Vec<Vesta>>`
    #[allow(clippy::result_unit_err)]
    fn vesta16_lagrange_basis_prefix(domain_log2: u8, count: u32) -> Result<Vec<u8>, ()> {
        native_builtin_srs::lagrange_basis_prefix_v1(domain_log2, count)
    }

    /// Chunk-aware built-in Vesta16 Lagrange-basis commitment prefix.
    ///
    /// - returns: `ArkScale<Vec<Vec<Vesta>>>`
    #[version(2)]
    #[allow(clippy::result_unit_err)]
    fn vesta16_lagrange_basis_prefix(
        max_poly_size: u32,
        domain_log2: u8,
        count: u32,
    ) -> Result<Vec<u8>, ()> {
        native_builtin_srs::lagrange_basis_prefix(max_poly_size, domain_log2, count)
    }

    /// Built-in Vesta16 SRS-backed variable-base MSM.
    ///
    /// - `h_scalar`: `ArkScale<Fp>`
    /// - `g_scalars`: `ArkScale<Vec<Fp>>`
    /// - `extra_bases`: `ArkScale<Vec<Vesta>>`
    /// - `extra_scalars`: `ArkScale<Vec<Fp>>`
    /// - returns: `ArkScaleProjective<ProjectiveVesta>`
    #[allow(clippy::result_unit_err)]
    fn vesta16_srs_msm(
        srs_len: u32,
        h_scalar: &[u8],
        g_scalars: &[u8],
        extra_bases: &[u8],
        extra_scalars: &[u8],
    ) -> Result<Vec<u8>, ()> {
        native_builtin_srs::srs_msm(srs_len, h_scalar, g_scalars, extra_bases, extra_scalars)
    }
}

#[cfg(feature = "std")]
mod native_builtin_srs {
    use std::sync::OnceLock;

    use ark_ec::VariableBaseMSM;
    use ark_scale::ark_serialize::CanonicalSerialize;
    use blake2::{digest::consts::U32, Blake2b, Digest};
    use poly_commitment::{ipa::SRS as IpaSrs, SRS as _};

    use super::*;

    type Blake2b256 = Blake2b<U32>;

    const MAX_CHUNKS: usize = 4;
    const PREWARM_PREFIX_CHECK_CHUNKS: usize = 2;
    const MIN_DOMAIN_LOG2_SIZE: u8 = 3;
    const MIN_MAX_POLY_LOG2_SIZE: u8 = MIN_DOMAIN_LOG2_SIZE - 1;
    const MAX_DOMAIN_LOG2_SIZE: u8 = 18;
    const V1_MIN_DOMAIN_LOG2_SIZE: u8 = 10;
    const V1_MAX_DOMAIN_LOG2_SIZE: u8 = 16;
    const VESTA_SRS_16_DIGEST: [u8; 32] = [
        118, 145, 96, 85, 33, 218, 182, 195, 62, 93, 252, 115, 198, 97, 194, 14, 79, 26, 208, 65,
        35, 3, 60, 185, 99, 21, 119, 27, 198, 4, 152, 179,
    ];
    const VESTA_SRS_16_LAGRANGE_DIGESTS: &[(u8, [u8; 32])] = &[
        (
            3,
            [
                103, 74, 217, 157, 134, 51, 119, 73, 214, 100, 89, 166, 101, 42, 239, 88, 34, 68,
                139, 145, 155, 79, 166, 248, 252, 244, 174, 167, 225, 98, 44, 159,
            ],
        ),
        (
            4,
            [
                232, 240, 71, 118, 134, 9, 247, 104, 227, 35, 195, 98, 13, 122, 70, 151, 146, 75,
                205, 240, 8, 160, 197, 83, 126, 97, 199, 135, 163, 119, 83, 27,
            ],
        ),
        (
            5,
            [
                111, 163, 158, 207, 232, 183, 37, 7, 162, 227, 38, 137, 42, 209, 213, 158, 96, 102,
                147, 178, 20, 253, 223, 253, 188, 72, 152, 246, 146, 210, 20, 86,
            ],
        ),
        (
            6,
            [
                6, 91, 23, 36, 185, 24, 154, 250, 98, 51, 47, 69, 236, 206, 132, 160, 11, 18, 141,
                144, 183, 114, 203, 74, 155, 119, 52, 46, 170, 198, 189, 14,
            ],
        ),
        (
            7,
            [
                228, 141, 33, 68, 95, 170, 101, 232, 117, 129, 49, 109, 240, 113, 223, 102, 127,
                90, 168, 89, 38, 84, 167, 162, 121, 181, 133, 26, 57, 222, 119, 187,
            ],
        ),
        (
            8,
            [
                224, 111, 210, 21, 62, 198, 84, 199, 123, 103, 161, 50, 177, 41, 246, 135, 173,
                218, 138, 189, 120, 226, 238, 3, 87, 177, 52, 171, 83, 60, 191, 54,
            ],
        ),
        (
            9,
            [
                237, 25, 139, 164, 4, 35, 84, 55, 10, 54, 138, 69, 54, 248, 120, 175, 21, 154, 170,
                15, 155, 102, 114, 192, 188, 247, 101, 98, 233, 33, 175, 25,
            ],
        ),
        (
            10,
            [
                62, 107, 180, 44, 172, 35, 75, 215, 130, 61, 38, 219, 105, 153, 182, 109, 87, 79,
                51, 75, 239, 152, 82, 100, 184, 102, 245, 0, 193, 5, 165, 179,
            ],
        ),
        (
            11,
            [
                212, 133, 142, 228, 29, 175, 56, 124, 161, 194, 216, 139, 166, 187, 85, 36, 100,
                242, 6, 208, 89, 2, 218, 229, 116, 234, 162, 168, 83, 192, 204, 218,
            ],
        ),
        (
            12,
            [
                95, 184, 34, 197, 62, 121, 185, 248, 172, 60, 7, 129, 158, 48, 206, 35, 157, 190,
                131, 54, 231, 204, 51, 154, 48, 72, 120, 152, 236, 27, 247, 165,
            ],
        ),
        (
            13,
            [
                195, 180, 144, 87, 222, 177, 101, 16, 98, 56, 222, 246, 167, 78, 234, 237, 161,
                209, 206, 221, 242, 27, 219, 88, 82, 208, 130, 42, 203, 161, 208, 226,
            ],
        ),
        (
            14,
            [
                66, 25, 87, 90, 108, 51, 238, 67, 107, 87, 228, 196, 42, 88, 198, 230, 176, 182,
                44, 249, 71, 115, 50, 1, 121, 248, 208, 53, 102, 148, 59, 215,
            ],
        ),
        (
            15,
            [
                78, 31, 93, 242, 49, 120, 194, 203, 113, 106, 52, 102, 229, 192, 84, 173, 151, 152,
                232, 187, 71, 74, 186, 58, 97, 40, 139, 166, 104, 213, 174, 110,
            ],
        ),
        (
            16,
            [
                24, 109, 49, 100, 59, 18, 111, 130, 33, 215, 232, 222, 105, 21, 21, 53, 69, 167,
                93, 234, 34, 149, 170, 1, 148, 38, 12, 195, 85, 87, 178, 80,
            ],
        ),
    ];
    const VESTA_SRS_16_MULTI_CHUNK_LAGRANGE_DIGESTS: &[(u8, [u8; 32])] = &[
        (
            3,
            [
                229, 190, 121, 214, 251, 139, 69, 151, 102, 141, 191, 190, 155, 101, 149, 136, 196,
                193, 130, 250, 118, 127, 82, 255, 246, 241, 124, 199, 184, 9, 177, 106,
            ],
        ),
        (
            4,
            [
                165, 36, 159, 45, 245, 150, 34, 175, 17, 242, 79, 92, 5, 199, 55, 9, 220, 92, 71,
                255, 173, 213, 215, 201, 83, 66, 17, 186, 73, 228, 168, 86,
            ],
        ),
        (
            5,
            [
                247, 186, 160, 139, 249, 204, 184, 190, 254, 138, 61, 209, 36, 234, 14, 106, 75,
                231, 33, 251, 48, 234, 160, 159, 179, 219, 137, 222, 35, 242, 85, 86,
            ],
        ),
        (
            6,
            [
                219, 109, 157, 64, 21, 67, 206, 64, 205, 43, 153, 255, 125, 222, 115, 219, 118, 6,
                249, 107, 234, 188, 154, 251, 108, 5, 173, 176, 165, 233, 212, 45,
            ],
        ),
        (
            7,
            [
                16, 1, 4, 180, 146, 35, 254, 200, 110, 185, 117, 55, 24, 12, 49, 215, 178, 83, 186,
                7, 83, 15, 255, 18, 100, 172, 197, 76, 71, 231, 98, 53,
            ],
        ),
        (
            8,
            [
                233, 201, 232, 95, 225, 115, 147, 91, 253, 126, 175, 68, 110, 141, 172, 231, 34,
                183, 27, 158, 55, 229, 68, 50, 117, 137, 244, 92, 137, 238, 201, 135,
            ],
        ),
        (
            9,
            [
                194, 86, 58, 61, 245, 23, 225, 111, 157, 182, 203, 19, 96, 231, 135, 5, 18, 209,
                143, 231, 96, 55, 248, 93, 39, 136, 173, 188, 52, 5, 51, 87,
            ],
        ),
        (
            10,
            [
                93, 52, 113, 184, 5, 16, 227, 120, 235, 15, 8, 109, 198, 14, 219, 49, 60, 173, 178,
                240, 29, 126, 147, 126, 1, 65, 232, 237, 58, 3, 175, 242,
            ],
        ),
        (
            11,
            [
                109, 127, 216, 27, 234, 190, 71, 167, 76, 133, 15, 119, 121, 58, 180, 147, 143, 19,
                23, 250, 168, 235, 185, 126, 13, 82, 187, 37, 170, 228, 201, 92,
            ],
        ),
        (
            12,
            [
                244, 219, 78, 87, 216, 216, 25, 220, 30, 49, 64, 220, 251, 127, 12, 255, 118, 177,
                132, 177, 247, 165, 43, 91, 10, 76, 69, 0, 74, 67, 17, 17,
            ],
        ),
        (
            13,
            [
                80, 134, 43, 85, 38, 131, 167, 202, 101, 111, 41, 175, 203, 44, 149, 73, 172, 48,
                244, 211, 104, 230, 238, 75, 173, 46, 2, 173, 34, 9, 112, 114,
            ],
        ),
        (
            14,
            [
                8, 219, 126, 74, 246, 133, 149, 188, 219, 162, 52, 200, 77, 49, 32, 29, 161, 101,
                27, 185, 188, 73, 242, 221, 25, 241, 11, 35, 231, 94, 101, 125,
            ],
        ),
        (
            15,
            [
                50, 152, 211, 120, 17, 90, 181, 111, 54, 216, 160, 237, 53, 41, 155, 199, 242, 195,
                46, 163, 208, 215, 218, 36, 229, 105, 53, 166, 113, 186, 240, 134,
            ],
        ),
        (
            16,
            [
                69, 29, 29, 208, 183, 140, 109, 111, 73, 202, 244, 104, 184, 240, 24, 205, 9, 183,
                153, 129, 95, 175, 142, 91, 210, 212, 93, 127, 248, 61, 93, 69,
            ],
        ),
        (
            17,
            [
                15, 69, 0, 239, 233, 69, 85, 43, 101, 38, 62, 24, 15, 147, 91, 50, 92, 38, 252,
                209, 224, 100, 129, 222, 138, 175, 235, 169, 76, 168, 147, 240,
            ],
        ),
    ];
    pub(super) const VESTA_SRS_16_FOUR_CHUNK_LAGRANGE_DIGESTS: &[(u8, [u8; 32])] = &[(
        18,
        [
            235, 170, 147, 63, 172, 75, 33, 247, 242, 236, 222, 107, 47, 81, 59, 137, 236, 183,
            156, 79, 162, 91, 158, 173, 199, 0, 52, 209, 199, 84, 196, 243,
        ],
    )];

    static VESTA16_SRS: OnceLock<IpaSrs<Vesta>> = OnceLock::new();
    static VESTA16_PREFIX_SRS: OnceLock<Vec<(usize, IpaSrs<Vesta>)>> = OnceLock::new();
    static VESTA16_PARAMETER_CHECK: OnceLock<bool> = OnceLock::new();

    fn srs() -> &'static IpaSrs<Vesta> {
        VESTA16_SRS.get_or_init(|| IpaSrs::<Vesta>::create(VESTA16_SRS_SIZE))
    }

    fn checked_srs() -> Result<&'static IpaSrs<Vesta>, VerifyError> {
        let srs = srs();
        let parameters_match =
            VESTA16_PARAMETER_CHECK.get_or_init(|| vesta16_srs_matches_expected(srs));
        if !*parameters_match {
            return Err(VerifyError::IncompatibleParameters);
        }

        Ok(srs)
    }

    fn checked_srs_prefix(max_poly_size: usize) -> Result<&'static IpaSrs<Vesta>, VerifyError> {
        let srs = checked_srs()?;
        if max_poly_size == VESTA16_SRS_SIZE {
            return Ok(srs);
        }

        VESTA16_PREFIX_SRS
            .get_or_init(|| {
                (MIN_MAX_POLY_LOG2_SIZE..16)
                    .map(|log2_size| {
                        let size = 1_usize << log2_size;
                        (size, IpaSrs::new(srs.g[..size].to_vec(), srs.h))
                    })
                    .collect()
            })
            .iter()
            .find_map(|(size, srs)| (*size == max_poly_size).then_some(srs))
            .ok_or(VerifyError::InvalidVerificationKey)
    }

    pub fn prewarm(domain_log2_sizes: &[u8]) -> Result<(), VerifyError> {
        if domain_log2_sizes.iter().any(|domain_log2_size| {
            !(MIN_DOMAIN_LOG2_SIZE..=MAX_DOMAIN_LOG2_SIZE).contains(domain_log2_size)
        }) {
            return Err(VerifyError::InvalidVerificationKey);
        }

        let srs = checked_srs()?;
        for &domain_log2_size in domain_log2_sizes {
            let domain_size = 1_usize
                .checked_shl(u32::from(domain_log2_size))
                .ok_or(VerifyError::InvalidVerificationKey)?;
            let basis = srs.get_lagrange_basis_from_domain_size(domain_size);
            if !lagrange_basis_matches_expected(
                VESTA16_SRS_SIZE,
                domain_log2_size,
                basis.as_slice(),
            ) {
                return Err(VerifyError::IncompatibleParameters);
            }

            if domain_size <= VESTA16_SRS_SIZE {
                let max_poly_size = domain_size / PREWARM_PREFIX_CHECK_CHUNKS;
                let basis = checked_srs_prefix(max_poly_size)?
                    .get_lagrange_basis_from_domain_size(domain_size);
                if !lagrange_basis_matches_expected(
                    max_poly_size,
                    domain_log2_size,
                    basis.as_slice(),
                ) {
                    return Err(VerifyError::IncompatibleParameters);
                }
            }
        }

        Ok(())
    }

    pub fn blinding_commitment() -> Result<Vec<u8>, ()> {
        checked_srs()
            .map(|srs| utils::encode(srs.h))
            .map_err(|_| ())
    }

    pub fn lagrange_basis_prefix_v1(domain_log2: u8, count: u32) -> Result<Vec<u8>, ()> {
        if !(V1_MIN_DOMAIN_LOG2_SIZE..=V1_MAX_DOMAIN_LOG2_SIZE).contains(&domain_log2) {
            return Err(());
        }

        let domain_size = 1_usize.checked_shl(domain_log2.into()).ok_or(())?;
        let count = usize::try_from(count).map_err(|_| ())?;
        if count > domain_size {
            return Err(());
        }

        let basis = checked_srs()
            .map_err(|_| ())?
            .get_lagrange_basis_from_domain_size(domain_size);
        let mut prefix = Vec::with_capacity(count);
        for commitment in basis.iter().take(count) {
            let [point] = commitment.chunks.as_slice() else {
                return Err(());
            };
            prefix.push(*point);
        }

        Ok(utils::encode(prefix))
    }

    pub fn lagrange_basis_prefix(
        max_poly_size: u32,
        domain_log2: u8,
        count: u32,
    ) -> Result<Vec<u8>, ()> {
        if !(MIN_DOMAIN_LOG2_SIZE..=MAX_DOMAIN_LOG2_SIZE).contains(&domain_log2) {
            return Err(());
        }

        let max_poly_size = usize::try_from(max_poly_size).map_err(|_| ())?;
        let domain_size = 1_usize.checked_shl(domain_log2.into()).ok_or(())?;
        let count = usize::try_from(count).map_err(|_| ())?;

        if max_poly_size == 0
            || !max_poly_size.is_power_of_two()
            || max_poly_size > VESTA16_SRS_SIZE
            || count > domain_size
        {
            return Err(());
        }
        let num_chunks = domain_size.div_ceil(max_poly_size);
        if num_chunks > MAX_CHUNKS {
            return Err(());
        }

        let basis_srs = if num_chunks == 1 {
            checked_srs().map_err(|_| ())?
        } else {
            checked_srs_prefix(max_poly_size).map_err(|_| ())?
        };
        let basis = basis_srs.get_lagrange_basis_from_domain_size(domain_size);
        if !lagrange_basis_matches_expected(max_poly_size, domain_log2, basis.as_slice()) {
            return Err(());
        }

        let prefix = basis
            .iter()
            .take(count)
            .map(|commitment| commitment.chunks.clone())
            .collect::<Vec<_>>();
        Ok(utils::encode(prefix))
    }

    pub fn srs_msm(
        srs_len: u32,
        h_scalar: &[u8],
        g_scalars: &[u8],
        extra_bases: &[u8],
        extra_scalars: &[u8],
    ) -> Result<Vec<u8>, ()> {
        let srs_len = usize::try_from(srs_len).map_err(|_| ())?;
        if srs_len == 0 || !srs_len.is_power_of_two() || srs_len > VESTA16_SRS_SIZE {
            return Err(());
        }

        let h_scalar = utils::decode::<Fp>(h_scalar)?;
        let g_scalars = utils::decode::<Vec<Fp>>(g_scalars)?;
        let extra_bases = utils::decode::<Vec<Vesta>>(extra_bases)?;
        let extra_scalars = utils::decode::<Vec<Fp>>(extra_scalars)?;

        if g_scalars.len() != srs_len || extra_bases.len() != extra_scalars.len() {
            return Err(());
        }

        let srs = checked_srs().map_err(|_| ())?;
        let mut bases = Vec::with_capacity(1 + srs_len + extra_bases.len());
        let mut scalars = Vec::with_capacity(1 + srs_len + extra_scalars.len());

        bases.push(srs.h);
        scalars.push(h_scalar);
        bases.extend_from_slice(&srs.g[..srs_len]);
        scalars.extend_from_slice(&g_scalars);
        bases.extend(extra_bases);
        scalars.extend(extra_scalars);

        let result = ProjectiveVesta::msm(&bases, &scalars).map_err(|_| ())?;
        Ok(utils::encode_proj_sw(&result))
    }

    fn vesta16_srs_matches_expected(srs: &IpaSrs<Vesta>) -> bool {
        srs.g.len() == VESTA16_SRS_SIZE
            && srs_digest(srs).is_ok_and(|digest| digest == VESTA_SRS_16_DIGEST)
    }

    fn lagrange_basis_matches_expected(
        max_poly_size: usize,
        domain_log2_size: u8,
        basis: &[poly_commitment::PolyComm<Vesta>],
    ) -> bool {
        let domain_size = 1_usize << domain_log2_size;
        let num_chunks = domain_size.div_ceil(max_poly_size);
        let expected_digests = match num_chunks {
            1 => VESTA_SRS_16_LAGRANGE_DIGESTS,
            2 => VESTA_SRS_16_MULTI_CHUNK_LAGRANGE_DIGESTS,
            4 => VESTA_SRS_16_FOUR_CHUNK_LAGRANGE_DIGESTS,
            _ => return false,
        };

        expected_digests
            .iter()
            .find_map(|(log2_size, digest)| (*log2_size == domain_log2_size).then_some(digest))
            .is_some_and(|expected| {
                basis.len() == 1_usize << domain_log2_size
                    && lagrange_basis_digest(domain_log2_size, basis)
                        .is_ok_and(|digest| &digest == expected)
            })
    }

    fn srs_digest(srs: &IpaSrs<Vesta>) -> Result<[u8; 32], VerifyError> {
        let mut hasher = Blake2b256::new();
        hasher.update(b"kimchi:v1:vesta16:srs");
        hash_usize(&mut hasher, srs.g.len());
        for point in &srs.g {
            hash_point(&mut hasher, point)?;
        }
        hash_point(&mut hasher, &srs.h)?;
        Ok(hasher.finalize().into())
    }

    fn lagrange_basis_digest(
        domain_log2_size: u8,
        basis: &[poly_commitment::PolyComm<Vesta>],
    ) -> Result<[u8; 32], VerifyError> {
        let mut hasher = Blake2b256::new();
        hasher.update(b"kimchi:v1:vesta16:lagrange");
        hasher.update([domain_log2_size]);
        hash_usize(&mut hasher, basis.len());
        for commitment in basis {
            hash_usize(&mut hasher, commitment.chunks.len());
            for point in &commitment.chunks {
                hash_point(&mut hasher, point)?;
            }
        }
        Ok(hasher.finalize().into())
    }

    #[cfg(test)]
    pub(super) fn lagrange_basis_digest_for_test(
        max_poly_size: usize,
        domain_log2_size: u8,
    ) -> Result<[u8; 32], VerifyError> {
        let domain_size = 1_usize
            .checked_shl(u32::from(domain_log2_size))
            .ok_or(VerifyError::InvalidVerificationKey)?;
        let basis =
            checked_srs_prefix(max_poly_size)?.get_lagrange_basis_from_domain_size(domain_size);

        lagrange_basis_digest(domain_log2_size, basis.as_slice())
    }

    fn hash_usize(hasher: &mut Blake2b256, value: usize) {
        hasher.update((value as u64).to_le_bytes());
    }

    fn hash_point(hasher: &mut Blake2b256, point: &Vesta) -> Result<(), VerifyError> {
        let mut compressed = [0_u8; 33];
        point
            .serialize_compressed(compressed.as_mut_slice())
            .map_err(|_| VerifyError::IncompatibleParameters)?;
        hasher.update(compressed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ec::{AffineRepr, VariableBaseMSM};
    use ark_ff::{One, Zero};
    use poly_commitment::{ipa::SRS as IpaSrs, SRS as _};

    #[test]
    fn vesta16_lagrange_basis_prefix_returns_requested_prefix() {
        let prefix =
            vesta16_lagrange_basis_prefix(1 << 16, 10, 2).expect("native basis prefix succeeds");

        assert_eq!(prefix.len(), 2);
        assert!(prefix.iter().all(|commitment| commitment.len() == 1));
    }

    #[test]
    fn vesta16_lagrange_basis_prefix_v1_remains_available_for_old_runtimes() {
        let encoded = native_builtin_srs::lagrange_basis_prefix_v1(10, 2)
            .expect("v1 native basis prefix succeeds");
        let prefix: Vec<Vesta> = utils::decode(&encoded).expect("v1 basis prefix decodes");

        assert_eq!(prefix.len(), 2);
        assert!(native_builtin_srs::lagrange_basis_prefix_v1(9, 2).is_err());
    }

    #[test]
    fn vesta16_lagrange_basis_prefix_rejects_unsupported_domain() {
        assert!(vesta16_lagrange_basis_prefix(1 << 16, 2, 2).is_err());
    }

    #[test]
    fn vesta16_lagrange_basis_prefix_rejects_count_larger_than_domain() {
        assert!(vesta16_lagrange_basis_prefix(1 << 16, 10, 1025).is_err());
    }

    #[test]
    fn vesta16_lagrange_basis_prefix_returns_multi_chunk_commitments() {
        let prefix =
            vesta16_lagrange_basis_prefix(1 << 9, 10, 2).expect("native basis prefix succeeds");

        assert_eq!(prefix.len(), 2);
        assert!(prefix.iter().all(|commitment| commitment.len() == 2));
    }

    #[test]
    fn vesta16_srs_msm_uses_builtin_blinding_commitment() {
        let h = vesta16_blinding_commitment().expect("native blinding commitment succeeds");
        let actual = vesta16_srs_msm(1, Fp::one(), &[Fp::zero()], &[], &[])
            .expect("native SRS MSM succeeds");

        assert_eq!(actual, h.into_group());
    }

    #[test]
    fn vesta16_srs_msm_rejects_mismatched_extra_lengths() {
        assert!(vesta16_srs_msm(1, Fp::zero(), &[Fp::zero()], &[Vesta::generator()], &[]).is_err());
    }

    #[test]
    fn vesta16_srs_msm_rejects_zero_length() {
        assert!(vesta16_srs_msm(0, Fp::zero(), &[], &[], &[]).is_err());
    }

    #[test]
    fn vesta16_srs_msm_rejects_length_larger_than_builtin_srs() {
        let oversized_len = u32::try_from(VESTA16_SRS_SIZE + 1).expect("SRS size fits u32");
        assert!(vesta16_srs_msm(oversized_len, Fp::zero(), &[], &[], &[]).is_err());
    }

    #[test]
    fn vesta16_srs_msm_rejects_non_power_of_two_length() {
        assert!(vesta16_srs_msm(3, Fp::zero(), &[Fp::zero(); 3], &[], &[]).is_err());
    }

    #[test]
    fn vesta16_srs_msm_rejects_mismatched_srs_scalar_length() {
        assert!(vesta16_srs_msm(2, Fp::zero(), &[Fp::zero()], &[], &[]).is_err());
    }

    #[test]
    fn vesta16_prewarm_accepts_supported_domains() {
        prewarm_vesta16_srs(&[3, 10, 11, 12]).expect("prewarm succeeds");
    }

    #[test]
    #[ignore = "expensive full consensus-parameter prewarm"]
    fn vesta16_prewarm_accepts_all_consensus_domains() {
        prewarm_vesta16_srs(&[3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18])
            .expect("full prewarm succeeds");
    }

    #[test]
    fn vesta16_prewarm_rejects_unsupported_domains() {
        assert_eq!(
            prewarm_vesta16_srs(&[2]),
            Err(VerifyError::InvalidVerificationKey)
        );
    }

    #[test]
    fn vesta16_srs_msm_matches_direct_srs_msm() {
        let srs_len = 8;
        let h_scalar = Fp::from(11_u64);
        let g_scalars = (0..srs_len)
            .map(|i| Fp::from((i + 1) as u64))
            .collect::<Vec<_>>();
        let extra_bases = vec![Vesta::generator(), Vesta::generator()];
        let extra_scalars = vec![Fp::from(17_u64), Fp::from(23_u64)];

        let actual = vesta16_srs_msm(
            srs_len as u32,
            h_scalar,
            &g_scalars,
            &extra_bases,
            &extra_scalars,
        )
        .expect("native SRS MSM succeeds");

        let srs = IpaSrs::<Vesta>::create(VESTA16_SRS_SIZE);
        let mut bases = vec![srs.h];
        bases.extend_from_slice(&srs.g[..srs_len]);
        bases.extend_from_slice(&extra_bases);
        let mut scalars = vec![h_scalar];
        scalars.extend_from_slice(&g_scalars);
        scalars.extend_from_slice(&extra_scalars);
        let expected = ProjectiveVesta::msm(&bases, &scalars).expect("direct SRS MSM succeeds");

        assert_eq!(actual, expected);
    }

    #[test]
    fn vesta16_four_chunk_lagrange_digest_matches_benchmark_constant() {
        let digest = native_builtin_srs::lagrange_basis_digest_for_test(1 << 16, 18)
            .expect("four-chunk digest should compute");

        assert_eq!(
            digest,
            native_builtin_srs::VESTA_SRS_16_FOUR_CHUNK_LAGRANGE_DIGESTS[0].1
        );
    }

    #[test]
    fn vesta16_lagrange_basis_prefix_returns_four_chunk_commitments() {
        let prefix =
            vesta16_lagrange_basis_prefix(1 << 16, 18, 2).expect("native basis prefix succeeds");

        assert_eq!(prefix.len(), 2);
        assert!(prefix.iter().all(|commitment| commitment.len() == 4));
    }
}
