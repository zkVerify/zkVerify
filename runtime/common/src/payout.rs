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

//! This module contains custom implementations that regulate the distribution of tokens
//! following inflation and fees/tips.
//!
//! Concretely, ZKVPayout implements the pallet_staking::EraPayout trait, which
//! provides a custom inflation model which tries to drive the staking rate toward an ideal target.
//! Details on the actual formula and its parameters are given below.
//!
//! DealWithFees, instead, implements the OnUnbalanced trait, and redirects fees and tips to the
//! block author and to the treasury with a configurable ratio.

use super::*;
use core::default::Default;
use frame_support::traits::fungible::Balanced;
use frame_support::traits::fungible::Credit;
use frame_support::traits::tokens::imbalance::ResolveTo;
use frame_support::traits::Imbalance;
use frame_support::traits::OnUnbalanced;
use sp_core::Get;
use sp_runtime::traits::{AtLeast32BitUnsigned, Zero};
use sp_runtime::Perbill;
pub use sp_runtime::{Percent, Perquintill};

fn abs(v: f64) -> f64 {
    if v > 0f64 {
        v
    } else {
        -v
    }
}

/// Implements a custom inflation model based on the following formula:
/// I(s_c) = I_b + I_v(s_c)
///
/// with:
/// I_v(x) = C * exp(K * (1 - (s_c / s_t)))
///
/// where:
/// I_b: base inflation
/// I_v: variable inflation
/// s_t: target staking rate
/// K: sensitivity coefficient
/// C: multiplier
pub trait InflationModel {
    /// The target precision for exp(); impacts on the final precision for inflation computation.
    type ExpPrecision: Get<f64>;
    /// Base inflation (I_b).
    type InflationBase: Get<Perquintill>;
    /// The optimal staking rate (s_t).
    type StakingTarget: Get<Percent>;
    /// Sensitivity coefficient (K).
    type K: Get<f64>;
    /// Multiplier (C).
    type C: Get<f64>;
}

trait InflationArithmetic: InflationModel {
    fn exp_precision() -> f64 {
        <Self as InflationModel>::ExpPrecision::get()
    }

    fn inflation_base() -> Perquintill {
        <Self as InflationModel>::InflationBase::get()
    }

    fn c() -> f64 {
        <Self as InflationModel>::C::get()
    }

    fn k() -> f64 {
        <Self as InflationModel>::K::get()
    }

    /// Sums: 1 + x + x^2/2! + x^3/3! + x^4/4! + x^5/5! + ...
    /// until the error goes below ExpPrecision.
    /// If |p| <= 1, the number of terms (iterations) is limited by n s.t. 1/n! <= ExpPrecision.
    /// For example, if ExpPrecision == 10e-15 => n == 18
    fn exp(p: f64) -> f64 {
        let mut res: f64 = 0f64;
        let mut next = 1f64;
        let mut i = 1f64;
        while abs(next) > Self::exp_precision() {
            res += next;
            next = (next * p) / i;
            i += 1f64;
        }
        res
    }

    fn f64_to_inflation(p: f64) -> Perquintill {
        Perquintill::from_rational(
            (p * (0.01f64 / Self::exp_precision())) as u64,
            (1f64 / Self::exp_precision()) as u64,
        )
    }
}

impl<T: InflationModel> InflationArithmetic for T {}

const MILLISECS_PER_YEAR: u64 = 1000 * 60 * 60 * 24 * 36525 / 100;

trait Inflation<Balance: AtLeast32BitUnsigned + Zero + Copy + From<u64>>: InflationArithmetic {
    fn compute_inflation(
        total_staked: Balance,
        total_issuance: Balance,
        era_duration_millis: u64,
    ) -> Balance {
        if total_issuance == Balance::zero() {
            return Balance::zero();
        }

        let scale: f64 = 1f64 / Self::exp_precision();
        let time_portion = Perquintill::from_rational(era_duration_millis, MILLISECS_PER_YEAR);

        let staking_current: Perbill = Perbill::from_rational(total_staked, total_issuance);

        // s = s_c / s_t
        let s = <Self as InflationModel>::StakingTarget::get()
            .saturating_reciprocal_mul(staking_current * scale as u128);

        // exp_arg = k * (1 - s)
        let inflation_arg = if Self::c() != 0f64 {
            let exp_arg = (scale - s as f64) * Self::k() / scale;
            // inflation_arg = C * e^(exp_arg)
            Self::c() * Self::exp(exp_arg)
        } else {
            // will be zeroed by C, so do not bother to compute exp
            0f64
        };

        let inflation_var = Self::f64_to_inflation(inflation_arg);

        time_portion * (Self::inflation_base() + inflation_var) * total_issuance
    }
}

