// pub mod serialize;

use std::collections::BTreeMap;
use std::io::BufReader;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::TypeInfo;
use halo2_proofs::arithmetic::{CurveAffine, Field};
use halo2_proofs::halo2curves::bn256;
use halo2_proofs::plonk::{keygen_vk, Advice, Any, Circuit, ConstraintSystem, Error, Expression, Fixed, Instance};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::poly::Rotation as Halo2Rotation;
use crate::vk::{ConvertError, Fr, G1Affine};

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct CircuitInfo<C, F> {
    vk_transcript_repr: F,
    fixed_commitments: Vec<C>,
    permutation_commitments: Vec<C>,

    num_fixed_columns: usize,
    num_instance_columns: usize,
    pub num_advice_columns: usize,

    k: u8,
    max_num_query_of_advice_column: u32,
    cs_degree: u32,
    advice_column_phase: Vec<u8>,
    challenge_phase: Vec<u8>,
    gates: Vec<Gate<F>>,

    advice_queries: Vec<ColumnQuery>,
    instance_queries: Vec<ColumnQuery>,
    fixed_queries: Vec<ColumnQuery>,
    permutation_columns: Vec<Column>,
    lookups: Vec<Lookup<F>>,
    shuffles: Vec<Shuffle<F>>,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct ColumnQuery {
    name: &'static str,
    pub column: Column,
    pub rotation: Rotation,
}
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Column {
    name: &'static str,
    pub index: u32,
    pub column_type: u8,
}

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub struct Selector(pub(crate) usize, bool);

#[derive(Clone, Debug)]
pub struct VirtualCell {
    // pub(crate) column: Column<Any>,
    // pub(crate) rotation: Rotation,
}

pub struct Gate<F> {
    pub name: &'static str,
    pub constraint_names: Vec<&'static str>,
    pub polys: Vec<Expression<F>>,
    /// We track queried selectors separately from other cells, so that we can use them to
    /// trigger debug checks on gates.
    pub queried_selectors: Vec<halo2_proofs::plonk::Selector>,
    pubqueried_cells: Vec<VirtualCell>,
}

impl From<halo2_proofs::plonk::Column<Any>> for Column {
    fn from(value: halo2_proofs::plonk::Column<Any>) -> Self {
        let column_type = match value.column_type() {
            Any::Advice(phase) => phase.phase(),
            Any::Fixed => 255,
            Any::Instance => 244,
        };
        Column {
            name: "TODO",
            index: value.index() as u32,
            column_type,
        }
    }
}
impl From<&Column> for halo2_proofs::plonk::Column<Any> {
    fn from(value: &Column) -> Self {
        let column_type = match value.column_type {
            255 => Any::Fixed,
            244 => Any::Instance,
            phase => Any::Advice(match phase {
                0 => Advice::new(halo2_proofs::plonk::FirstPhase),
                1 => Advice::new(halo2_proofs::plonk::SecondPhase),
                2 => Advice::new(halo2_proofs::plonk::ThirdPhase),
                _ => unreachable!(),
            }),
        };

        halo2_proofs::plonk::Column::new(value.index as usize, column_type)
    }
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, )]
pub struct Rotation {
    pub rotation: u32,
    pub next: bool,
}

impl From<halo2_proofs::poly::Rotation> for Rotation {
    fn from(value: halo2_proofs::poly::Rotation) -> Self {
        if value.0.is_negative() {
            Self {
                rotation: value.0.unsigned_abs(),
                next: false,
            }
        } else {
            Self {
                rotation: value.0 as u32,
                next: true,
            }
        }
    }
}
#[derive(Debug)]
pub struct Lookup<F> {
    pub input_exprs: Vec<Expression<F>>,
    pub table_exprs: Vec<Expression<F>>,
}
#[derive(Debug)]
pub struct Shuffle<F> {
    pub input_exprs: Vec<Expression<F>>,
    pub shuffle_exprs: Vec<Expression<F>>,
}

