// Copyright 2026, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use ark_serialize::CanonicalSerialize;
use kimchi::{
    bench::{BaseSpongeVesta, ScalarSpongeVesta},
    circuits::{
        constraints::zk_rows_strict_lower_bound,
        gate::{CircuitGate, GateType},
        lookup::runtime_tables::{RuntimeTable, RuntimeTableCfg},
        polynomial::COLUMNS,
        polynomials::{
            foreign_field_add::{self, witness::FFOps},
            foreign_field_common::BigUintForeignFieldHelpers,
            foreign_field_mul,
            generic::GenericGateSpec,
            range_check,
            rot::{self, RotMode},
            xor,
        },
        wires::Wire,
    },
    groupmap::GroupMap,
    mina_curves::pasta::{Fp, Vesta},
    mina_poseidon::pasta::FULL_ROUNDS,
    poly_commitment::{
        commitment::CommitmentCurve,
        ipa::{OpeningProof, SRS as IpaSrs},
        SRS as _,
    },
    proof::ProverProof,
    prover_index::testing::new_index_for_test_with_lookups_and_custom_srs,
};
use num_bigint::BigUint;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use std::{array, env, fs, path::PathBuf};

const DEFAULT_MAX_POLY_SIZE_LOG2: u32 = 16;
const DEFAULT_PUBLIC_INPUTS: usize = 64;
type VestaProof = ProverProof<Vesta, OpeningProof<Vesta, FULL_ROUNDS>, FULL_ROUNDS>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixtureShape {
    AllFeatures,
    LookupRuntime,
    Simple,
}

impl FixtureShape {
    fn from_arg(value: Option<String>) -> Self {
        match value.as_deref() {
            None | Some("lookup-runtime") => Self::LookupRuntime,
            Some("all-features") => Self::AllFeatures,
            Some("simple") => Self::Simple,
            Some(other) => panic!("unsupported fixture shape: {other}"),
        }
    }

    const fn uses_runtime_tables(self) -> bool {
        matches!(self, Self::AllFeatures | Self::LookupRuntime)
    }

    const fn uses_all_features(self) -> bool {
        matches!(self, Self::AllFeatures)
    }
}