impl<R: InflationModel, B: AtLeast32BitUnsigned + Copy + Default + From<u64>> Inflation<B> for R {}

pub struct ZKVPayout<Model: InflationModel, Splitter: Get<Percent>>(
    core::marker::PhantomData<(Model, Splitter)>,
);

impl<Model: InflationModel, Splitter: Get<Percent>> pallet_staking::EraPayout<Balance>
    for ZKVPayout<Model, Splitter>
{
    /// Calculates the validators reward based on the duration of the era.
    fn era_payout(
        total_staked: Balance,
        total_issuance: Balance,
        era_duration_millis: u64,
    ) -> (Balance, Balance) {
        let inflation_tot =
            Model::compute_inflation(total_staked, total_issuance, era_duration_millis);
        let to_author = Splitter::get() * inflation_tot;
        let other = (Percent::from_percent(100) - Splitter::get()) * inflation_tot;
        (to_author, other)
    }
}

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(core::marker::PhantomData<R>);

impl<R> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>> for ToAuthor<R>
where
    R: pallet_balances::Config + pallet_authorship::Config,
    <R as frame_system::Config>::AccountId: From<polkadot_primitives::AccountId>,
    <R as frame_system::Config>::AccountId: Into<polkadot_primitives::AccountId>,
{
    fn on_nonzero_unbalanced(
        amount: Credit<<R as frame_system::Config>::AccountId, pallet_balances::Pallet<R>>,
    ) {
        if let Some(author) = <pallet_authorship::Pallet<R>>::author() {
            let _ = <pallet_balances::Pallet<R>>::resolve(&author, amount);
        }
    }
}
pub struct DealWithFees<R, FeesBurnSplit: Get<Percent>, FeesAuthorSplit: Get<Percent>>(
    core::marker::PhantomData<(R, FeesBurnSplit, FeesAuthorSplit)>,
);
impl<R, FeesBurnSplit, FeesAuthorSplit>
    OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>>
    for DealWithFees<R, FeesBurnSplit, FeesAuthorSplit>
