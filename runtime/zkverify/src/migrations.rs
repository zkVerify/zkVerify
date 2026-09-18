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

//! This module contains the code for all the current and past runtime migrations.

use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{
    traits::{Currency, Get, OnRuntimeUpgrade},
    weights::Weight,
    BoundedVec,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_vesting::{MaxVestingSchedulesGet, VestingInfo};
use sp_runtime::{traits::BlockNumberProvider, SaturatedConversion, Saturating};

#[cfg(feature = "try-runtime")]
use codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use frame_support::ensure;
#[cfg(feature = "try-runtime")]
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, Convert},
    TryRuntimeError,
};

type BalanceOf<T> = <<T as pallet_vesting::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

type SchedulesOf<T> =
    BoundedVec<VestingInfo<BalanceOf<T>, BlockNumberFor<T>>, MaxVestingSchedulesGet<T>>;

/// Migrate the vesting schedules from block numbers to slots elapsed since genesis.
///
/// `pallet_vesting`'s `BlockNumberProvider` moved from `frame_system` (block numbers) to
/// [`crate::SlotNumber`] (slots numbers).
pub struct VestingConfiguration<T>(PhantomData<T>);

impl<T: pallet_vesting::Config + pallet_babe::Config> VestingConfiguration<T> {
    /// Distance between the new block number provider and the old one (`frame_system`).
    fn offset() -> BlockNumberFor<T> {
        u64::from(pallet_babe::Pallet::<T>::genesis_slot()).saturated_into()
    }
}

impl<T: pallet_vesting::Config + pallet_babe::Config> OnRuntimeUpgrade for VestingConfiguration<T> {
    fn on_runtime_upgrade() -> Weight {
        if frame_system::Pallet::<T>::last_runtime_upgrade_spec_version() != 2_000_000 {
            return Default::default();
        }
        let offset = Self::offset();
        let mut accounts = 0u64;

        pallet_vesting::Vesting::<T>::translate(|_who, schedules: SchedulesOf<T>| {
            accounts.saturating_inc();
            let shifted = schedules
                .into_iter()
                .map(|schedule| {
                    VestingInfo::new(
                        schedule.locked(),
                        schedule.per_block(),
                        schedule.starting_block().saturating_add(offset),
                    )
                })
                .collect::<Vec<_>>();
            // The schedules are just rewritten in place: their number, and hence the bound on
            // the vector, cannot change.
            Some(BoundedVec::truncate_from(shifted))
        });

        log::info!(
            target: "runtime::migration",
            "VestingConfiguration: shifted the schedules of {accounts} account(s) by {offset:?}",
        );

        // One read for the block number of each provider (the system one, and the BABE current
        // and genesis slots), plus a read and a write for every migrated account.
        <T as frame_system::Config>::DbWeight::get()
            .reads_writes(accounts.saturating_add(3), accounts)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
        Ok(pallet_vesting::Vesting::<T>::iter()
            .collect::<Vec<_>>()
            .encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
        let before = <Vec<(T::AccountId, SchedulesOf<T>)>>::decode(&mut state.as_slice())
            .map_err(|_| "VestingConfiguration: cannot decode the pre-upgrade state")?;

        ensure!(
            before.len() == pallet_vesting::Vesting::<T>::iter().count(),
            "VestingConfiguration: the number of vesting accounts changed"
        );

        let offset = Self::offset();
        let now = T::BlockNumberProvider::current_block_number();
        let elapsed_slots = now.saturating_sub(offset);
        let produced_blocks = frame_system::Pallet::<T>::block_number();

        // Every schedule moves forward by _drift_ units at once, and keeps the wall clock from then on.
        let drift = elapsed_slots.saturating_sub(produced_blocks);

        log::info!(
            target: "runtime::migration",
            "VestingConfiguration: {elapsed_slots:?} slot(s) elapsed since genesis against \
             {produced_blocks:?} block(s) produced: every schedule vests {drift:?} unit(s) further",
        );

        for (who, schedules_before) in before {
            let schedules_after = pallet_vesting::Vesting::<T>::get(&who)
                .ok_or("VestingConfiguration: an account lost its vesting schedules")?;
            ensure!(
                schedules_after.len() == schedules_before.len(),
                "VestingConfiguration: an account lost some of its vesting schedules"
            );

            for (after, before) in schedules_after.iter().zip(schedules_before.iter()) {
                ensure!(
                    after.locked() == before.locked() && after.per_block() == before.per_block(),
                    "VestingConfiguration: the amount or the rate of a schedule changed"
                );
                ensure!(
                    before.starting_block().saturating_add(offset) == after.starting_block(),
                    "VestingConfiguration: a schedule was not shifted by the genesis slot"
                );

                let locked_after = after.locked_at::<T::BlockNumberToBalance>(now);

                // Reconciling the drift can only bring a schedule forward, and by no more than
                // the missed slots are worth (less, once it is fully vested).
                let locked_before = before.locked_at::<T::BlockNumberToBalance>(produced_blocks);
                let reconciled = locked_before
                    .checked_sub(&locked_after)
                    .ok_or("VestingConfiguration: a schedule vests less than it did before")?;
                ensure!(
                    reconciled
                        <= T::BlockNumberToBalance::convert(drift)
                            .saturating_mul(before.per_block()),
                    "VestingConfiguration: a schedule vests more than the drift is worth"
                );
            }
        }

        Ok(())
    }
}

pub type Unreleased = (VestingConfiguration<crate::Runtime>, ());
