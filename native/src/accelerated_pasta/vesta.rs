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
pub fn vesta16_lagrange_basis_prefix(domain_log2: u8, count: u32) -> Result<Vec<Vesta>, ()> {
    let result = host_calls::vesta16_lagrange_basis_prefix(domain_log2, count)?;
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
        native_builtin_srs::lagrange_basis_prefix(domain_log2, count)
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

    const MIN_DOMAIN_LOG2_SIZE: u8 = 10;
    const MAX_DOMAIN_LOG2_SIZE: u8 = 16;
    const VESTA_SRS_16_DIGEST: [u8; 32] = [
        118, 145, 96, 85, 33, 218, 182, 195, 62, 93, 252, 115, 198, 97, 194, 14, 79, 26, 208, 65,
        35, 3, 60, 185, 99, 21, 119, 27, 198, 4, 152, 179,
    ];
    const VESTA_SRS_16_LAGRANGE_DIGESTS: &[(u8, [u8; 32])] = &[
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

    static VESTA16_SRS: OnceLock<IpaSrs<Vesta>> = OnceLock::new();
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
            if !lagrange_basis_matches_expected(domain_log2_size, basis.as_slice()) {
                return Err(VerifyError::IncompatibleParameters);
            }
        }

        Ok(())
    }

    pub fn blinding_commitment() -> Result<Vec<u8>, ()> {
        checked_srs()
            .map(|srs| utils::encode(srs.h))
            .map_err(|_| ())
    }

    pub fn lagrange_basis_prefix(domain_log2: u8, count: u32) -> Result<Vec<u8>, ()> {
        if !(MIN_DOMAIN_LOG2_SIZE..=MAX_DOMAIN_LOG2_SIZE).contains(&domain_log2) {
            return Err(());
        }

        let domain_size = 1_usize
            .checked_shl(domain_log2.into())
            .filter(|domain_size| *domain_size <= VESTA16_SRS_SIZE)
            .ok_or(())?;
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
        domain_log2_size: u8,
        basis: &[poly_commitment::PolyComm<Vesta>],
    ) -> bool {
        VESTA_SRS_16_LAGRANGE_DIGESTS
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
        let prefix = vesta16_lagrange_basis_prefix(10, 2).expect("native basis prefix succeeds");

        assert_eq!(prefix.len(), 2);
    }

    #[test]
    fn vesta16_lagrange_basis_prefix_rejects_unsupported_domain() {
        assert!(vesta16_lagrange_basis_prefix(9, 2).is_err());
    }

    #[test]
    fn vesta16_lagrange_basis_prefix_rejects_count_larger_than_domain() {
        assert!(vesta16_lagrange_basis_prefix(10, 1025).is_err());
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
        prewarm_vesta16_srs(&[10, 11, 12]).expect("prewarm succeeds");
    }

    #[test]
    fn vesta16_prewarm_rejects_unsupported_domains() {
        assert_eq!(
            prewarm_vesta16_srs(&[9]),
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
}
