// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains the parameters related to inflation and dealing with fee.

use super::*;
pub use sp_runtime::{Percent, Perquintill};

parameter_types! {
    /// The target precions for exp(); impacts on the final precision for inflation computation.
    pub const ExpPrecision: f64 = 10e-15f64;
    /// Base inflation (I_b).
    pub InflationBase: Perquintill = Perquintill::from_rational(25u128, 1000u128);
    /// The optimal staking rate (s_t).
    pub StakingTarget: Percent = Percent::from_percent(50);
    /// Sensitivity coefficient (K).
    pub const K: f64 = 1f64;
    /// Multiplier (C).
    pub const C: f64 = 0f64; // zero I_var
    /// Percentage of the minted tokens that goes to the validators (leaving the rest to the
    /// others).
    pub EraPayoutValidatorsSplit: Percent = Percent::from_percent(100);

    /// Fees split between author and treasury.
    pub FeesAuthorSplit: Percent = Percent::from_percent(100);
    /// Burned fees.
    pub FeesBurnSplit: Percent = Percent::from_percent(0);
}
