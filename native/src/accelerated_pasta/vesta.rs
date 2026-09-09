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
//!
//! # Compatibility and upgrades
//!
//! `Vesta16` pins the Vesta IPA SRS to `2^16` generators, together with the
//! supported domain/chunk envelope and authenticated parameter digests. These
//! are native node compatibility parameters and must not be changed in place.
//! Supporting a different SRS, curve, domain envelope, or parameter set
//! requires a distinct, versioned verifier/profile path and a coordinated node
//! and runtime upgrade; it cannot be delivered by a runtime-only upgrade.

extern crate alloc;

use alloc::vec::Vec;
use mina_curves::pasta::{Fp, ProjectiveVesta, Vesta};
use sp_runtime_interface::pass_by::{AllocateAndReturnByCodec, PassFatPointerAndRead};
use sp_runtime_interface::runtime_interface;
#[cfg(feature = "std")]
use std::path::Path;

use crate::arkworks_utils as utils;
#[cfg(feature = "std")]
use crate::VerifyError;

/// Base-two logarithm of the built-in Vesta16 SRS size.
pub const VESTA16_SRS_LOG2_SIZE: u8 = 16;
/// Number of bases in the built-in Vesta16 SRS.
pub const VESTA16_SRS_SIZE: usize = 1 << VESTA16_SRS_LOG2_SIZE;
/// Maximum number of polynomial commitment chunks supported by the Vesta host functions.
pub const VESTA16_MAX_CHUNKS: usize = 4;
/// Base-two logarithm of the smallest supported Vesta evaluation domain.
pub const VESTA16_MIN_DOMAIN_LOG2_SIZE: u8 = 3;
/// Base-two logarithm of the largest supported Vesta evaluation domain.
pub const VESTA16_MAX_DOMAIN_LOG2_SIZE: u8 = 18;
/// Smallest Vesta evaluation domain supported by the native parameters.
pub const VESTA16_MIN_DOMAIN_SIZE: usize = 1 << VESTA16_MIN_DOMAIN_LOG2_SIZE;
/// Largest Vesta evaluation domain supported by the native parameters.
pub const VESTA16_MAX_DOMAIN_SIZE: usize = 1 << VESTA16_MAX_DOMAIN_LOG2_SIZE;

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

/// Result of preparing all Vesta16 verifier parameters.
#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Vesta16PrewarmStatus {
    /// All precomputed Lagrange bases were loaded from a validated local cache.
    Loaded,
    /// The parameters were generated and persisted to the local cache.
    Generated,
    /// The parameters were generated, but the local cache could not be persisted.
    GeneratedWithoutCache,
}

/// Builds and validates every supported Vesta16 Lagrange basis before proof
/// verification can encounter a first-use cost.
///
/// A validated cache is loaded from `cache_root` when available. Missing or
/// invalid cache data is regenerated from the built-in SRS.
#[cfg(feature = "std")]
pub fn prewarm_vesta16_srs(cache_root: &Path) -> Result<Vesta16PrewarmStatus, VerifyError> {
    native_builtin_srs::prewarm(cache_root)
}

/// Native interfaces for Vesta verifier parameters.
#[runtime_interface]
pub trait HostCalls {
    /// Built-in Vesta16 SRS blinding commitment.
    ///
    /// - returns: `ArkScale<Vesta>`
    #[allow(clippy::result_unit_err)]
    fn vesta16_blinding_commitment() -> AllocateAndReturnByCodec<Result<Vec<u8>, ()>> {
        native_builtin_srs::blinding_commitment()
    }

    /// Chunk-aware built-in Vesta16 Lagrange-basis commitment prefix.
    ///
    /// - returns: `ArkScale<Vec<Vec<Vesta>>>`
    #[allow(clippy::result_unit_err)]
    fn vesta16_lagrange_basis_prefix(
        max_poly_size: u32,
        domain_log2: u8,
        count: u32,
    ) -> AllocateAndReturnByCodec<Result<Vec<u8>, ()>> {
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
        h_scalar: PassFatPointerAndRead<&[u8]>,
        g_scalars: PassFatPointerAndRead<&[u8]>,
        extra_bases: PassFatPointerAndRead<&[u8]>,
        extra_scalars: PassFatPointerAndRead<&[u8]>,
    ) -> AllocateAndReturnByCodec<Result<Vec<u8>, ()>> {
        native_builtin_srs::srs_msm(srs_len, h_scalar, g_scalars, extra_bases, extra_scalars)
    }
}

