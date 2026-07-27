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

//! Accepted Kimchi verification profiles.
//!
//! A profile is a consensus policy, not only an SRS choice. It bounds every
//! decoded field that can influence verifier work before calling into Kimchi.

use alloc::vec::Vec;
use kimchi::{
    circuits::constraints::zk_rows_strict_lower_bound,
    circuits::lookup::lookups::{LookupFeatures, LookupInfo, LookupPatterns},
    proof::{PointEvaluations, ProofEvaluations},
    verifier_index::LookupVerifierIndex,
};
use mina_curves::pasta::Fp;
use poly_commitment::PolyComm;
use static_assertions::const_assert;

use crate::{KimchiProfileId, Vesta16Proof, Vesta16VerifierIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProfileError {
    CommitmentChunks,
    EvaluationLengths,
    ProofConfiguration,
    VerifierIndexConfiguration,
}

pub(crate) trait KimchiProfile {
    const MAX_DECODED_PROOF_BYTES: usize;
    const MAX_DECODED_VK_BYTES: usize;
    const MAX_CHUNKS: usize;
    const MAX_DOMAIN_SIZE: usize;
    const MAX_LOOKUP_TABLE_WIDTH: usize;
    const MAX_POLY_SIZE: usize;
    const MAX_PREV_CHALLENGES: usize;
    const MAX_PUBLIC_INPUTS: usize;
    const MIN_DOMAIN_SIZE: usize;

    fn supports_domain_size(domain_size: usize) -> bool {
        domain_size.is_power_of_two()
            && (Self::MIN_DOMAIN_SIZE..=Self::MAX_DOMAIN_SIZE).contains(&domain_size)
    }
}

/// Production profile for externally submitted Vesta Kimchi proofs.
///
/// Generic Kimchi custom gates and lookup arguments are supported. Recursive
/// accumulators remain a Pickles concern. The chunk and public-input caps are
/// the benchmarked block-work envelope.
pub(crate) struct Vesta16;

impl KimchiProfile for Vesta16 {
    const MAX_DECODED_PROOF_BYTES: usize = 262_144;
    const MAX_DECODED_VK_BYTES: usize = 65_536;
    const MAX_CHUNKS: usize = 4;
    const MAX_DOMAIN_SIZE: usize = Self::MAX_POLY_SIZE * Self::MAX_CHUNKS;
    const MAX_LOOKUP_TABLE_WIDTH: usize = 3;
    const MAX_POLY_SIZE: usize = 1 << 16;
    const MAX_PREV_CHALLENGES: usize = 0;
    const MAX_PUBLIC_INPUTS: usize = 1024;
    const MIN_DOMAIN_SIZE: usize = 1 << 3;
}

const_assert!(Vesta16::MAX_CHUNKS <= native::vesta::VESTA16_MAX_CHUNKS);
const_assert!(Vesta16::MAX_POLY_SIZE <= native::vesta::VESTA16_SRS_SIZE);
const_assert!(Vesta16::MIN_DOMAIN_SIZE >= native::vesta::VESTA16_MIN_DOMAIN_SIZE);
const_assert!(Vesta16::MAX_DOMAIN_SIZE <= native::vesta::VESTA16_MAX_DOMAIN_SIZE);

pub(crate) fn validate_verifier_index(
    profile: KimchiProfileId,
    index: &Vesta16VerifierIndex,
) -> Result<(), ProfileError> {
    match profile {
        KimchiProfileId::Vesta16 => validate_vesta16_verifier_index(index),
    }
}

pub(crate) fn validate_proof(
    profile: KimchiProfileId,
    proof: &Vesta16Proof,
    index: &Vesta16VerifierIndex,
) -> Result<(), ProfileError> {
    match profile {
        KimchiProfileId::Vesta16 => validate_vesta16_proof(proof, index),
    }
}

fn validate_vesta16_verifier_index(index: &Vesta16VerifierIndex) -> Result<(), ProfileError> {
    let domain_size =
        usize::try_from(index.domain.size).map_err(|_| ProfileError::VerifierIndexConfiguration)?;
    let num_chunks = expected_chunks(index.max_poly_size, domain_size)
        .ok_or(ProfileError::VerifierIndexConfiguration)?;
    let expected_zk_rows = u64::try_from(zk_rows_strict_lower_bound(num_chunks) + 1)
        .map_err(|_| ProfileError::VerifierIndexConfiguration)?;

    if !Vesta16::supports_domain_size(domain_size)
        || index.max_poly_size > Vesta16::MAX_POLY_SIZE
        || !index.max_poly_size.is_power_of_two()
        || num_chunks > Vesta16::MAX_CHUNKS
        || !vesta16_supports_work_shape(index.max_poly_size, domain_size, num_chunks)
        || index.zk_rows != expected_zk_rows
        || index.public > Vesta16::MAX_PUBLIC_INPUTS
        || index.public > domain_size
        || index.prev_challenges > Vesta16::MAX_PREV_CHALLENGES
    {
        return Err(ProfileError::VerifierIndexConfiguration);
    }

    for commitment in index
        .sigma_comm
        .iter()
        .chain(index.coefficients_comm.iter())
        .chain([
            &index.generic_comm,
            &index.psm_comm,
            &index.complete_add_comm,
            &index.mul_comm,
            &index.emul_comm,
            &index.endomul_scalar_comm,
        ])
        .chain(index.range_check0_comm.iter())
        .chain(index.range_check1_comm.iter())
        .chain(index.foreign_field_add_comm.iter())
        .chain(index.foreign_field_mul_comm.iter())
        .chain(index.xor_comm.iter())
        .chain(index.rot_comm.iter())
    {
        validate_commitment(commitment, num_chunks)?;
    }

    validate_lookup_verifier_index(index, num_chunks)
}

fn vesta16_supports_work_shape(
    max_poly_size: usize,
    domain_size: usize,
    num_chunks: usize,
) -> bool {
    match num_chunks {
        1 => domain_size <= max_poly_size && max_poly_size <= Vesta16::MAX_POLY_SIZE,
        2 => max_poly_size == Vesta16::MAX_POLY_SIZE && domain_size == Vesta16::MAX_POLY_SIZE * 2,
        4 => max_poly_size == Vesta16::MAX_POLY_SIZE && domain_size == Vesta16::MAX_POLY_SIZE * 4,
        _ => false,
    }
}

fn validate_vesta16_proof(
    proof: &Vesta16Proof,
    index: &Vesta16VerifierIndex,
) -> Result<(), ProfileError> {
    let domain_size =
        usize::try_from(index.domain.size).map_err(|_| ProfileError::ProofConfiguration)?;
    let num_chunks = expected_chunks(index.max_poly_size, domain_size)
        .ok_or(ProfileError::ProofConfiguration)?;

    if !proof.prev_challenges.is_empty()
        || proof.proof.rounds() != index.max_poly_size.trailing_zeros() as usize
    {
        return Err(ProfileError::ProofConfiguration);
    }

    for commitment in proof
        .commitments
        .w_comm
        .iter()
        .chain([&proof.commitments.z_comm])
    {
        validate_commitment(commitment, num_chunks)?;
    }
    if proof.commitments.t_comm.len() < num_chunks
        || proof.commitments.t_comm.len() > 7 * num_chunks
    {
        return Err(ProfileError::CommitmentChunks);
    }

    validate_lookup_proof(proof, index, num_chunks)?;
    validate_evaluations(&proof.evals, index, num_chunks)
}

fn validate_lookup_verifier_index(
    index: &Vesta16VerifierIndex,
    num_chunks: usize,
) -> Result<(), ProfileError> {
    let gate_patterns = LookupPatterns {
        xor: index.xor_comm.is_some(),
        lookup: false,
        range_check: index.range_check0_comm.is_some()
            || index.range_check1_comm.is_some()
            || index.rot_comm.is_some(),
        foreign_field_mul: index.foreign_field_mul_comm.is_some(),
    };

    let Some(lookup_index) = &index.lookup_index else {
        return (gate_patterns == LookupPatterns::default())
            .then_some(())
            .ok_or(ProfileError::VerifierIndexConfiguration);
    };

    let patterns = LookupPatterns {
        lookup: lookup_index.lookup_info.features.patterns.lookup,
        ..gate_patterns
    };
    if patterns == LookupPatterns::default() {
        return Err(ProfileError::VerifierIndexConfiguration);
    }

    let expected_features = LookupFeatures {
        patterns,
        joint_lookup_used: patterns.joint_lookups_used(),
        uses_runtime_tables: lookup_index.runtime_tables_selector.is_some(),
    };
    let expected_info = LookupInfo::create(expected_features);
    let actual_info = lookup_index.lookup_info;

    if actual_info.features != expected_features
        || actual_info.max_per_row != expected_info.max_per_row
        || actual_info.max_joint_size != expected_info.max_joint_size
        || lookup_index.joint_lookup_used != expected_features.joint_lookup_used
        || lookup_index.lookup_selectors.xor.is_some() != patterns.xor
        || lookup_index.lookup_selectors.lookup.is_some() != patterns.lookup
        || lookup_index.lookup_selectors.range_check.is_some() != patterns.range_check
        || lookup_index.lookup_selectors.ffmul.is_some() != patterns.foreign_field_mul
        || lookup_index.lookup_table.len() < expected_info.max_joint_size as usize
        || lookup_index.lookup_table.len() > Vesta16::MAX_LOOKUP_TABLE_WIDTH
    {
        return Err(ProfileError::VerifierIndexConfiguration);
    }

    for commitment in lookup_index
        .lookup_table
        .iter()
        .chain(lookup_index.lookup_selectors.xor.iter())
        .chain(lookup_index.lookup_selectors.lookup.iter())
        .chain(lookup_index.lookup_selectors.range_check.iter())
        .chain(lookup_index.lookup_selectors.ffmul.iter())
        .chain(lookup_index.table_ids.iter())
        .chain(lookup_index.runtime_tables_selector.iter())
    {
        validate_commitment(commitment, num_chunks)?;
    }

    Ok(())
}

fn validate_lookup_proof(
    proof: &Vesta16Proof,
    index: &Vesta16VerifierIndex,
    num_chunks: usize,
) -> Result<(), ProfileError> {
    match (&proof.commitments.lookup, &index.lookup_index) {
        (None, None) => Ok(()),
        (Some(commitments), Some(lookup_index)) => {
            if commitments.sorted.len() != lookup_index.lookup_info.max_per_row + 1
                || commitments.runtime.is_some() != lookup_index.runtime_tables_selector.is_some()
            {
                return Err(ProfileError::ProofConfiguration);
            }

            for commitment in commitments
                .sorted
                .iter()
                .chain([&commitments.aggreg])
                .chain(commitments.runtime.iter())
            {
                validate_commitment(commitment, num_chunks)?;
            }

            Ok(())
        }
        _ => Err(ProfileError::ProofConfiguration),
    }
}

fn validate_commitment<T>(commitment: &PolyComm<T>, num_chunks: usize) -> Result<(), ProfileError> {
    (commitment.chunks.len() == num_chunks)
        .then_some(())
        .ok_or(ProfileError::CommitmentChunks)
}

fn validate_evaluations(
    evals: &ProofEvaluations<PointEvaluations<Vec<Fp>>>,
    index: &Vesta16VerifierIndex,
    num_chunks: usize,
) -> Result<(), ProfileError> {
    let ProofEvaluations {
        public,
        w,
        z,
        s,
        coefficients,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = evals;

    for evaluation in public
        .iter()
        .chain(w.iter())
        .chain([z])
        .chain(s.iter())
        .chain(coefficients.iter())
        .chain([
            generic_selector,
            poseidon_selector,
            complete_add_selector,
            mul_selector,
            emul_selector,
            endomul_scalar_selector,
        ])
    {
        validate_evaluation(evaluation, num_chunks)?;
    }

    validate_optional_evaluation(
        range_check0_selector,
        index.range_check0_comm.is_some(),
        num_chunks,
    )?;
    validate_optional_evaluation(
        range_check1_selector,
        index.range_check1_comm.is_some(),
        num_chunks,
    )?;
    validate_optional_evaluation(
        foreign_field_add_selector,
        index.foreign_field_add_comm.is_some(),
        num_chunks,
    )?;
    validate_optional_evaluation(
        foreign_field_mul_selector,
        index.foreign_field_mul_comm.is_some(),
        num_chunks,
    )?;
    validate_optional_evaluation(xor_selector, index.xor_comm.is_some(), num_chunks)?;
    validate_optional_evaluation(rot_selector, index.rot_comm.is_some(), num_chunks)?;

    validate_lookup_evaluations(
        (
            lookup_aggregation,
            lookup_table,
            lookup_sorted,
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        ),
        index.lookup_index.as_ref(),
        num_chunks,
    )
}

#[allow(clippy::type_complexity)]
fn validate_lookup_evaluations(
    (
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    ): (
        &Option<PointEvaluations<Vec<Fp>>>,
        &Option<PointEvaluations<Vec<Fp>>>,
        &[Option<PointEvaluations<Vec<Fp>>>; 5],
        &Option<PointEvaluations<Vec<Fp>>>,
        &Option<PointEvaluations<Vec<Fp>>>,
        &Option<PointEvaluations<Vec<Fp>>>,
        &Option<PointEvaluations<Vec<Fp>>>,
        &Option<PointEvaluations<Vec<Fp>>>,
        &Option<PointEvaluations<Vec<Fp>>>,
    ),
    lookup_index: Option<&LookupVerifierIndex<mina_curves::pasta::Vesta>>,
    num_chunks: usize,
) -> Result<(), ProfileError> {
    let Some(lookup_index) = lookup_index else {
        if lookup_aggregation.is_some()
            || lookup_table.is_some()
            || lookup_sorted.iter().any(Option::is_some)
            || runtime_lookup_table.is_some()
            || runtime_lookup_table_selector.is_some()
            || xor_lookup_selector.is_some()
            || lookup_gate_lookup_selector.is_some()
            || range_check_lookup_selector.is_some()
            || foreign_field_mul_lookup_selector.is_some()
        {
            return Err(ProfileError::EvaluationLengths);
        }
        return Ok(());
    };

    validate_optional_evaluation(lookup_aggregation, true, num_chunks)?;
    validate_optional_evaluation(lookup_table, true, num_chunks)?;
    for (position, evaluation) in lookup_sorted.iter().enumerate() {
        validate_optional_evaluation(
            evaluation,
            position <= lookup_index.lookup_info.max_per_row,
            num_chunks,
        )?;
    }

    let patterns = lookup_index.lookup_info.features.patterns;
    let runtime = lookup_index.runtime_tables_selector.is_some();
    validate_optional_evaluation(runtime_lookup_table, runtime, num_chunks)?;
    validate_optional_evaluation(runtime_lookup_table_selector, runtime, num_chunks)?;
    validate_optional_evaluation(xor_lookup_selector, patterns.xor, num_chunks)?;
    validate_optional_evaluation(lookup_gate_lookup_selector, patterns.lookup, num_chunks)?;
    validate_optional_evaluation(
        range_check_lookup_selector,
        patterns.range_check,
        num_chunks,
    )?;
    validate_optional_evaluation(
        foreign_field_mul_lookup_selector,
        patterns.foreign_field_mul,
        num_chunks,
    )
}

fn validate_optional_evaluation(
    evaluation: &Option<PointEvaluations<Vec<Fp>>>,
    expected: bool,
    num_chunks: usize,
) -> Result<(), ProfileError> {
    if evaluation.is_some() != expected {
        return Err(ProfileError::EvaluationLengths);
    }
    if let Some(evaluation) = evaluation {
        validate_evaluation(evaluation, num_chunks)?;
    }
    Ok(())
}

fn validate_evaluation(
    evaluation: &PointEvaluations<Vec<Fp>>,
    num_chunks: usize,
) -> Result<(), ProfileError> {
    (evaluation.zeta.len() == num_chunks && evaluation.zeta_omega.len() == num_chunks)
        .then_some(())
        .ok_or(ProfileError::EvaluationLengths)
}

pub(crate) fn expected_chunks(max_poly_size: usize, domain_size: usize) -> Option<usize> {
    if max_poly_size == 0 || !max_poly_size.is_power_of_two() || !domain_size.is_power_of_two() {
        return None;
    }

    Some(if domain_size < max_poly_size {
        1
    } else {
        domain_size / max_poly_size
    })
}
