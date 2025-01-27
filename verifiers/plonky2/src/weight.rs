#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

pub trait WeightInfo {
    fn submit_proof() -> Weight;
    fn submit_proof_with_vk_hash() -> Weight;
    fn register_vk() -> Weight;
    fn unregister_vk() -> Weight;
}

impl WeightInfo for () {
    fn submit_proof() -> Weight {
        Weight::from_parts(1_000_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn submit_proof_with_vk_hash() -> Weight {
        Weight::from_parts(1_000_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn register_vk() -> Weight {
        Weight::from_parts(1_000_000, 0)
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    fn unregister_vk() -> Weight {
        Weight::from_parts(1_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
}