#[cfg(feature = "std")]
mod native_builtin_srs {
    use std::{
        fs::{self, File, OpenOptions},
        io::{self, BufReader, BufWriter, Read, Write},
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicU8, Ordering},
            Mutex, OnceLock,
        },
    };

    use ark_ec::VariableBaseMSM;
    use ark_scale::ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use blake2::{digest::consts::U32, Blake2b, Digest};
    use poly_commitment::{ipa::SRS as IpaSrs, PolyComm, SRS as _};

    use super::*;

    type Blake2b256 = Blake2b<U32>;

    // Uncompressed points make cache loading substantially faster. Each cached point is
    // structurally validated and every complete basis is authenticated against its baked
    // uncompressed-point digest before it enters the SRS cache.
    const CACHE_FILE_NAME: &str = "kimchi-vesta16-v1.cache";
    const CACHE_MAGIC: &[u8; 16] = b"ZKVKIMCHIVESTA1\0";
    const CACHE_FORMAT_VERSION: u32 = 1;
    const VESTA_UNCOMPRESSED_POINT_SIZE: usize = 65;
    const PREWARM_PREFIX_CHECK_CHUNKS: usize = 2;
    const MIN_MAX_POLY_LOG2_SIZE: u8 = VESTA16_MIN_DOMAIN_LOG2_SIZE - 1;
    const LAGRANGE_BASIS_CHECK_UNKNOWN: u8 = 0;
    const LAGRANGE_BASIS_CHECK_FAILED: u8 = 1;
    const LAGRANGE_BASIS_CHECK_PASSED: u8 = 2;
    const VESTA_SRS_16_DIGEST: [u8; 32] = [
        129, 217, 167, 114, 133, 196, 91, 86, 141, 204, 189, 251, 142, 227, 172, 65, 40, 123, 158,
        68, 19, 20, 222, 153, 64, 31, 80, 185, 87, 21, 60, 234,
    ];
    const VESTA_SRS_16_LAGRANGE_DIGESTS: &[(u8, [u8; 32])] = &[
        (
            3,
            [
                188, 58, 238, 63, 194, 79, 255, 205, 130, 187, 252, 206, 126, 205, 28, 85, 222, 88,
                150, 86, 255, 98, 182, 167, 63, 57, 84, 47, 153, 240, 124, 223,
            ],
        ),
        (
            4,
            [
                79, 211, 209, 187, 3, 204, 195, 231, 196, 43, 207, 25, 238, 124, 191, 102, 5, 116,
                188, 132, 141, 131, 153, 77, 78, 113, 69, 47, 6, 57, 241, 130,
            ],
        ),
        (
            5,
            [
                18, 17, 243, 23, 44, 213, 23, 92, 149, 235, 115, 176, 34, 143, 7, 220, 245, 217,
                133, 173, 133, 42, 103, 241, 147, 105, 120, 71, 146, 64, 101, 162,
            ],
        ),
        (
            6,
            [
                81, 114, 122, 195, 187, 219, 242, 191, 220, 74, 246, 223, 205, 180, 47, 246, 125,
                65, 225, 120, 28, 98, 243, 214, 158, 213, 25, 91, 125, 179, 113, 45,
            ],
        ),
        (
            7,
            [
                143, 134, 131, 119, 216, 5, 54, 37, 141, 144, 57, 26, 199, 8, 123, 128, 223, 49,
                253, 4, 109, 219, 36, 109, 201, 178, 115, 244, 106, 245, 13, 178,
            ],
        ),
        (
            8,
            [
                126, 169, 102, 126, 150, 95, 61, 98, 188, 111, 81, 40, 203, 27, 248, 219, 239, 82,
                252, 83, 32, 217, 3, 188, 28, 73, 250, 131, 164, 72, 205, 51,
            ],
        ),
        (
            9,
            [
                243, 27, 107, 227, 100, 201, 142, 197, 68, 60, 252, 250, 162, 98, 77, 249, 7, 178,
                100, 39, 49, 33, 224, 246, 180, 130, 71, 159, 201, 58, 10, 53,
            ],
        ),
        (
            10,
            [
                87, 43, 231, 211, 155, 240, 109, 218, 95, 31, 231, 21, 167, 213, 30, 136, 19, 226,
                55, 102, 45, 253, 120, 131, 75, 23, 35, 95, 167, 69, 182, 203,
            ],
        ),
        (
            11,
            [
                195, 65, 6, 3, 129, 6, 214, 144, 99, 42, 190, 56, 97, 65, 110, 184, 80, 111, 135,
                77, 145, 207, 151, 58, 120, 225, 182, 214, 156, 217, 215, 36,
            ],
        ),
        (
            12,
            [
                70, 136, 114, 26, 127, 135, 181, 166, 198, 35, 177, 181, 251, 96, 86, 42, 156, 231,
                46, 248, 187, 16, 218, 169, 48, 123, 21, 150, 220, 9, 187, 53,
            ],
        ),
        (
            13,
            [
                163, 103, 237, 253, 120, 6, 181, 213, 210, 121, 8, 250, 72, 93, 252, 162, 0, 250,
                214, 140, 232, 222, 136, 254, 143, 68, 172, 131, 109, 249, 235, 118,
            ],
        ),
        (
            14,
            [
                26, 157, 141, 131, 46, 112, 23, 57, 139, 167, 47, 215, 56, 83, 247, 78, 202, 255,
                192, 33, 84, 244, 240, 106, 36, 109, 183, 47, 150, 2, 140, 221,
            ],
        ),
        (
            15,
            [
                119, 116, 246, 73, 57, 212, 118, 158, 58, 200, 27, 198, 182, 145, 118, 20, 113, 12,
                126, 124, 39, 140, 135, 37, 71, 13, 47, 111, 221, 148, 102, 8,
            ],
        ),
        (
            16,
            [
                205, 169, 210, 137, 45, 125, 16, 250, 135, 144, 63, 141, 64, 158, 35, 64, 166, 129,
                90, 98, 57, 178, 36, 221, 212, 22, 53, 71, 71, 91, 91, 121,
            ],
        ),
    ];
    const VESTA_SRS_16_MULTI_CHUNK_LAGRANGE_DIGESTS: &[(u8, [u8; 32])] = &[
        (
            3,
            [
                27, 104, 211, 36, 254, 56, 149, 88, 182, 125, 32, 218, 112, 10, 185, 101, 125, 225,
                19, 71, 217, 137, 128, 243, 21, 78, 83, 96, 111, 36, 88, 142,
            ],
        ),
        (
            4,
            [
                151, 95, 63, 132, 140, 254, 199, 17, 81, 179, 117, 165, 227, 255, 98, 167, 40, 126,
                46, 56, 173, 112, 145, 179, 100, 214, 144, 27, 205, 50, 90, 253,
            ],
        ),
        (
            5,
            [
                147, 139, 188, 251, 34, 64, 59, 101, 201, 88, 52, 20, 35, 146, 89, 139, 48, 28, 66,
                7, 155, 159, 136, 100, 214, 112, 241, 124, 121, 157, 60, 18,
            ],
        ),
        (
            6,
            [
                96, 74, 92, 175, 214, 140, 183, 241, 127, 247, 47, 10, 253, 52, 168, 10, 62, 34,
                47, 217, 223, 110, 11, 106, 228, 237, 125, 168, 199, 125, 116, 184,
            ],
        ),
        (
            7,
            [
                121, 83, 86, 85, 96, 229, 27, 115, 99, 131, 148, 95, 140, 246, 129, 52, 47, 125,
                37, 211, 239, 31, 88, 1, 153, 118, 5, 80, 250, 174, 17, 4,
            ],
        ),
        (
            8,
            [
                209, 225, 218, 217, 101, 181, 158, 50, 15, 142, 134, 0, 9, 141, 79, 25, 180, 19,
                94, 83, 124, 161, 49, 134, 86, 205, 126, 101, 72, 133, 143, 62,
            ],
        ),
        (
            9,
            [
                103, 240, 49, 74, 83, 115, 74, 125, 204, 9, 239, 65, 144, 178, 232, 86, 171, 182,
                45, 174, 8, 69, 216, 194, 128, 209, 191, 205, 144, 5, 251, 0,
            ],
        ),
        (
            10,
            [
                209, 68, 100, 225, 110, 128, 93, 21, 25, 157, 27, 122, 220, 102, 248, 219, 73, 58,
                199, 92, 187, 199, 215, 10, 224, 204, 189, 55, 208, 224, 200, 1,
            ],
        ),
        (
            11,
            [
                123, 42, 130, 78, 95, 226, 161, 45, 117, 107, 151, 5, 251, 172, 56, 143, 196, 36,
                56, 176, 251, 195, 203, 55, 214, 104, 9, 71, 46, 234, 168, 1,
            ],
        ),
        (
            12,
            [
                224, 96, 199, 87, 93, 120, 172, 199, 249, 199, 64, 89, 238, 15, 96, 146, 85, 129,
                27, 242, 48, 78, 218, 125, 140, 6, 191, 214, 52, 250, 239, 195,
            ],
        ),
        (
            13,
            [
                238, 180, 77, 142, 75, 133, 153, 114, 129, 4, 236, 31, 97, 185, 75, 0, 19, 7, 57,
                73, 2, 9, 255, 75, 156, 91, 187, 71, 191, 252, 9, 93,
            ],
        ),
        (
            14,
            [
                163, 240, 227, 35, 62, 103, 230, 150, 168, 178, 67, 71, 95, 144, 211, 140, 193,
                206, 49, 229, 13, 154, 243, 112, 11, 233, 8, 211, 88, 173, 181, 219,
            ],
        ),
        (
            15,
            [
                6, 155, 162, 45, 80, 177, 37, 108, 209, 38, 234, 8, 179, 109, 81, 173, 229, 252,
                162, 41, 74, 226, 248, 10, 128, 72, 70, 74, 152, 46, 121, 206,
            ],
        ),
        (
            16,
            [
                249, 34, 64, 111, 240, 209, 34, 131, 238, 205, 50, 204, 128, 100, 98, 175, 244, 95,
                172, 188, 75, 6, 141, 246, 76, 222, 115, 207, 101, 253, 211, 160,
            ],
        ),
        (
            17,
            [
                203, 199, 203, 131, 42, 113, 236, 54, 97, 154, 82, 244, 100, 160, 242, 60, 241, 35,
                162, 133, 24, 228, 172, 122, 90, 232, 5, 206, 246, 27, 64, 126,
            ],
        ),
    ];
    pub(super) const VESTA_SRS_16_FOUR_CHUNK_LAGRANGE_DIGESTS: &[(u8, [u8; 32])] = &[(
        18,
        [
            237, 159, 130, 152, 232, 168, 231, 23, 130, 196, 163, 166, 55, 239, 247, 50, 106, 121,
            38, 245, 93, 112, 1, 59, 18, 45, 192, 210, 72, 101, 76, 44,
        ],
    )];

    static VESTA16_SRS: OnceLock<IpaSrs<Vesta>> = OnceLock::new();
    static VESTA16_PREFIX_SRS: OnceLock<Vec<(usize, IpaSrs<Vesta>)>> = OnceLock::new();
    static VESTA16_PARAMETER_CHECK: OnceLock<bool> = OnceLock::new();
    // Digesting a full Lagrange basis is expensive and depends only on this shape.
    static VESTA16_LAGRANGE_BASIS_CHECKS: OnceLock<Vec<AtomicU8>> = OnceLock::new();
    static VESTA16_PREWARM_LOCK: Mutex<()> = Mutex::new(());

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct LagrangeShape {
        max_poly_size: usize,
        domain_log2_size: u8,
    }

    struct CachedLagrangeBasis {
        shape: LagrangeShape,
        basis: Vec<PolyComm<Vesta>>,
    }

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

    pub fn prewarm(cache_root: &Path) -> Result<Vesta16PrewarmStatus, VerifyError> {
        prewarm_with_shapes(cache_root, &supported_lagrange_shapes())
    }

    fn prewarm_with_shapes(
        cache_root: &Path,
        shapes: &[LagrangeShape],
    ) -> Result<Vesta16PrewarmStatus, VerifyError> {
        let _guard = VESTA16_PREWARM_LOCK
            .lock()
            .map_err(|_| VerifyError::IncompatibleParameters)?;
        checked_srs()?;

        let cache_path = cache_root.join(CACHE_FILE_NAME);
        match load_lagrange_cache(&cache_path, shapes) {
            Ok(cached_bases) => {
                install_cached_bases(cached_bases)?;
                return Ok(Vesta16PrewarmStatus::Loaded);
            }
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                log::warn!(
                    "Ignoring invalid Kimchi Vesta16 parameter cache at {}: {error}",
                    cache_path.display()
                );
            }
            Err(_) => {}
        }

        prewarm_shapes(shapes)?;
        match persist_lagrange_cache(&cache_path, shapes) {
            Ok(()) => Ok(Vesta16PrewarmStatus::Generated),
            Err(error) => {
                log::warn!(
                    "Could not persist Kimchi Vesta16 parameter cache at {}: {error}",
                    cache_path.display()
                );
                Ok(Vesta16PrewarmStatus::GeneratedWithoutCache)
            }
        }
    }

    fn supported_lagrange_shapes() -> Vec<LagrangeShape> {
        let domain_count =
            usize::from(VESTA16_MAX_DOMAIN_LOG2_SIZE - VESTA16_MIN_DOMAIN_LOG2_SIZE + 1);
        let prefix_count = usize::from(VESTA16_SRS_LOG2_SIZE - VESTA16_MIN_DOMAIN_LOG2_SIZE + 1);
        let mut shapes = Vec::with_capacity(domain_count + prefix_count);

        for domain_log2_size in VESTA16_MIN_DOMAIN_LOG2_SIZE..=VESTA16_MAX_DOMAIN_LOG2_SIZE {
            shapes.push(LagrangeShape {
                max_poly_size: VESTA16_SRS_SIZE,
                domain_log2_size,
            });

            if domain_log2_size <= VESTA16_SRS_LOG2_SIZE {
                let domain_size = 1_usize << domain_log2_size;
                shapes.push(LagrangeShape {
                    max_poly_size: domain_size / PREWARM_PREFIX_CHECK_CHUNKS,
                    domain_log2_size,
                });
            }
        }

        shapes
    }

    fn prewarm_shapes(shapes: &[LagrangeShape]) -> Result<(), VerifyError> {
        for &shape in shapes {
            let domain_size = shape_domain_size(shape)?;
            let basis = srs_for_shape(shape)?.get_lagrange_basis_from_domain_size(domain_size);
            if !cached_lagrange_basis_matches_expected(
                shape.max_poly_size,
                shape.domain_log2_size,
                basis.as_slice(),
            ) {
                return Err(VerifyError::IncompatibleParameters);
            }
        }

        Ok(())
    }

    fn install_cached_bases(cached_bases: Vec<CachedLagrangeBasis>) -> Result<(), VerifyError> {
        for CachedLagrangeBasis { shape, basis } in cached_bases {
            let domain_size = shape_domain_size(shape)?;
            let srs = srs_for_shape(shape)?;
            srs.lagrange_bases().set_once(domain_size, basis);

            let installed = srs.get_lagrange_basis_from_domain_size(domain_size);
            if !lagrange_basis_matches_expected(
                shape.max_poly_size,
                shape.domain_log2_size,
                installed.as_slice(),
            ) {
                return Err(VerifyError::IncompatibleParameters);
            }
            mark_lagrange_basis_checked(shape)?;
        }

        Ok(())
    }

    fn srs_for_shape(shape: LagrangeShape) -> Result<&'static IpaSrs<Vesta>, VerifyError> {
        let domain_size = shape_domain_size(shape)?;
        if shape.max_poly_size == 0
            || !shape.max_poly_size.is_power_of_two()
            || shape.max_poly_size > VESTA16_SRS_SIZE
            || domain_size.div_ceil(shape.max_poly_size) > VESTA16_MAX_CHUNKS
            || expected_lagrange_basis_digest(shape.max_poly_size, shape.domain_log2_size).is_none()
        {
            return Err(VerifyError::InvalidVerificationKey);
        }

        if domain_size <= shape.max_poly_size {
            checked_srs()
        } else {
            checked_srs_prefix(shape.max_poly_size)
        }
    }

    fn shape_domain_size(shape: LagrangeShape) -> Result<usize, VerifyError> {
        if !(VESTA16_MIN_DOMAIN_LOG2_SIZE..=VESTA16_MAX_DOMAIN_LOG2_SIZE)
            .contains(&shape.domain_log2_size)
        {
            return Err(VerifyError::InvalidVerificationKey);
        }

        1_usize
            .checked_shl(u32::from(shape.domain_log2_size))
            .ok_or(VerifyError::InvalidVerificationKey)
    }

    fn persist_lagrange_cache(cache_path: &Path, shapes: &[LagrangeShape]) -> io::Result<()> {
        let parent = cache_path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Vesta16 cache path has no parent directory",
            )
        })?;
        fs::create_dir_all(parent)?;

        let temporary_path = temporary_cache_path(cache_path);
        let result = write_lagrange_cache(&temporary_path, shapes)
            .and_then(|()| fs::rename(&temporary_path, cache_path));
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    fn temporary_cache_path(cache_path: &Path) -> PathBuf {
        let mut temporary_path = cache_path.as_os_str().to_owned();
        temporary_path.push(format!(".tmp-{}", std::process::id()));
        temporary_path.into()
    }

    fn write_lagrange_cache(cache_path: &Path, shapes: &[LagrangeShape]) -> io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(cache_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(CACHE_MAGIC)?;
        writer.write_all(&CACHE_FORMAT_VERSION.to_le_bytes())?;
        writer.write_all(&cache_fingerprint())?;
        write_u32(&mut writer, shapes.len())?;

        for &shape in shapes {
            let domain_size = shape_domain_size(shape).map_err(cache_parameter_error)?;
            let chunks = domain_size.div_ceil(shape.max_poly_size);
            let basis = srs_for_shape(shape)
                .map_err(cache_parameter_error)?
                .get_lagrange_basis_from_domain_size(domain_size);
            if !lagrange_basis_matches_expected(
                shape.max_poly_size,
                shape.domain_log2_size,
                basis.as_slice(),
            ) {
                return Err(invalid_cache(
                    "refusing to persist an invalid Lagrange basis",
                ));
            }

            writer.write_all(&[shape.domain_log2_size])?;
            writer.write_all(&[max_poly_log2(shape)?])?;
            write_u32(&mut writer, basis.len())?;
            write_u32(&mut writer, chunks)?;
            for commitment in basis.iter() {
                if commitment.chunks.len() != chunks {
                    return Err(invalid_cache(
                        "Lagrange commitment has an unexpected number of chunks",
                    ));
                }
                for point in &commitment.chunks {
                    point
                        .serialize_uncompressed(&mut writer)
                        .map_err(cache_serialization_error)?;
                }
            }
        }

        writer.flush()?;
        writer.get_ref().sync_all()
    }

    fn load_lagrange_cache(
        cache_path: &Path,
        shapes: &[LagrangeShape],
    ) -> io::Result<Vec<CachedLagrangeBasis>> {
        let file = File::open(cache_path)?;
        let mut reader = BufReader::new(file);
        let mut magic = [0_u8; CACHE_MAGIC.len()];
        reader.read_exact(&mut magic)?;
        if &magic != CACHE_MAGIC {
            return Err(invalid_cache("invalid Vesta16 cache magic"));
        }

        if read_u32(&mut reader)? != CACHE_FORMAT_VERSION {
            return Err(invalid_cache("unsupported Vesta16 cache format"));
        }

        let mut fingerprint = [0_u8; 32];
        reader.read_exact(&mut fingerprint)?;
        if fingerprint != cache_fingerprint() {
            return Err(invalid_cache("Vesta16 cache parameters do not match"));
        }

        if read_usize(&mut reader)? != shapes.len() {
            return Err(invalid_cache("Vesta16 cache shape count does not match"));
        }

        let mut cached_bases = Vec::with_capacity(shapes.len());
        for &shape in shapes {
            let domain_size = shape_domain_size(shape).map_err(cache_parameter_error)?;
            let chunks = domain_size.div_ceil(shape.max_poly_size);
            if read_u8(&mut reader)? != shape.domain_log2_size
                || read_u8(&mut reader)? != max_poly_log2(shape)?
                || read_usize(&mut reader)? != domain_size
                || read_usize(&mut reader)? != chunks
            {
                return Err(invalid_cache("Vesta16 cache shape metadata does not match"));
            }

            let mut basis = Vec::with_capacity(domain_size);
            for _ in 0..domain_size {
                let mut commitment_chunks = Vec::with_capacity(chunks);
                for _ in 0..chunks {
                    // Cache entries are untrusted: validate every point before authenticating the
                    // complete basis against its baked digest below.
                    let point = Vesta::deserialize_uncompressed(&mut reader)
                        .map_err(cache_serialization_error)?;
                    commitment_chunks.push(point);
                }
                basis.push(PolyComm {
                    chunks: commitment_chunks,
                });
            }

            if !lagrange_basis_matches_expected(shape.max_poly_size, shape.domain_log2_size, &basis)
            {
                return Err(invalid_cache("Vesta16 cache basis digest does not match"));
            }
            cached_bases.push(CachedLagrangeBasis { shape, basis });
        }

        let mut trailing = [0_u8; 1];
        if reader.read(&mut trailing)? != 0 {
            return Err(invalid_cache("Vesta16 cache contains trailing data"));
        }

        Ok(cached_bases)
    }

    fn cache_fingerprint() -> [u8; 32] {
        let mut hasher = Blake2b256::new();
        hasher.update(b"kimchi:v1:vesta16:local-cache");
        hasher.update(CACHE_FORMAT_VERSION.to_le_bytes());
        hasher.update([
            VESTA16_SRS_LOG2_SIZE,
            VESTA16_MIN_DOMAIN_LOG2_SIZE,
            VESTA16_MAX_DOMAIN_LOG2_SIZE,
        ]);
        hash_usize(&mut hasher, VESTA16_MAX_CHUNKS);
        hasher.update(VESTA_SRS_16_DIGEST);
        hash_digest_table(&mut hasher, 1, VESTA_SRS_16_LAGRANGE_DIGESTS);
        hash_digest_table(&mut hasher, 2, VESTA_SRS_16_MULTI_CHUNK_LAGRANGE_DIGESTS);
        hash_digest_table(&mut hasher, 4, VESTA_SRS_16_FOUR_CHUNK_LAGRANGE_DIGESTS);
        hasher.finalize().into()
    }

    fn hash_digest_table(hasher: &mut Blake2b256, chunks: u8, table: &[(u8, [u8; 32])]) {
        hasher.update([chunks]);
        hash_usize(hasher, table.len());
        for (domain_log2_size, digest) in table {
            hasher.update([*domain_log2_size]);
            hasher.update(digest);
        }
    }

    fn max_poly_log2(shape: LagrangeShape) -> io::Result<u8> {
        if shape.max_poly_size == 0 || !shape.max_poly_size.is_power_of_two() {
            return Err(invalid_cache("invalid maximum polynomial size"));
        }
        u8::try_from(shape.max_poly_size.trailing_zeros())
            .map_err(|_| invalid_cache("maximum polynomial size does not fit the cache format"))
    }

    fn write_u32(writer: &mut impl Write, value: usize) -> io::Result<()> {
        let value = u32::try_from(value)
            .map_err(|_| invalid_cache("cache value does not fit the cache format"))?;
        writer.write_all(&value.to_le_bytes())
    }

    fn read_u8(reader: &mut impl Read) -> io::Result<u8> {
        let mut value = [0_u8; 1];
        reader.read_exact(&mut value)?;
        Ok(value[0])
    }

    fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
        let mut value = [0_u8; 4];
        reader.read_exact(&mut value)?;
        Ok(u32::from_le_bytes(value))
    }

    fn read_usize(reader: &mut impl Read) -> io::Result<usize> {
        usize::try_from(read_u32(reader)?)
            .map_err(|_| invalid_cache("cache value does not fit this platform"))
    }

    fn invalid_cache(message: &'static str) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, message)
    }

    fn cache_parameter_error(error: VerifyError) -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid Vesta16 cache parameters: {error:?}"),
        )
    }

    fn cache_serialization_error(error: ark_scale::ark_serialize::SerializationError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, error)
    }

    pub fn blinding_commitment() -> Result<Vec<u8>, ()> {
        checked_srs()
            .map(|srs| utils::encode(srs.h))
            .map_err(|_| ())
    }

    pub fn lagrange_basis_prefix(
        max_poly_size: u32,
        domain_log2: u8,
        count: u32,
    ) -> Result<Vec<u8>, ()> {
        if !(VESTA16_MIN_DOMAIN_LOG2_SIZE..=VESTA16_MAX_DOMAIN_LOG2_SIZE).contains(&domain_log2) {
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
        if num_chunks > VESTA16_MAX_CHUNKS {
            return Err(());
        }

        let basis_srs = if num_chunks == 1 {
            checked_srs().map_err(|_| ())?
        } else {
            checked_srs_prefix(max_poly_size).map_err(|_| ())?
        };
        let basis = basis_srs.get_lagrange_basis_from_domain_size(domain_size);
        if !cached_lagrange_basis_matches_expected(max_poly_size, domain_log2, basis.as_slice()) {
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

    fn cached_lagrange_basis_matches_expected(
        max_poly_size: usize,
        domain_log2_size: u8,
        basis: &[poly_commitment::PolyComm<Vesta>],
    ) -> bool {
        let Some(slot) = lagrange_basis_check_slot(max_poly_size, domain_log2_size) else {
            return false;
        };

        match slot.load(Ordering::Acquire) {
            LAGRANGE_BASIS_CHECK_PASSED => return true,
            LAGRANGE_BASIS_CHECK_FAILED => return false,
            _ => {}
        }

        let matches_expected =
            lagrange_basis_matches_expected(max_poly_size, domain_log2_size, basis);
        if !matches_expected {
            slot.store(LAGRANGE_BASIS_CHECK_FAILED, Ordering::Release);
            return false;
        }

        match slot.compare_exchange(
            LAGRANGE_BASIS_CHECK_UNKNOWN,
            LAGRANGE_BASIS_CHECK_PASSED,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => true,
            Err(LAGRANGE_BASIS_CHECK_PASSED) => true,
            Err(_) => false,
        }
    }

    fn lagrange_basis_matches_expected(
        max_poly_size: usize,
        domain_log2_size: u8,
        basis: &[PolyComm<Vesta>],
    ) -> bool {
        let Some(domain_size) = 1_usize.checked_shl(u32::from(domain_log2_size)) else {
            return false;
        };
        if basis.len() != domain_size {
            return false;
        }

        let Some(expected) = expected_lagrange_basis_digest(max_poly_size, domain_log2_size) else {
            return false;
        };
        lagrange_basis_digest(domain_log2_size, basis).is_ok_and(|digest| &digest == expected)
    }

    fn mark_lagrange_basis_checked(shape: LagrangeShape) -> Result<(), VerifyError> {
        let slot = lagrange_basis_check_slot(shape.max_poly_size, shape.domain_log2_size)
            .ok_or(VerifyError::IncompatibleParameters)?;
        match slot.compare_exchange(
            LAGRANGE_BASIS_CHECK_UNKNOWN,
            LAGRANGE_BASIS_CHECK_PASSED,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) | Err(LAGRANGE_BASIS_CHECK_PASSED) => Ok(()),
            Err(_) => Err(VerifyError::IncompatibleParameters),
        }
    }

    fn expected_lagrange_basis_digest(
        max_poly_size: usize,
        domain_log2_size: u8,
    ) -> Option<&'static [u8; 32]> {
        if max_poly_size == 0 {
            return None;
        }

        let domain_size = 1_usize.checked_shl(u32::from(domain_log2_size))?;
        let num_chunks = domain_size.div_ceil(max_poly_size);
        let expected_digests = match num_chunks {
            1 => VESTA_SRS_16_LAGRANGE_DIGESTS,
            2 => VESTA_SRS_16_MULTI_CHUNK_LAGRANGE_DIGESTS,
            4 => VESTA_SRS_16_FOUR_CHUNK_LAGRANGE_DIGESTS,
            _ => return None,
        };

        expected_digests
            .iter()
            .find_map(|(log2_size, digest)| (*log2_size == domain_log2_size).then_some(digest))
    }

    fn lagrange_basis_check_slot(
        max_poly_size: usize,
        domain_log2_size: u8,
    ) -> Option<&'static AtomicU8> {
        if max_poly_size == 0
            || !max_poly_size.is_power_of_two()
            || domain_log2_size > VESTA16_MAX_DOMAIN_LOG2_SIZE
        {
            return None;
        }

        let max_poly_log2 = usize::try_from(max_poly_size.trailing_zeros()).ok()?;
        if max_poly_log2 > usize::from(VESTA16_SRS_LOG2_SIZE) {
            return None;
        }

        let stride = usize::from(VESTA16_SRS_LOG2_SIZE) + 1;
        let slot = usize::from(domain_log2_size)
            .checked_mul(stride)?
            .checked_add(max_poly_log2)?;
        VESTA16_LAGRANGE_BASIS_CHECKS
            .get_or_init(|| {
                let slots = (usize::from(VESTA16_MAX_DOMAIN_LOG2_SIZE) + 1) * stride;
                (0..slots)
                    .map(|_| AtomicU8::new(LAGRANGE_BASIS_CHECK_UNKNOWN))
                    .collect()
            })
            .get(slot)
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
    pub(super) fn prewarm_domains_for_test(
        cache_root: &Path,
        domain_log2_sizes: &[u8],
    ) -> Result<Vesta16PrewarmStatus, VerifyError> {
        if domain_log2_sizes.iter().any(|domain_log2_size| {
            !(VESTA16_MIN_DOMAIN_LOG2_SIZE..=VESTA16_MAX_DOMAIN_LOG2_SIZE)
                .contains(domain_log2_size)
        }) {
            return Err(VerifyError::InvalidVerificationKey);
        }

        let shapes = supported_lagrange_shapes()
            .into_iter()
            .filter(|shape| domain_log2_sizes.contains(&shape.domain_log2_size))
            .collect::<Vec<_>>();
        prewarm_with_shapes(cache_root, &shapes)
    }

    #[cfg(test)]
    pub(super) fn supported_shapes_for_test() -> Vec<(usize, u8)> {
        supported_lagrange_shapes()
            .into_iter()
            .map(|shape| (shape.max_poly_size, shape.domain_log2_size))
            .collect()
    }

    #[cfg(test)]
    pub(super) fn cache_path_for_test(cache_root: &Path) -> PathBuf {
        cache_root.join(CACHE_FILE_NAME)
    }

    #[cfg(test)]
    pub(super) fn first_cached_point_offset_for_test() -> usize {
        CACHE_MAGIC.len()
            + std::mem::size_of::<u32>()
            + 32
            + std::mem::size_of::<u32>()
            + 2
            + 2 * std::mem::size_of::<u32>()
    }

    #[cfg(test)]
    pub(super) fn point_digest_for_test(point: &Vesta) -> Result<[u8; 32], VerifyError> {
        let mut hasher = Blake2b256::new();
        hash_point(&mut hasher, point)?;
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

    #[cfg(test)]
    pub(super) fn lagrange_basis_check_state_for_test(
        max_poly_size: usize,
        domain_log2_size: u8,
    ) -> Option<bool> {
        match lagrange_basis_check_slot(max_poly_size, domain_log2_size)?.load(Ordering::Acquire) {
            LAGRANGE_BASIS_CHECK_PASSED => Some(true),
            LAGRANGE_BASIS_CHECK_FAILED => Some(false),
            _ => None,
        }
    }

    fn hash_usize(hasher: &mut Blake2b256, value: usize) {
        hasher.update((value as u64).to_le_bytes());
    }

    fn hash_point(hasher: &mut Blake2b256, point: &Vesta) -> Result<(), VerifyError> {
        let mut uncompressed = [0_u8; VESTA_UNCOMPRESSED_POINT_SIZE];
        if point.uncompressed_size() != uncompressed.len() {
            return Err(VerifyError::IncompatibleParameters);
        }
        point
            .serialize_uncompressed(uncompressed.as_mut_slice())
            .map_err(|_| VerifyError::IncompatibleParameters)?;
        hasher.update(uncompressed);
        Ok(())
    }

    #[cfg(test)]
    mod digest_verification {
        use super::*;

        #[test]
        #[ignore = "expensive raw-SRS parameter digest verification"]
        fn authenticated_parameter_digests_match_raw_srs() {
            let srs = IpaSrs::<Vesta>::create(VESTA16_SRS_SIZE);
            assert_eq!(
                srs_digest(&srs).expect("SRS digest computes"),
                VESTA_SRS_16_DIGEST
            );

            for &(domain_log2_size, expected) in VESTA_SRS_16_LAGRANGE_DIGESTS {
                let domain_size = 1_usize << domain_log2_size;
                let basis = srs.get_lagrange_basis_from_domain_size(domain_size);
                let digest = lagrange_basis_digest(domain_log2_size, basis.as_slice())
                    .expect("one-chunk Lagrange digest computes");
                assert_eq!(digest, expected, "one-chunk domain 2^{domain_log2_size}");
            }

            for &(domain_log2_size, expected) in VESTA_SRS_16_MULTI_CHUNK_LAGRANGE_DIGESTS {
                let domain_size = 1_usize << domain_log2_size;
                let digest = if domain_log2_size <= VESTA16_SRS_LOG2_SIZE {
                    let max_poly_size = domain_size / PREWARM_PREFIX_CHECK_CHUNKS;
                    let prefix_srs = IpaSrs::new(srs.g[..max_poly_size].to_vec(), srs.h);
                    let basis = prefix_srs.get_lagrange_basis_from_domain_size(domain_size);
                    lagrange_basis_digest(domain_log2_size, basis.as_slice())
                } else {
                    let basis = srs.get_lagrange_basis_from_domain_size(domain_size);
                    lagrange_basis_digest(domain_log2_size, basis.as_slice())
                }
                .expect("two-chunk Lagrange digest computes");
                assert_eq!(digest, expected, "two-chunk domain 2^{domain_log2_size}");
            }

            for &(domain_log2_size, expected) in VESTA_SRS_16_FOUR_CHUNK_LAGRANGE_DIGESTS {
                let domain_size = 1_usize << domain_log2_size;
                let basis = srs.get_lagrange_basis_from_domain_size(domain_size);
                let digest = lagrange_basis_digest(domain_log2_size, basis.as_slice())
                    .expect("four-chunk Lagrange digest computes");
                assert_eq!(digest, expected, "four-chunk domain 2^{domain_log2_size}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ec::{AffineRepr, VariableBaseMSM};
    use ark_ff::{One, Zero};
    use ark_scale::ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use mina_curves::pasta::Fq;
    use poly_commitment::{ipa::SRS as IpaSrs, SRS as _};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_CACHE_ID: AtomicU64 = AtomicU64::new(0);

    fn temporary_cache_root(test_name: &str) -> PathBuf {
        let id = TEST_CACHE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("zkverify-{test_name}-{}-{id}", std::process::id()))
    }

    #[test]
    fn vesta16_lagrange_basis_prefix_returns_requested_prefix() {
        let prefix =
            vesta16_lagrange_basis_prefix(1 << 16, 10, 2).expect("native basis prefix succeeds");

        assert_eq!(prefix.len(), 2);
        assert!(prefix.iter().all(|commitment| commitment.len() == 1));
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
    fn vesta16_lagrange_basis_prefix_records_verified_digest_shape() {
        let max_poly_size = 1 << 12;
        let domain_log2 = 13;

        let prefix = vesta16_lagrange_basis_prefix(max_poly_size, domain_log2, 2)
            .expect("native basis prefix succeeds");
        let prefix_again = vesta16_lagrange_basis_prefix(max_poly_size, domain_log2, 2)
            .expect("cached native basis prefix succeeds");

        assert_eq!(prefix, prefix_again);
        assert_eq!(
            native_builtin_srs::lagrange_basis_check_state_for_test(
                max_poly_size as usize,
                domain_log2
            ),
            Some(true)
        );
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
        let cache_root = temporary_cache_root("vesta16-prewarm-supported");
        let status = native_builtin_srs::prewarm_domains_for_test(&cache_root, &[3, 10])
            .expect("prewarm succeeds");
        assert_eq!(status, Vesta16PrewarmStatus::Generated);

        let status = native_builtin_srs::prewarm_domains_for_test(&cache_root, &[3, 10])
            .expect("cached prewarm succeeds");
        assert_eq!(status, Vesta16PrewarmStatus::Loaded);
        fs::remove_dir_all(cache_root).expect("test cache cleanup succeeds");
    }

    #[test]
    fn vesta16_prewarm_rebuilds_a_corrupt_cache() {
        let cache_root = temporary_cache_root("vesta16-prewarm-corrupt");
        native_builtin_srs::prewarm_domains_for_test(&cache_root, &[3])
            .expect("initial prewarm succeeds");
        let cache_path = native_builtin_srs::cache_path_for_test(&cache_root);
        let mut cache = fs::read(&cache_path).expect("reading test cache succeeds");
        let first_point_byte = cache
            .get_mut(native_builtin_srs::first_cached_point_offset_for_test())
            .expect("test cache contains a basis point");
        *first_point_byte ^= 1;
        fs::write(cache_path, cache).expect("corrupting cached basis succeeds");

        let status = native_builtin_srs::prewarm_domains_for_test(&cache_root, &[3])
            .expect("corrupt cache is rebuilt");
        assert_eq!(status, Vesta16PrewarmStatus::Generated);
        let status = native_builtin_srs::prewarm_domains_for_test(&cache_root, &[3])
            .expect("rebuilt cache loads");
        assert_eq!(status, Vesta16PrewarmStatus::Loaded);
        fs::remove_dir_all(cache_root).expect("test cache cleanup succeeds");
    }

    #[test]
    fn vesta16_prewarm_rebuilds_cache_with_off_curve_y_hidden_by_compressed_encoding() {
        let cache_root = temporary_cache_root("vesta16-prewarm-off-curve-y");
        assert_eq!(
            native_builtin_srs::prewarm_domains_for_test(&cache_root, &[3]),
            Ok(Vesta16PrewarmStatus::Generated)
        );

        let cache_path = native_builtin_srs::cache_path_for_test(&cache_root);
        let mut cache = fs::read(&cache_path).expect("reading test cache succeeds");
        let basis_offset = native_builtin_srs::first_cached_point_offset_for_test();
        let point_size = Vesta::generator().uncompressed_size();
        let (point_offset, original) = (0..(1 << 3))
            .find_map(|index| {
                let offset = basis_offset + index * point_size;
                let encoded = cache.get(offset..offset + point_size)?;
                let point = Vesta::deserialize_uncompressed(encoded).ok()?;
                (!point.is_zero()).then_some((offset, point))
            })
            .expect("small-domain basis contains a non-infinity point");

        let mut original_compressed = Vec::new();
        original
            .serialize_compressed(&mut original_compressed)
            .expect("valid point serializes");

        let invalid = (1_u64..=1024)
            .find_map(|delta| {
                let candidate = Vesta::new_unchecked(original.x, original.y + Fq::from(delta));
                if candidate.y == original.y || candidate.is_on_curve() {
                    return None;
                }

                let mut candidate_compressed = Vec::new();
                candidate
                    .serialize_compressed(&mut candidate_compressed)
                    .expect("unchecked point serializes");
                (candidate_compressed == original_compressed).then_some(candidate)
            })
            .expect("an off-curve y with the same compressed encoding exists");

        let mut invalid_compressed = Vec::new();
        invalid
            .serialize_compressed(&mut invalid_compressed)
            .expect("unchecked point serializes");
        assert_eq!(invalid.x, original.x);
        assert_ne!(invalid.y, original.y);
        assert!(!invalid.is_on_curve());
        assert_eq!(invalid_compressed, original_compressed);
        assert_ne!(
            native_builtin_srs::point_digest_for_test(&invalid)
                .expect("invalid point digest computes"),
            native_builtin_srs::point_digest_for_test(&original)
                .expect("valid point digest computes")
        );

        let mut replacement = Vec::new();
        invalid
            .serialize_uncompressed(&mut replacement)
            .expect("unchecked point serializes");
        assert_eq!(replacement.len(), point_size);
        let unchecked = Vesta::deserialize_uncompressed_unchecked(replacement.as_slice())
            .expect("unchecked decoding accepts the off-curve point");
        assert!(!unchecked.is_on_curve());
        assert!(Vesta::deserialize_uncompressed(replacement.as_slice()).is_err());

        cache[point_offset..point_offset + point_size].copy_from_slice(&replacement);
        fs::write(&cache_path, cache).expect("corrupting cached point succeeds");

        assert_eq!(
            native_builtin_srs::prewarm_domains_for_test(&cache_root, &[3]),
            Ok(Vesta16PrewarmStatus::Generated)
        );
        assert_eq!(
            native_builtin_srs::prewarm_domains_for_test(&cache_root, &[3]),
            Ok(Vesta16PrewarmStatus::Loaded)
        );
        fs::remove_dir_all(cache_root).expect("test cache cleanup succeeds");
    }

    #[test]
    #[ignore = "expensive full consensus-parameter prewarm"]
    fn vesta16_prewarm_accepts_all_consensus_domains() {
        let cache_root = temporary_cache_root("vesta16-prewarm-all");
        assert_eq!(
            prewarm_vesta16_srs(&cache_root),
            Ok(Vesta16PrewarmStatus::Generated)
        );
        assert_eq!(
            prewarm_vesta16_srs(&cache_root),
            Ok(Vesta16PrewarmStatus::Loaded)
        );
        fs::remove_dir_all(cache_root).expect("test cache cleanup succeeds");
    }

    #[test]
    fn vesta16_prewarm_rejects_unsupported_domains() {
        let cache_root = temporary_cache_root("vesta16-prewarm-unsupported");
        assert_eq!(
            native_builtin_srs::prewarm_domains_for_test(&cache_root, &[2]),
            Err(VerifyError::InvalidVerificationKey)
        );
    }

    #[test]
    fn vesta16_prewarm_shapes_cover_all_supported_domains_without_holes() {
        let shapes = native_builtin_srs::supported_shapes_for_test();
        for domain_log2_size in VESTA16_MIN_DOMAIN_LOG2_SIZE..=VESTA16_MAX_DOMAIN_LOG2_SIZE {
            assert!(
                shapes
                    .iter()
                    .any(|(_, shape_domain)| *shape_domain == domain_log2_size),
                "missing prewarm shape for domain 2^{domain_log2_size}"
            );
        }

        assert!(shapes.iter().all(|(max_poly_size, domain_log2_size)| {
            let domain_size = 1_usize << domain_log2_size;
            domain_size.div_ceil(*max_poly_size) <= VESTA16_MAX_CHUNKS
        }));
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
