//! The common pallet-verifiers component.

pub use pallet::*;

use frame_support::weights::Weight;
use sp_core::Get;

/// Weight functions needed for `pallet_aggregate`.
#[allow(missing_docs)]
pub trait WeightInfo {
    fn disable_verifier() -> Weight;
    fn on_verify_disabled_verifier() -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
    use super::WeightInfo;

    #[pallet::pallet]
    /// The common pallet-verifiers component.
    pub struct Pallet<T>(_);

    #[pallet::composite_enum]
    /// A reason for this pallet placing a hold on funds.
    pub enum HoldReason {
        /// The funds are held as storage deposit for a verification key registration.
        VkRegistration,
    }

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Weights
        type CommonWeightInfo: WeightInfo;
    }
}

/// The implementation is quite rude now but should be fine. We implement this fixed weight
/// for each runtime. Obviously we can write a real benchmark later to use in the final
/// runtime but it should never be lot so far from this.
impl<T: frame_system::Config> WeightInfo for T {
    fn disable_verifier() -> Weight {
        T::DbWeight::get().writes(1_u64)
    }

    fn on_verify_disabled_verifier() -> Weight {
        T::DbWeight::get().reads(1_u64)
    }
}
