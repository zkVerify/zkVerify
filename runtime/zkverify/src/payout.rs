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

use crate::currency::Balance;
use core::default::Default;
use frame_support::traits::{
    fungible::{Balanced, Credit},
    tokens::imbalance::ResolveTo,
    Imbalance, OnUnbalanced,
};
use sp_core::Get;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Zero},
    Perbill, Percent, Perquintill,
};

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

pub trait AuthorResolve<AccountId> {
    fn author() -> Option<AccountId>;
}

impl<P: pallet_authorship::Config> AuthorResolve<<P as frame_system::Config>::AccountId>
    for pallet_authorship::Pallet<P>
{
    fn author() -> Option<<P as frame_system::Config>::AccountId> {
        <pallet_authorship::Pallet<P>>::author()
    }
}

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R, AuthorResolve>(core::marker::PhantomData<(R, AuthorResolve)>);

impl<R, Author> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>>
    for ToAuthor<R, Author>
where
    R: pallet_balances::Config,
    <R as frame_system::Config>::AccountId: From<polkadot_primitives::AccountId>,
    <R as frame_system::Config>::AccountId: Into<polkadot_primitives::AccountId>,
    Author: AuthorResolve<<R as frame_system::Config>::AccountId>,
{
    fn on_nonzero_unbalanced(
        amount: Credit<<R as frame_system::Config>::AccountId, pallet_balances::Pallet<R>>,
    ) {
        if let Some(author) = Author::author() {
            let _ = <pallet_balances::Pallet<R>>::resolve(&author, amount);
        }
    }
}

/// Deals with fees: burns some of them, distributes the rest to the author and a treasury account.
///   - `FeesBurnSplit`: percentage of fees to burn
///   - `FeesAuthorSplit`: percentage of the unburned fees to send to the author and the rest goes
///     treasury.
///
/// Tip amount goes to the author.
///
/// Example: FeesBurnSplit = 10%, FeesAuthorSplit = 70%, fees = 1000 tip = 500
///   - Burned = `100` <- `1000 * 10%`
///   - To Treasury = `270` <- `(1000-100) * 30%`
///   - To Author = `1130` <- `(1000-100) * 70% + 500`
pub struct DealWithFees<
    R,
    FeesBurnSplit: Get<Percent>,
    FeesAuthorSplit: Get<Percent>,
    Author,
    TreasuryAccount,
>(core::marker::PhantomData<(R, FeesBurnSplit, FeesAuthorSplit, Author, TreasuryAccount)>);
impl<R, FeesBurnSplit, FeesAuthorSplit, Author, TreasuryAccount>
    OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>>
    for DealWithFees<R, FeesBurnSplit, FeesAuthorSplit, Author, TreasuryAccount>
