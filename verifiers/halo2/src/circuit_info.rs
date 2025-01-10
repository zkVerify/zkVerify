
use sp_std::fmt::Debug;
use crate::vk::{ConvertError, Fr, G1Affine};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::TypeInfo;
use frame_support::traits::IsType;
use halo2_proofs::arithmetic::{CurveAffine, Field};
use halo2_proofs::halo2curves::bn256;
use halo2_proofs::halo2curves::bn256::Bn256;
use halo2_proofs::plonk::{
    keygen_vk, Advice, Any, Challenge, Circuit, ConstraintSystem, Error, Fixed, Instance,
};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::poly::Rotation as Halo2Rotation;
use sp_std::collections::btree_map::{BTreeMap, Entry};

#[cfg(not(feature = "std"))]
use alloc::{vec::Vec, boxed::Box};

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo)]
pub struct CircuitInfo<F> {
    pub num_fixed_columns: u64,
    pub num_instance_columns: u64,
    pub num_advice_columns: u64,

    pub max_num_query_of_advice_column: u32,
    pub cs_degree: u32,
    pub advice_column_phase: Vec<u8>,
    pub challenge_phase: Vec<u8>,
    pub gates: Vec<Gate<F>>,

    pub advice_queries: Vec<ColumnQuery>,
    pub instance_queries: Vec<ColumnQuery>,
    pub fixed_queries: Vec<ColumnQuery>,
    pub permutation_columns: Vec<Column>,
    pub lookups: Vec<Lookup<F>>,
    // pub shuffles: Vec<Shuffle<F>>,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct ColumnQuery {
    // name: &'static str,
    pub column: Column,
    pub rotation: Rotation,
}
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Column {
    // name: String,
    pub index: u32,
    pub column_type: u8,
}

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub struct Selector(pub(crate) usize, bool);

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct VirtualCell {
    pub(crate) column: Column,
    pub(crate) rotation: Rotation,
}

