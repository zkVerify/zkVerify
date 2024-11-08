# ![cfg_attr(rustfmt, rustfmt_skip)]

# ![allow(unused_parens)]

# ![allow(unused_imports)]

# ![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;
use crate::weights::pallet_plonky2_verifier;

/// Weights for `pallet_plonky2_verifier` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_plonky2_verifier::WeightInfo for ZKVWeight<T> {
    fn submit_proof() -> Weight {
        Weight::from_parts(1_000_000, 1000)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    fn submit_proof_with_vk_hash() -> Weight {
        Weight::from_parts(1_000_000, 1000)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    fn register_vk() -> Weight {
        Weight::from_parts(1_000_000, 0)
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}