// pub fn generate_circuit_info<'params, C, P, ConcreteCircuit>(
//     params: &P,
//     circuit: &ConcreteCircuit,
// ) -> Result<CircuitInfo<C>, Error>
// where
//     C: CurveAffine,
//     P: Params<'params, C>,
//     ConcreteCircuit: Circuit<C::Scalar>,
//     C::Scalar: FromUniformBytes<64>,
//     C::ScalarExt: FromUniformBytes<64>,
// {
//     let vk = keygen_vk(params, circuit)?;
//     let cs = vk.cs();
//     // as halo2 dones'nt expose vk's transcript_repr,
//     // we had to copy the code here.
//     let vk_repr = {
//         let mut hasher = blake2b_simd::Params::new()
//             .hash_length(64)
//             .personal(b"Halo2-Verify-Key")
//             .to_state();
//
//         let s = format!("{:?}", vk.pinned());
//
//         hasher.update(&(s.len() as u64).to_le_bytes());
//         hasher.update(s.as_bytes());
//
//         // Hash in final Blake2bState
//         C::Scalar::from_uniform_bytes(hasher.finalize().as_array())
//     };
//
//     let info = CircuitInfo {
//         vk_transcript_repr: vk_repr,
//         fixed_commitments: vk.fixed_commitments().clone(),
//         permutation_commitments: vk.permutation().commitments().clone(),
//         k: (params.k() as u8), // we expect k would not be too large.
//         cs_degree: cs.degree() as u32,
//         num_fixed_columns: cs.num_fixed_columns() as u64,
//         num_instance_columns: cs.num_instance_columns() as u64,
//         advice_column_phase: cs.advice_column_phase(),
//         challenge_phase: cs.challenge_phase(),
//
//         advice_queries: cs
//             .advice_queries()
//             .iter()
//             .map(|(c, r)| ColumnQuery {
//                 column: halo2_proofs::plonk::Column::<Any>::from(*c).into(),
//                 rotation: From::from(*r),
//             })
//             .collect(),
//         instance_queries: cs
//             .instance_queries()
//             .iter()
//             .map(|(c, r)| ColumnQuery {
//                 column: halo2_proofs::plonk::Column::<Any>::from(*c).into(),
//                 rotation: From::from(*r),
//             })
//             .collect(),
//         fixed_queries: cs
//             .fixed_queries()
//             .iter()
//             .map(|(c, r)| ColumnQuery {
//                 column: halo2_proofs::plonk::Column::<Any>::from(*c).into(),
//                 rotation: From::from(*r),
//             })
//             .collect(),
//         permutation_columns: cs
//             .permutation()
//             .get_columns()
//             .iter()
//             .map(|c| From::from(*c))
//             .collect(),
//         lookups: cs
//             .lookups()
//             .iter()
//             .map(|l| Lookup {
//                 input_exprs: l
//                     .input_expressions()
//                     .iter()
//                     .map(|e| {
//                         expression_transform(
//                             cs,
//                             e,
//                             cs.advice_queries().len(),
//                             cs.fixed_queries().len(),
//                             cs.instance_queries().len(),
//                             cs.challenge_phase().len(),
//                         )
//                     })
//                     .collect(),
//                 table_exprs: l
//                     .table_expressions(),
//                 // .iter()
//                 // .map(|e| {
//                 //     expression_transform(
//                 //         cs,
//                 //         e,
//                 //         cs.advice_queries().len(),
//                 //         cs.fixed_queries().len(),
//                 //         cs.instance_queries().len(),
//                 //         cs.challenge_phase().len(),
//                 //     )
//                 // })
//                 // .collect(),
//             })
//             .collect(),
//         shuffles: cs
//             .shuffles()
//             .iter()
//             .map(|s| Shuffle {
//                 input_exprs: s
//                     .input_expressions(),
//                 // .iter()
//                 // .map(|e| {
//                 //     expression_transform(
//                 //         cs,
//                 //         e,
//                 //         cs.advice_queries().len(),
//                 //         cs.fixed_queries().len(),
//                 //         cs.instance_queries().len(),
//                 //         cs.challenge_phase().len(),
//                 //     )
//                 // })
//                 // .collect(),
//                 shuffle_exprs: s
//                     .shuffle_expressions(),
//                 // .iter()
//                 // .map(|e| {
//                 //     expression_transform(
//                 //         cs,
//                 //         e,
//                 //         cs.advice_queries().len(),
//                 //         cs.fixed_queries().len(),
//                 //         cs.instance_queries().len(),
//                 //         cs.challenge_phase().len(),
//                 //     )
//                 // })
//                 // .collect(),
//             })
//             .collect(),
//         max_num_query_of_advice_column: cs
//             .advice_queries()
//             .iter()
//             .fold(BTreeMap::default(), |mut m, (c, _r)| {
//                 if let std::collections::btree_map::Entry::Vacant(e) = m.entry(c.index()) {
//                     e.insert(1u32);
//                 } else {
//                     *m.get_mut(&c.index()).unwrap() += 1;
//                 }
//                 m
//             })
//             .values()
//             .max()
//             .cloned()
//             .unwrap_or_default(),
//
//         gates: cs
//             .gates(),
//         // .iter()
//         // .map(|g| {
//         //     g.polynomials()
//         //         // .iter()
//         //         // .map(|e| {
//         //         //     expression_transform(
//         //         //         cs,
//         //         //         e,
//         //         //         cs.advice_queries().len(),
//         //         //         cs.fixed_queries().len(),
//         //         //         cs.instance_queries().len(),
//         //         //         cs.challenge_phase().len(),
//         //         //     )
//         //         // })
//         //         // .collect()
//         // })
//         // .collect(),
//     };
//     Ok(info)
// }

