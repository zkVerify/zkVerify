#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_plonky2_verifier`.
pub trait WeightInfo {
    fn verify_proof() -> Weight;
    fn get_vk() -> Weight;
    fn validate_vk() -> Weight;
    fn compute_statement_hash() -> Weight;
    fn register_vk() -> Weight;
    fn unregister_vk() -> Weight;
}

impl WeightInfo for () {
    fn verify_proof() -> Weight {
        Weight::from_parts(1_000_000_000, 0)
    }

    fn get_vk() -> Weight {
        Weight::from_parts(1_000_000, 10000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
    }

    fn validate_vk() -> Weight {
        Weight::from_parts(100_000, 0)
    }

    fn compute_statement_hash() -> Weight {
        Weight::from_parts(10_000_000, 0)
    }

    fn register_vk() -> Weight {
        Weight::from_parts(100_000_000, 10000)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }

    fn unregister_vk() -> Weight {
        Weight::from_parts(100_000_000, 10000)
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
}