where
    R: pallet_balances::Config,
    <R as frame_system::Config>::AccountId: From<polkadot_primitives::AccountId>,
    <R as frame_system::Config>::AccountId: Into<polkadot_primitives::AccountId>,
    FeesBurnSplit: Get<Percent>,
    FeesAuthorSplit: Get<Percent>,
    Author: AuthorResolve<<R as frame_system::Config>::AccountId>,
    TreasuryAccount: sp_runtime::traits::TypedGet<Type = <R as frame_system::Config>::AccountId>,
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
            ResolveTo::<TreasuryAccount, pallet_balances::Pallet<R>>::on_unbalanced(treasury);
            <ToAuthor<R, Author> as OnUnbalanced<_>>::on_unbalanced(author);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::derive_impl;
    use frame_support::pallet_prelude::ConstU32;
    use frame_support::traits::fungible::Dust;
    use frame_support::traits::ConstU128;
    use pallet_staking::EraPayout;
    use rstest::rstest;
    use sp_runtime::traits::IdentityLookup;
    use sp_runtime::BuildStorage;

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

    pub type Balance = u128;
    pub type AccountId = polkadot_primitives::AccountId;

    frame_support::construct_runtime!(
        pub enum Runtime {
            System: frame_system,
            Balances: pallet_balances,
        }
    );

    impl pallet_balances::Config for Runtime {
        /// The ubiquitous event type.
        type RuntimeEvent = RuntimeEvent;
        type RuntimeHoldReason = ();
        type RuntimeFreezeReason = ();
        type WeightInfo = ();
        /// The type for recording an account's balance.
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = ();
        type MaxLocks = ConstU32<50>;
        type MaxReserves = ();
        type MaxFreezes = ();
        type DoneSlashHandler = ();
    }

    //noinspection RsSortImplTraitMembers
    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig
    )]
    impl frame_system::Config for Runtime {
        type RuntimeEvent = RuntimeEvent;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Block = frame_system::mocking::MockBlockU32<Runtime>;
        type AccountData = pallet_balances::AccountData<Balance>;
    }

    const AUTHOR: [u8; 32] = [1_u8; 32];
    const TREASURY: [u8; 32] = [2_u8; 32];

    struct ToAuthor;

    impl AuthorResolve<AccountId> for ToAuthor {
        fn author() -> Option<AccountId> {
            Some(AUTHOR.into())
        }
    }

    struct ToTreasury;

    impl sp_runtime::traits::TypedGet for ToTreasury {
        type Type = AccountId;
        fn get() -> Self::Type {
            TREASURY.into()
        }
    }

    const AUTHOR_INITIAL_BALANCE: Balance = 1_000_000_000;
    const TREASURY_INITIAL_BALANCE: Balance = 1_000_000_000_000;
    static USERS: &[([u8; 32], Balance)] = &[
        (AUTHOR, AUTHOR_INITIAL_BALANCE),
        (TREASURY, TREASURY_INITIAL_BALANCE),
    ];

    // Build genesis storage according to the mock runtime.
    pub fn test() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Runtime>::default()
            .build_storage()
            .unwrap();
        pallet_balances::GenesisConfig::<Runtime> {
            balances: USERS
                .iter()
                .map(|(a, b)| (AccountId::from(*a), *b))
                .collect(),
            dev_accounts: None,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::from(t);

        ext.execute_with(|| {
            System::set_block_number(1);
        });
        ext
    }

    #[test]
    fn gen_deal_with_fee() {
        getter!(FeesBurnSplit, Percent, Percent::from_percent(10));
        getter!(FeesAuthorSplit, Percent, Percent::from_percent(70));

        type D = DealWithFees<Runtime, FeesBurnSplit, FeesAuthorSplit, ToAuthor, ToTreasury>;

        use frame_support::traits::OnUnbalanced;

        test().execute_with(|| {
            let start_issuance = Balances::total_issuance();
            const FEE: Balance = 1000;
            const TIP: Balance = 500;

            D::on_unbalanceds(
                vec![
                    Dust::<AccountId, pallet_balances::Pallet<Runtime>>(FEE).into_credit(),
                    Dust::<AccountId, pallet_balances::Pallet<Runtime>>(TIP).into_credit(),
                ]
                .into_iter(),
            );

            let expected_burn = FEE * 10 / 100; // 10% of fee
            let remaining_fee = FEE - expected_burn;
            let expected_treasury = remaining_fee * 30 / 100; // 30% of remaining fee
            let expected_author = remaining_fee - expected_treasury + TIP; // 70% of remaining fee + full tip

            assert_eq!(start_issuance - Balances::total_issuance(), expected_burn);
            assert_eq!(
                Balances::free_balance(AccountId::from(TREASURY)) - TREASURY_INITIAL_BALANCE,
                expected_treasury
            );
            assert_eq!(
                Balances::free_balance(AccountId::from(AUTHOR)) - AUTHOR_INITIAL_BALANCE,
                expected_author
            );
        })
    }

    #[test]
    fn trivial_deal_with_fee() {
        getter!(FeesBurnSplit, Percent, Percent::from_percent(0));
        getter!(FeesAuthorSplit, Percent, Percent::from_percent(100));

        type D = DealWithFees<Runtime, FeesBurnSplit, FeesAuthorSplit, ToAuthor, ToTreasury>;

        use frame_support::traits::OnUnbalanced;

        test().execute_with(|| {
            let start_issuance = Balances::total_issuance();
            const FEE: Balance = 1000;
            const TIP: Balance = 500;

            D::on_unbalanceds(
                vec![
                    Dust::<AccountId, pallet_balances::Pallet<Runtime>>(FEE).into_credit(),
                    Dust::<AccountId, pallet_balances::Pallet<Runtime>>(TIP).into_credit(),
                ]
                .into_iter(),
            );

            let expected_burn = 0;
            let expected_treasury = 0;
            let expected_author = FEE + TIP; // 70% of remaining fee + full tip

            assert_eq!(Balances::total_issuance() - start_issuance, expected_burn);
            assert_eq!(
                Balances::free_balance(AccountId::from(AUTHOR)) - AUTHOR_INITIAL_BALANCE,
                expected_author
            );
            assert_eq!(
                Balances::free_balance(AccountId::from(TREASURY)) - TREASURY_INITIAL_BALANCE,
                expected_treasury
            );
        })
    }
}