fn main() {
    let domain_log2 = env::args()
        .nth(1)
        .map(|value| value.parse().expect("domain log2 should be an integer"))
        .unwrap_or(17_u32);
    let max_poly_size_log2 = env::args()
        .nth(2)
        .map(|value| {
            value
                .parse()
                .expect("max polynomial size log2 should be an integer")
        })
        .unwrap_or(DEFAULT_MAX_POLY_SIZE_LOG2);
    let public_input_count = env::args()
        .nth(3)
        .map(|value| {
            value
                .parse()
                .expect("public input count should be an integer")
        })
        .unwrap_or(DEFAULT_PUBLIC_INPUTS);
    let shape = FixtureShape::from_arg(env::args().nth(4));
    assert!((12..=20).contains(&domain_log2));
    assert!((2..=DEFAULT_MAX_POLY_SIZE_LOG2).contains(&max_poly_size_log2));
    let domain_size = 1_usize << domain_log2;
    let max_poly_size = 1_usize << max_poly_size_log2;
    assert!(domain_size <= 16 * max_poly_size);
    let num_chunks = if domain_size < max_poly_size {
        1
    } else {
        domain_size / max_poly_size
    };
    let zk_rows = zk_rows_strict_lower_bound(num_chunks) + 1;
    let num_gates = domain_size - zk_rows - 1;
    assert!(public_input_count <= num_gates);
    let mut gates = (0..public_input_count)
        .map(|row| {
            CircuitGate::create_generic_gadget(Wire::for_row(row), GenericGateSpec::Pub, None)
        })
        .collect::<Vec<_>>();
    let mut witness_segments = Vec::new();
    let lookup_row = shape.uses_runtime_tables().then(|| {
        let row = gates.len();
        gates.push(CircuitGate::new(
            GateType::Lookup,
            Wire::for_row(row),
            Vec::new(),
        ));
        row
    });
    let xor_row = shape.uses_runtime_tables().then(|| {
        let row = gates.len();
        CircuitGate::extend_xor_gadget(&mut gates, 64);
        row
    });
    if shape.uses_all_features() {
        add_all_feature_gates(&mut gates, &mut witness_segments);
    }
    while gates.len() < num_gates {
        let row = gates.len();
        gates.push(CircuitGate::create_generic_gadget(
            Wire::for_row(row),
            GenericGateSpec::Const(1_u32.into()),
            None,
        ));
    }
    let runtime_cfg = shape.uses_runtime_tables().then(|| RuntimeTableCfg {
        id: 1,
        first_column: vec![Fp::from(0_u32)],
    });
    let runtime_tables = runtime_cfg
        .as_ref()
        .map(|cfg| {
            vec![RuntimeTable {
                id: cfg.id,
                data: vec![Fp::from(0_u32)],
            }]
        })
        .unwrap_or_default();

    let mut index = new_index_for_test_with_lookups_and_custom_srs::<FULL_ROUNDS, Vesta, _, _>(
        gates,
        public_input_count,
        0,
        Vec::new(),
        runtime_cfg.map(|cfg| vec![cfg]),
        false,
        Some(max_poly_size),
        |domain, size| {
            let srs = IpaSrs::<Vesta>::create(size);
            srs.get_lagrange_basis(domain);
            srs
        },
        false,
    );
    assert_eq!(index.cs.domain.d1.size, domain_size as u64);
    assert_eq!(index.max_poly_size, max_poly_size);
    assert_eq!(
        index.cs.zk_rows,
        u64::try_from(zk_rows_strict_lower_bound(num_chunks) + 1).unwrap()
    );
    index.compute_verifier_index_digest::<BaseSpongeVesta>();

    let public_inputs = vec![Fp::from(1_u32); public_input_count];
    let mut witness = array::from_fn(|_| vec![Fp::from(1_u32); num_gates]);
    for (row, input) in public_inputs.iter().enumerate() {
        witness[0][row] = *input;
    }
    if let Some(lookup_row) = lookup_row {
        witness[0][lookup_row] = Fp::from(1_u32);
        for column in witness.iter_mut().take(7).skip(1) {
            column[lookup_row] = Fp::from(0_u32);
        }
    }
    if let Some(xor_row) = xor_row {
        let xor_witness = xor::create_xor_witness(Fp::from(0_u32), Fp::from(0_u32), 64);
        for (column, xor_column) in witness.iter_mut().zip(xor_witness) {
            column[xor_row..xor_row + xor_column.len()].copy_from_slice(&xor_column);
        }
    }
    for (row, segment) in witness_segments {
        copy_witness_segment(&mut witness, row, &segment);
    }

    let group_map = <Vesta as CommitmentCurve>::Map::setup();
    let mut rng = ChaCha20Rng::from_seed([domain_log2 as u8; 32]);
    let proof = VestaProof::create_recursive::<BaseSpongeVesta, ScalarSpongeVesta, _>(
        &group_map,
        witness,
        &runtime_tables,
        &index,
        Vec::new(),
        None,
        &mut rng,
    )
    .expect("Kimchi proof should be created");
    let verifier_index = index
        .verifier_index
        .clone()
        .expect("verifier index should be computed");

    let fixture_name = match (shape, max_poly_size_log2 == DEFAULT_MAX_POLY_SIZE_LOG2) {
        (FixtureShape::AllFeatures, true) => {
            format!(
                "generated_{domain_size}_all_features_pubs_{}",
                public_inputs.len()
            )
        }
        (FixtureShape::AllFeatures, false) => format!(
            "generated_{domain_size}_maxpoly_{max_poly_size}_all_features_pubs_{}",
            public_inputs.len()
        ),
        (FixtureShape::LookupRuntime, true) => {
            format!(
                "generated_{domain_size}_lookup_runtime_pubs_{}",
                public_inputs.len()
            )
        }
        (FixtureShape::LookupRuntime, false) => format!(
            "generated_{domain_size}_maxpoly_{max_poly_size}_lookup_runtime_pubs_{}",
            public_inputs.len()
        ),
        (FixtureShape::Simple, true) => {
            format!(
                "generated_{domain_size}_simple_pubs_{}",
                public_inputs.len()
            )
        }
        (FixtureShape::Simple, false) => format!(
            "generated_{domain_size}_maxpoly_{max_poly_size}_simple_pubs_{}",
            public_inputs.len()
        ),
    };
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("verifiers/kimchi/src/resources")
        .join(fixture_name);
    fs::create_dir_all(&output_dir).expect("fixture output directory should be created");
    fs::write(
        output_dir.join("proof.bin"),
        bincode::serde::encode_to_vec(&proof, bincode::config::standard())
            .expect("proof should serialize"),
    )
    .expect("proof fixture should be written");
    fs::write(
        output_dir.join("verifier_index.bin"),
        bincode::serde::encode_to_vec(verifier_index, bincode::config::standard())
            .expect("verifier index should serialize"),
    )
    .expect("verifier index fixture should be written");
    fs::write(output_dir.join("pubs.bin"), serialize_pubs(&public_inputs))
        .expect("public inputs fixture should be written");

    println!("wrote {}", output_dir.display());
}