impl TryFrom<CircuitInfo<G1Affine, Fr>> for ConstraintSystem<bn256::Fr> {
    type Error = ConvertError;

    fn try_from(value: CircuitInfo<G1Affine, Fr>) -> Result<Self, Self::Error> {
        let mut cs = ConstraintSystem::default();
        cs.num_fixed_columns = value.num_fixed_columns;
        cs.num_advice_columns = value.num_advice_columns;
        cs.num_instance_columns = value.num_instance_columns;

        cs.advice_column_phase = value.advice_column_phase.iter().map(|x| match x {
            0 => halo2_proofs::plonk::FirstPhase.to_sealed(),
            1 => halo2_proofs::plonk::SecondPhase.to_sealed(),
            2 => halo2_proofs::plonk::ThirdPhase.to_sealed(),
            _ => unreachable!(),
        }).collect();

        cs.challenge_phase = value.challenge_phase.iter().map(|x| match x {
            0 => halo2_proofs::plonk::FirstPhase.to_sealed(),
            1 => halo2_proofs::plonk::SecondPhase.to_sealed(),
            2 => halo2_proofs::plonk::ThirdPhase.to_sealed(),
            _ => unreachable!(),
        }).collect();

        for gate in value.gates {
            cs.create_gate(gate.name, |cs| {
                let mut polys = Vec::new();
                for poly in gate.polys {
                    let mut terms = Vec::new();
                    for (selector, coeff) in poly.iter() {
                        let column = match selector.0 {
                            0 => cs.advice_column(selector.1),
                            1 => cs.fixed_column(selector.1),
                            2 => cs.instance_column(selector.1),
                            _ => unreachable!(),
                        };
                        terms.push((column, coeff));
                    }
                    polys.push(cs.poly(terms));
                }
                gate.queried_selectors.iter().for_each(|selector| {
                    cs.query_selector(*selector);
                });
                polys
            });
        }

        cs.advice_queries = value.advice_queries.iter().map(|q| {
            ((&q.column).into(), (&q.rotation).into())
        }).collect();

        cs.instance_queries = value.instance_queries.iter().map(|q| {
            ((&q.column).into(), (&q.rotation).into())
        }).collect();

        cs.fixed_queries = value.fixed_queries.iter().map(|q| {
            ((&q.column).into(), (&q.rotation).into())
        }).collect();

        cs.permutation = halo2_proofs::plonk::permutation::Argument {
            columns: value.permutation_columns.iter().map(|c| c.into()).collect(),
        };

        cs.lookups = value.lookups.iter().map(|l| {
            halo2_proofs::plonk::lookup::Argument::new("", l.input_exprs.iter().zip(l.table_exprs.iter()).map(|(i, t)| (i.clone(), t.clone())).collect())
        }).collect();

        // halo2_proofs::plonk:: VerifyingKey::<G1Affine>::read::<BufReader<&[u8]>, BaseCircuitBuilder<Fr>>

        Ok(cs)
    }
}