impl From<&halo2_proofs::plonk::VirtualCell> for VirtualCell {
    fn from(value: &halo2_proofs::plonk::VirtualCell) -> Self {
        Self {
            column: value.column().into(),
            rotation: value.rotation().into(),
        }
    }
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo)]
pub struct Gate<F> {
    // pub name: String,
    // pub constraint_names: Vec<String>,
    pub polys: Vec<Expression<F>>,
    /// We track queried selectors separately from other cells, so that we can use them to
    /// trigger debug checks on gates.
    pub queried_selectors: Vec<Selector>,
    pub queried_cells: Vec<VirtualCell>,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Selector(pub(crate) u64, bool);

impl From<Selector> for halo2_proofs::plonk::Selector {
    fn from(value: Selector) -> Self {
        Self::new(value.0 as usize, value.1)
    }
}

impl From<&halo2_proofs::plonk::Selector> for Selector {
    fn from(value: &halo2_proofs::plonk::Selector) -> Self {
        Self(value.index() as u64, value.is_simple())
    }
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum Expression<F> {
    /// This is a constant polynomial
    Constant(F),
    /// This is a virtual selector
    Selector(Selector),
    /// This is a fixed column queried at a certain relative location
    Fixed(FixedQuery),
    /// This is an advice (witness) column queried at a certain relative location
    Advice(AdviceQuery),
    /// This is an instance (external) column queried at a certain relative location
    Instance(InstanceQuery),
    /// This is a challenge
    Challenge((u32, u8)),
    /// This is a negated polynomial
    Negated(Box<Expression<F>>),
    /// This is the sum of two polynomials
    Sum(Box<Expression<F>>, Box<Expression<F>>),
    /// This is the product of two polynomials
    Product(Box<Expression<F>>, Box<Expression<F>>),
    /// This is a scaled polynomial
    Scaled(Box<Expression<F>>, F),
}

impl From<&halo2_proofs::plonk::Expression<bn256::Fr>> for Expression<Fr> {
    fn from(value: &halo2_proofs::plonk::Expression<bn256::Fr>) -> Self {
        match value {
            halo2_proofs::plonk::Expression::Constant(c) => Self::Constant((*c).into()),
            halo2_proofs::plonk::Expression::Selector(s) => Self::Selector(s.into()),
            halo2_proofs::plonk::Expression::Fixed(f) => Self::Fixed(f.into()),
            halo2_proofs::plonk::Expression::Advice(a) => Self::Advice(a.into()),
            halo2_proofs::plonk::Expression::Instance(i) => Self::Instance(i.into()),
            halo2_proofs::plonk::Expression::Challenge(c) => Self::Challenge((c.index() as u32, c.phase() as u8)),
            halo2_proofs::plonk::Expression::Negated(n) => Self::Negated(n.into()),
            halo2_proofs::plonk::Expression::Sum(s, s2) => Self::Sum(s.into(), s2.into()),
            halo2_proofs::plonk::Expression::Product(p, p2) => Self::Product(p.into(), p2.into()),
            halo2_proofs::plonk::Expression::Scaled(s, f) => Self::Scaled(s.into(), (*f).into()),
        }
    }
}

impl From<&Box<halo2_proofs::plonk::Expression<bn256::Fr>>> for Box<Expression<Fr>> {
    fn from(value: &Box<halo2_proofs::plonk::Expression<bn256::Fr>>) -> Self {
        Box::new(value.as_ref().into())
    }
}

impl From<Box<Expression<Fr>>> for Box<halo2_proofs::plonk::Expression<bn256::Fr>> {
    fn from(value: Box<Expression<Fr>>) -> Self {
        Box::new(value.as_ref().clone().into())
    }
}

impl From<Expression<Fr>> for halo2_proofs::plonk::Expression<bn256::Fr> {
    fn from(value: Expression<Fr>) -> Self {
        match value {
            Expression::Constant(c) => halo2_proofs::plonk::Expression::Constant(c.into()),
            Expression::Selector(s) => halo2_proofs::plonk::Expression::Selector(s.into()),
            Expression::Fixed(f) => halo2_proofs::plonk::Expression::Fixed(f.into()),
            Expression::Advice(a) => halo2_proofs::plonk::Expression::Advice(a.into()),
            Expression::Instance(i) => halo2_proofs::plonk::Expression::Instance(i.into()),
            Expression::Challenge((index, phase)) => halo2_proofs::plonk::Expression::Challenge(halo2_proofs::plonk::Challenge::new(index as usize, phase as u8)),
            Expression::Negated(n) => halo2_proofs::plonk::Expression::Negated(n.into()),
            Expression::Sum(s, s2) => halo2_proofs::plonk::Expression::Sum(s.into(), s2.into()),
            Expression::Product(p, p2) => halo2_proofs::plonk::Expression::Product(p.into(), p2.into()),
            Expression::Scaled(s, f) => halo2_proofs::plonk::Expression::Scaled(s.into(), f.into()),
        }
    }
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
struct FixedQuery {
    pub index: u32,
    pub column_index: u32,
    pub rotation: Rotation,
}

impl From<&halo2_proofs::plonk::FixedQuery> for FixedQuery {
    fn from(value: &halo2_proofs::plonk::FixedQuery) -> Self {
        Self {
            index: value.index() as u32,
            column_index: value.column_index() as u32,
            rotation: value.rotation().into_ref().into(),
        }
    }
}

impl From<FixedQuery> for halo2_proofs::plonk::FixedQuery {
    fn from(value: FixedQuery) -> Self {
        halo2_proofs::plonk::FixedQuery::new(value.index as usize, value.column_index as usize, value.rotation.into_ref().into())
    }
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
struct InstanceQuery {
    pub index: u32,
    pub column_index: u32,
    pub rotation: Rotation,
}

impl From<&halo2_proofs::plonk::InstanceQuery> for InstanceQuery {
    fn from(value: &halo2_proofs::plonk::InstanceQuery) -> Self {
        Self {
            index: value.index() as u32,
            column_index: value.column_index() as u32,
            rotation: value.rotation().into_ref().into(),
        }
    }
}

impl From<InstanceQuery> for halo2_proofs::plonk::InstanceQuery {
    fn from(value: InstanceQuery) -> Self {
        halo2_proofs::plonk::InstanceQuery::new(value.index as usize, value.column_index as usize, value.rotation.into_ref().into())
    }
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
struct AdviceQuery {
    pub index: u32,
    pub column_index: u32,
    pub rotation: Rotation,
    pub phase: u8,
}

impl From<&halo2_proofs::plonk::AdviceQuery> for AdviceQuery {
    fn from(value: &halo2_proofs::plonk::AdviceQuery) -> Self {
        Self {
            index: value.index() as u32,
            column_index: value.column_index() as u32,
            rotation: value.rotation().into_ref().into(),
            phase: value.phase(),
        }
    }
}

impl From<AdviceQuery> for halo2_proofs::plonk::AdviceQuery {
    fn from(value: AdviceQuery) -> Self {
        let phase = match value.phase {
            0 => halo2_proofs::plonk::FirstPhase::sealed(),
            1 => halo2_proofs::plonk::SecondPhase::sealed(),
            2 => halo2_proofs::plonk::ThirdPhase::sealed(),
            _ => unreachable!(),
        };
        halo2_proofs::plonk::AdviceQuery::new(value.index as usize, value.column_index as usize, value.rotation.into_ref().into(), phase)
    }
}

impl From<&halo2_proofs::plonk::Column<Any>> for Column {
    fn from(value: &halo2_proofs::plonk::Column<Any>) -> Self {
        let column_type = match value.column_type() {
            Any::Advice(phase) => phase.phase(),
            Any::Fixed => 255,
            Any::Instance => 244,
        };
        Column {
            index: value.index() as u32,
            column_type,
        }
    }
}

impl From<halo2_proofs::plonk::Column<Any>> for Column {
    fn from(value: halo2_proofs::plonk::Column<Any>) -> Self {
        Self::from(&value)
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


impl From<&Column> for halo2_proofs::plonk::Column<Advice> {
    
    fn from(value: &Column) -> Self {
        let column_type = match value.column_type {
            0 => Advice::new(halo2_proofs::plonk::FirstPhase),
            1 => Advice::new(halo2_proofs::plonk::SecondPhase),
            2 => Advice::new(halo2_proofs::plonk::ThirdPhase),
            _ => unreachable!(),
        };

        halo2_proofs::plonk::Column::new(value.index as usize, column_type)
    }
}

impl From<&Column> for halo2_proofs::plonk::Column<Instance> {

    fn from(value: &Column) -> Self {
        if value.column_type != 244 {
            panic!("Expected an instance column, got {:?}", value.column_type);
        }

        halo2_proofs::plonk::Column::new(value.index as usize, Instance)
    }
}

impl From<&Column> for halo2_proofs::plonk::Column<Fixed> {
    fn from(value: &Column) -> Self {
        if value.column_type != 255 {
            panic!("Expected a fixed column, got {:?}", value.column_type);
        }

        halo2_proofs::plonk::Column::new(value.index as usize, Fixed)
    }
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Rotation {
    pub rotation: u32,
    pub next: bool,
}

impl From<&halo2_proofs::poly::Rotation> for Rotation {
    fn from(value: &halo2_proofs::poly::Rotation) -> Self {
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

impl From<&Rotation> for halo2_proofs::poly::Rotation {
    fn from(value: &Rotation) -> Self {
        if value.next {
            halo2_proofs::poly::Rotation(value.rotation as i32)
        } else {
            halo2_proofs::poly::Rotation(-(value.rotation as i32))
        }
    }
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Lookup<F> {
    pub input_exprs: Vec<Expression<F>>,
    pub table_exprs: Vec<Expression<F>>,
}
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Shuffle<F> {
    pub input_exprs: Vec<Expression<F>>,
    pub shuffle_exprs: Vec<Expression<F>>,
}

impl TryFrom<&halo2_proofs::plonk::ConstraintSystem<bn256::Fr>> for CircuitInfo<Fr> {
    type Error = ConvertError;

    fn try_from(cs: &halo2_proofs::plonk::ConstraintSystem<bn256::Fr>) -> Result<Self, Self::Error> {

    let info = CircuitInfo {
        // k: (params.k() as u8), // we expect k would not be too large.
        cs_degree: cs.degree() as u32,
        num_fixed_columns: cs.num_fixed_columns() as u64,
        num_advice_columns: cs.num_advice_columns() as u64,
        num_instance_columns: cs.num_instance_columns() as u64,
        advice_column_phase: cs.advice_column_phase(),
        challenge_phase: cs.challenge_phase(),

        advice_queries: cs
            .advice_queries()
            .iter()
            .map(|(c, r)| ColumnQuery {
                column: halo2_proofs::plonk::Column::<Any>::from(*c).into(),
                rotation: From::from(r),
            })
            .collect(),
        instance_queries: cs
            .instance_queries()
            .iter()
            .map(|(c, r)| ColumnQuery {
                column: halo2_proofs::plonk::Column::<Any>::from(*c).into(),
                rotation: From::from(r),
            })
            .collect(),
        fixed_queries: cs
            .fixed_queries()
            .iter()
            .map(|(c, r)| ColumnQuery {
                column: halo2_proofs::plonk::Column::<Any>::from(*c).into(),
                rotation: From::from(r),
            })
            .collect(),
        permutation_columns: cs
            .permutation()
            .get_columns()
            .iter()
            .map(|c| From::from(*c))
            .collect(),
        lookups: cs
            .lookups()
            .iter()
            .map(|l| Lookup {
                input_exprs: l.input_expressions().iter().map(|e| e.into()).collect(),
                table_exprs: l.table_expressions().iter().map(|e| e.into()).collect(),
            })
            .collect(),
        // shuffles: cs
        //     .shuffles()
        //     .iter()
        //     .map(|s| Shuffle {
        //         input_exprs: s
        //             .input_expressions()
        //             .iter()
        //             .map(|e| {
        //                 e.into()
        //             })
        //             .collect(),
        //         shuffle_exprs: s
        //             .shuffle_expressions()
        //             .iter()
        //             .map(|e| {
        //                 e.into()
        //             })
        //             .collect(),
        //     })
        //     .collect(),
        max_num_query_of_advice_column: cs
            .advice_queries()
            .iter()
            .fold(BTreeMap::default(), |mut m, (c, _r)| {
                if let Entry::Vacant(e) = m.entry(c.index()) {
                    e.insert(1u32);
                } else {
                    *m.get_mut(&c.index()).unwrap() += 1;
                }
                m
            })
            .values()
            .max()
            .cloned()
            .unwrap_or_default(),

        gates: cs
            .gates()
            .iter()
            .map(|g| {
                Gate {
                    // name: g.name(),
                    // constraint_names: g.constraint_names.clone(),
                    polys: g.polynomials().iter().map(|e| e.into()).collect(),
                    queried_selectors: g.queried_selectors().iter().map(|e| e.into()).collect(),
                    queried_cells: g.queried_cells().iter().map(|e| e.into()).collect(),
                }
            })
            .collect(),
    };

    Ok(info)
    }
}

impl TryFrom<CircuitInfo<Fr>> for ConstraintSystem<bn256::Fr> {
    type Error = ConvertError;

    fn try_from(value: CircuitInfo<Fr>) -> Result<Self, Self::Error> {
        let mut cs = ConstraintSystem::default();
        cs.num_fixed_columns = value.num_fixed_columns as usize;
        cs.num_advice_columns = value.num_advice_columns as usize;
        cs.num_instance_columns = value.num_instance_columns as usize;

        cs.advice_column_phase = value
            .advice_column_phase
            .iter()
            .map(|x| match x {
                0 => halo2_proofs::plonk::FirstPhase::sealed(),
                1 => halo2_proofs::plonk::SecondPhase::sealed(),
                2 => halo2_proofs::plonk::ThirdPhase::sealed(),
                _ => unreachable!(),
            })
            .collect();

        cs.challenge_phase = value
            .challenge_phase
            .iter()
            .map(|x| match x {
                0 => halo2_proofs::plonk::FirstPhase::sealed(),
                1 => halo2_proofs::plonk::SecondPhase::sealed(),
                2 => halo2_proofs::plonk::ThirdPhase::sealed(),
                _ => unreachable!(),
            })
            .collect();

        cs.advice_queries = value
            .advice_queries
            .iter()
            .map(|q| (q.column.into_ref().into(), q.rotation.into_ref().into()))
            .collect();

        cs.instance_queries = value
            .instance_queries
            .iter()
            .map(|q| (q.column.into_ref().into(), q.rotation.into_ref().into()))
            .collect();

        cs.fixed_queries = value
            .fixed_queries
            .iter()
            .map(|q| (q.column.into_ref().into(), q.rotation.into_ref().into()))
            .collect();

        cs.permutation = halo2_proofs::plonk::permutation::Argument {
            columns: value.permutation_columns.iter().map(|c| c.into()).collect(),
        };

        cs.lookups = value
            .lookups
            .iter()
            .map(|l| {
                halo2_proofs::plonk::lookup::Argument::new(
                    "",
                    l.input_exprs
                        .iter()
                        .zip(l.table_exprs.iter())
                        .map(|(i, t)| (i.clone().into(), t.clone().into()))
                        .collect(),
                )
            })
            .collect();

            for gate in value.gates {
                cs.create_gate("", |cs| {
                   
                    gate.queried_selectors.iter().for_each(|selector| {
                        cs.query_selector(selector.clone().into());
                    });
                    gate.queried_cells.iter().for_each(|cell| {
                        cs.query_any::<halo2_proofs::plonk::Column<Any>>(cell.column.into_ref().into(), cell.rotation.into_ref().into());
                    });
                    gate.polys.into_iter().map(|e| ("", e.into()))
                });
            }

        Ok(cs)
    }
}