fn add_all_feature_gates(
    gates: &mut Vec<CircuitGate<Fp>>,
    witness_segments: &mut Vec<(usize, [Vec<Fp>; COLUMNS])>,
) {
    let modulus = BigUint::max_foreign_field_modulus::<Fp>();
    let ff_left = &modulus - BigUint::from(4_u32);
    let ff_right = BigUint::from(3_u32);

    let start = gates.len();
    let mut row = start;
    CircuitGate::extend_multi_range_check(gates, &mut row);
    witness_segments.push((
        start,
        range_check::witness::create_multi(Fp::from(1_u32), Fp::from(2_u32), Fp::from(3_u32)),
    ));

    let start = gates.len();
    let (next_row, ffadd_gates) = CircuitGate::create_chain_ffadd(start, &[FFOps::Add], &modulus);
    debug_assert_eq!(next_row, start + ffadd_gates.len());
    gates.extend(ffadd_gates);
    witness_segments.push((
        start,
        foreign_field_add::witness::create_chain(
            &[ff_left.clone(), ff_right.clone()],
            &[FFOps::Add],
            modulus.clone(),
        ),
    ));

    let start = gates.len();
    let (next_row, ffmul_gates) = CircuitGate::create_foreign_field_mul(start, &modulus);
    debug_assert_eq!(next_row, start + ffmul_gates.len());
    gates.extend(ffmul_gates);
    let (ffmul_witness, _) = foreign_field_mul::witness::create(
        &BigUint::from(11_u32),
        &BigUint::from(13_u32),
        &modulus,
    );
    witness_segments.push((start, ffmul_witness));

    let start = gates.len();
    let (next_row, rot_gates) = CircuitGate::create_rot(start, 13, RotMode::Left);
    debug_assert_eq!(next_row, start + rot_gates.len());
    gates.extend(rot_gates);
    let mut rot_witness = array::from_fn(|_| Vec::new());
    rot::extend_rot(&mut rot_witness, 0x1234_5678_9abc_def0, 13, RotMode::Left);
    witness_segments.push((start, rot_witness));
}

fn copy_witness_segment(
    witness: &mut [Vec<Fp>; COLUMNS],
    row: usize,
    segment: &[Vec<Fp>; COLUMNS],
) {
    for (column, segment_column) in witness.iter_mut().zip(segment) {
        column[row..row + segment_column.len()].copy_from_slice(segment_column);
    }
}

fn serialize_pubs(public_inputs: &[Fp]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(public_inputs.len() * 32);
    for input in public_inputs {
        input
            .serialize_compressed(&mut bytes)
            .expect("public input should serialize");
    }
    bytes
}
