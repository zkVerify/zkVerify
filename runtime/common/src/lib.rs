#![cfg_attr(not(feature = "std"), no_std)]

pub mod genesis_config_presets_helper;
pub use genesis_config_presets_helper::*;

pub mod types;
pub use types::*;

mod payout;

pub use payout::{DealWithFees, InflationModel, ToAuthor, ZKVPayout};

extern crate alloc;

#[macro_export]
macro_rules! prod_or_fast {
    ($prod:expr, $test:expr) => {
        if cfg!(feature = "fast-runtime") {
            $test
        } else {
            $prod
        }
    };
    ($prod:expr, $test:expr, $env:expr) => {
        if cfg!(feature = "fast-runtime") {
            core::option_env!($env)
                .map(|s| s.parse().ok())
                .flatten()
                .unwrap_or($test)
        } else {
            $prod
        }
    };
}
