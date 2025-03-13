// Copyright 2024, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! This module tests the correct computation of rewards for validators.

use super::*;
use crate::{payout::*, AccountId, Balance, Balances, Perbill, Runtime, Treasury, ACME};
use pallet_staking::EraPayout;
use sp_runtime::traits::Convert;

#[test]
fn check_params_sanity() {
    // staking_target should be at least 0.5 (and so 0 <= s_c / s_t <= 2)
    assert!(
        StakingTarget::get() * 10u16 >= 5u16,
        "too low staking target"
    );
    // base inflation is not too high
    assert!(
        InflationBase::get() * 1000u64 == 25u64, // 2.5%
        "unexpected base inflation"
    );
}

#[test]
fn check_era_rewards() {
    const ERA_DURATION_MILLIS: u64 = 1000 * 60 * 60 * 24 * 36525 / 100; // 1 year era
    const TOT_ISSUANCE: Balance = 1_000_000_000 * ACME;
    let others_split = Percent::from_percent(100) - ValidatorsSplit::get();

    // Check the reward for an empty era.
    assert_eq!(
        <Runtime as pallet_staking::Config>::EraPayout::era_payout(0, 0, ERA_DURATION_MILLIS),
        (0, 0)
    );

    // Check the reward for a normal era, s_c == s_t  ==> I_var == 1%
    let tot_staked: Balance = Perbill::from_percent(50) * TOT_ISSUANCE;
    let expected_inflation: u128 =
        (InflationBase::get() + Perquintill::from_float(C::get() * 0.01)) * TOT_ISSUANCE;
    assert_eq!(
        <Runtime as pallet_staking::Config>::EraPayout::era_payout(
            tot_staked,
            TOT_ISSUANCE,
            ERA_DURATION_MILLIS
        ),
        (
            ValidatorsSplit::get() * expected_inflation,
            others_split * expected_inflation
        )
    );

    // Check the reward for a normal era, s_c == 0.0 (min)  ==> I_var == 2.718281828459%
    let tot_staked: Balance = Perbill::from_percent(0) * TOT_ISSUANCE;
    let expected_inflation: u128 = (InflationBase::get()
        + Perquintill::from_float(C::get() * 0.02718281828459))
        * TOT_ISSUANCE;
    assert_eq!(
        <Runtime as pallet_staking::Config>::EraPayout::era_payout(
            tot_staked,
            TOT_ISSUANCE,
            ERA_DURATION_MILLIS
        ),
        (
            ValidatorsSplit::get() * expected_inflation,
            others_split * expected_inflation
        )
    );

    // Check the reward for a normal era, s_c == 1.0 (max)  ==> I_var == 5.367879441171%
    let tot_staked: Balance = Perbill::from_percent(100) * TOT_ISSUANCE;
    let expected_inflation: u128 = (InflationBase::get()
        + Perquintill::from_float(C::get() * 0.00367879441171))
        * TOT_ISSUANCE;
    assert_eq!(
        <Runtime as pallet_staking::Config>::EraPayout::era_payout(
            tot_staked,
            TOT_ISSUANCE,
            ERA_DURATION_MILLIS
        ),
        (
            ValidatorsSplit::get() * expected_inflation,
            others_split * expected_inflation
        )
    );

    // Check the reward for an era with half the duration, s_c == 1.0 (max)
    let tot_staked: Balance = Perbill::from_percent(100) * TOT_ISSUANCE;
    let expected_inflation: u128 = (InflationBase::get()
        + Perquintill::from_float(C::get() * 0.00367879441171))
        * TOT_ISSUANCE
        / 2;
    assert_eq!(
        <Runtime as pallet_staking::Config>::EraPayout::era_payout(
            tot_staked,
            TOT_ISSUANCE,
            ERA_DURATION_MILLIS / 2
        ),
        (
            ValidatorsSplit::get() * expected_inflation,
            others_split * expected_inflation
        )
    );

    // Check the reward for an era with double the duration, s_c == 1.0 (max)
    let tot_staked: Balance = Perbill::from_percent(100) * TOT_ISSUANCE;
    let expected_inflation: u128 = (InflationBase::get()
        + Perquintill::from_float(C::get() * 0.00367879441171))
        * TOT_ISSUANCE;
    assert_eq!(
        <Runtime as pallet_staking::Config>::EraPayout::era_payout(
            tot_staked,
            TOT_ISSUANCE,
            ERA_DURATION_MILLIS * 2
        ),
        // capped at 1 year
        (
            ValidatorsSplit::get() * expected_inflation,
            others_split * expected_inflation
        )
    );

    // Check the reward for an era with zero duration, s_c == 1.0 (max)
    let tot_staked: Balance = Perbill::from_percent(100) * TOT_ISSUANCE;
    assert_eq!(
        <Runtime as pallet_staking::Config>::EraPayout::era_payout(tot_staked, TOT_ISSUANCE, 0),
        (0, 0)
    );

    // Check that tot_issuance is actually used, s_c == s_t
    let half_tot_issuance: Balance = TOT_ISSUANCE / 2;
    let tot_staked: Balance = Perbill::from_percent(50) * half_tot_issuance;
    let expected_inflation: u128 =
        (InflationBase::get() + Perquintill::from_float(C::get() * 0.01)) * half_tot_issuance;
    assert_eq!(
        <Runtime as pallet_staking::Config>::EraPayout::era_payout(
            tot_staked,
            half_tot_issuance,
            ERA_DURATION_MILLIS
        ),
        (
            ValidatorsSplit::get() * expected_inflation,
            others_split * expected_inflation
        )
    );
}