where
    R: pallet_balances::Config + pallet_authorship::Config + pallet_treasury::Config,
    <R as frame_system::Config>::AccountId: From<polkadot_primitives::AccountId>,
    <R as frame_system::Config>::AccountId: Into<polkadot_primitives::AccountId>,
    FeesBurnSplit: Get<Percent>,
    FeesAuthorSplit: Get<Percent>,
{
    fn on_unbalanceds(
        mut fees_then_tips: impl Iterator<Item = Credit<R::AccountId, pallet_balances::Pallet<R>>>,
    ) {
        if let Some(fees) = fees_then_tips.next() {
            let burn_split = FeesBurnSplit::get() * 100u32;
            let fees_split = 100u32 - burn_split;
            let (_burned, fees) = fees.ration(burn_split, fees_split);
            // for unburned fees FeesValidatorsSplit% goes to authors and the rest
            let val_split = FeesAuthorSplit::get() * 100u32;
            let treasury_split = 100u32 - val_split;
            let (treasury, mut author) = fees.ration(treasury_split, val_split);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any, 100% to author
                tips.merge_into(&mut author);
            }
            ResolveTo::<pallet_treasury::TreasuryAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(treasury);
            <ToAuthor<R> as OnUnbalanced<_>>::on_unbalanced(author);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pallet_staking::EraPayout;
    use rstest::rstest;

    macro_rules! getter {
        ( const, $name:ident, $t:ty, $v:expr ) => {
            getter!($name, $t, $name::get());

            impl $name {
                const fn get() -> $t {
                    $v
                }
            }
        };
        ( $name:ident, $t:ty, $v:expr ) => {
            #[derive(Default, Clone)]
            struct $name;

            impl Get<$t> for $name {
                fn get() -> $t {
                    $v
                }
            }
        };
    }

    #[rstest]
    #[case::year(1_000_000_000_000_000_000, MILLISECS_PER_YEAR, 25_000_000_000_000_000)]
    #[case::half_year(1_000_000_000, MILLISECS_PER_YEAR/2, 12_500_000)]
    #[case::day(1_000_000_000_000_000_000_000_000, MILLISECS_PER_YEAR * 100 / 36525, 68446269678302000000
    )]
    #[case::era(1_000_000_000_000_000_000_000_000, 1_000 * 60 * 60 * 6, 17111567419575000000)]
    fn no_staking_inflation(#[case] supply: u128, #[case] elapsed: u64, #[case] expected: u128) {
        struct TestModel;

        getter!(const, Precision, f64, 10e-15f64);
        getter!(
            InflationBase,
            Perquintill,
            Perquintill::from_rational(25u128, 1000u128)
        );
        getter!(StakingTarget, Percent, Percent::from_percent(50));
        getter!(const, K, f64, 1_f64);
        getter!(const, C, f64, 0_f64);

        impl InflationModel for TestModel {
            type ExpPrecision = Precision;
            type InflationBase = InflationBase;
            type StakingTarget = StakingTarget;
            type K = K;
            type C = C;
        }

        let computed = TestModel::compute_inflation(0_u128, supply, elapsed);
        assert_eq!(expected, computed);
    }

    #[rstest]
    #[case::year_in_target(
        1_000_000_000_000_000_000,
        Percent::from_percent(50),
        MILLISECS_PER_YEAR,
        26_000_000_000_000_000
    )]
    #[case::year_below_target(
        1_000_000_000_000_000_000,
        Percent::from_percent(30),
        MILLISECS_PER_YEAR,
        26_491_824_697_640_000
    )]
    #[case::year_over_target(
        1_000_000_000_000_000_000,
        Percent::from_percent(70),
        MILLISECS_PER_YEAR,
        25_670_320_046_030_000
    )]
    #[case::era_in_target(1_000_000_000_000_000_000_000_000, Percent::from_percent(50), 1_000 * 60 * 60 * 6, 17_796_030_116_358_000_000
    )]
    #[case::era_below_target(1_000_000_000_000_000_000_000_000, Percent::from_percent(20), 1_000 * 60 * 60 * 6, 18_358_739_767_549_000_000
    )]
    #[case::era_over_target(1_000_000_000_000_000_000_000_000, Percent::from_percent(80), 1_000 * 60 * 60 * 6, 17_487_208_512_039_000_000
    )]
    fn staking_aware_inflation(
        #[case] supply: u128,
        #[case] staked: Percent,
        #[case] elapsed: u64,
        #[case] expected: u128,
    ) {
        struct TestModel;

        getter!(const, Precision, f64, 10e-15f64);
        getter!(
            InflationBase,
            Perquintill,
            Perquintill::from_rational(25u128, 1000u128)
        );
        getter!(StakingTarget, Percent, Percent::from_percent(50));
        getter!(const, K, f64, 1_f64);
        getter!(const, C, f64, 0.1_f64); // Variable contribution

        impl InflationModel for TestModel {
            type ExpPrecision = Precision;
            type InflationBase = InflationBase;
            type StakingTarget = StakingTarget;
            type K = K;
            type C = C;
        }

        let staked_amount = staked * supply;

        let computed = TestModel::compute_inflation(staked_amount, supply, elapsed);
        assert_eq!(expected, computed);
    }

    #[test]
    fn era_payout() {
        struct TestModel;

        getter!(const, Precision, f64, 10e-15f64);
        getter!(
            InflationBase,
            Perquintill,
            Perquintill::from_rational(25u128, 1000u128)
        );
        getter!(StakingTarget, Percent, Percent::from_percent(50));
        getter!(const, K, f64, 1_f64);
        getter!(const, C, f64, 0_f64); // Staked doesn't mater

        impl InflationModel for TestModel {
            type ExpPrecision = Precision;
            type InflationBase = InflationBase;
            type StakingTarget = StakingTarget;
            type K = K;
            type C = C;
        }

        getter!(Splitter, Percent, Percent::from_percent(90));

        let (author_amount, other_amount) = ZKVPayout::<TestModel, Splitter>::era_payout(
            0,
            1_000_000_000_000_000_000,
            MILLISECS_PER_YEAR,
        );

        let tot = author_amount + other_amount;

        assert_eq!(tot, 25_000_000_000_000_000);
        assert_eq!(Splitter::get(), Percent::from_rational(author_amount, tot));
    }

    #[test]
    fn check_exp() {
        struct TestModel;

        getter!(const, Precision, f64, 10e-15f64);
        getter!(FakeInflationBase, Perquintill, Perquintill::from_percent(0));
        getter!(FakeStakingTarget, Percent, Percent::from_percent(50));
        getter!(const, K, f64, 1_f64);
        getter!(const, C, f64, 0_f64);

        impl InflationModel for TestModel {
            type ExpPrecision = Precision;
            type InflationBase = FakeInflationBase;
            type StakingTarget = FakeStakingTarget;
            type K = K;
            type C = C;
        }

        const TEST_VALUES: [f64; 9] = [
            -K::get(),
            -1f64,
            -0.5f64,
            -Precision::get(),
            0f64,
            Precision::get(),
            0.5f64,
            1f64,
            K::get(),
        ];

        for v in TEST_VALUES {
            assert!(
                (<TestModel as InflationArithmetic>::exp(v) - v.exp()).abs() <= Precision::get(),
                "failed for {v}"
            );
        }
    }
}