#[test]
fn deal_with_fees() {
    super::test().execute_with(|| {
        let fee_amount = ACME;
        let tip_amount = ACME;
        let fee = Balances::issue(fee_amount);
        let tip = Balances::issue(tip_amount);

        let author_account: AccountId = testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize]
            .raw_account
            .into();
        let author_balance = testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].starting_balance;
        assert_eq!(Balances::free_balance(Treasury::account_id()), 0);
        assert_eq!(
            Balances::free_balance(author_account.clone()),
            author_balance
        );

        DealWithFees::on_unbalanceds([fee, tip].into_iter());
        // FeesValidatorsSplit of the fee and all the tips go to the block author
        assert_eq!(
            Balances::free_balance(author_account),
            author_balance + FeesValidatorsSplit::get() * fee_amount + tip_amount
        );

        // The rest of the fees goes to the Treasury (as hardcoded in DealWithFees)
        assert_eq!(
            Balances::free_balance(Treasury::account_id()),
            (Percent::from_percent(100) - FeesValidatorsSplit::get()) * fee_amount
        );
    })
}

#[test]
fn block_cost_after_k_full_blocks() {
    super::test().execute_with(|| {
        // We check that after k full blocks, the fee multiplier is ~26.67, so that filling a block
        // completely costs ~200ACME, considering time only.
        let mut mul: Multiplier = 1.into();
        let k = 100;
        let final_mul = 26.67f64;
        System::set_block_consumed_resources(
            BlockWeights::get()
                .get(DispatchClass::Normal)
                .max_total
                .unwrap(),
            0,
        );
        for _i in 0..k {
            mul = ZKVFeeUpdate::<Runtime>::convert(mul);
        }

        assert!((mul.to_float() - final_mul).abs() < 1e-9f64);
    })
}

#[test]
fn block_cost_after_k_empty_blocks() {
    super::test().execute_with(|| {
        let mut mul: Multiplier = 1.into();
        // We check that after k empty blocks, the fee multiplier never goes below the minimum.
        let k = 100;
        System::set_block_consumed_resources(0.into(), 0);
        for _i in 0..k {
            mul = ZKVFeeUpdate::<Runtime>::convert(mul);
        }

        assert_eq!(mul, MinimumMultiplier::get());
    })
}
