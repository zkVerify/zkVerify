// Copyright 2025, Horizen Labs, Inc.

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

//! Storage migration for the UltraHonk verifier pallet from V1 to V2.
//!
//! In V1, the `Vk` associated type was `[u8; VK_SIZE]` (a fixed-size byte array
//! from `ultrahonk-no-std` v0.84).
//!
//! In V2, the `Vk` type changed to [`VersionedVk`](crate::VersionedVk), an enum
//! supporting multiple protocol versions (`V0_84` and `V3_0`). Because the SCALE
//! encoding changed, existing `Vks` entries cannot be decoded with the new type.
//! VK hashes are also different under the new encoding, so `Tickets` entries
//! (keyed by VK hash) are stale.
//!
//! This migration drains all existing `Vks` and `Tickets` storage entries for the
//! UltraHonk verifier instance.

use alloc::vec::Vec;
use frame_support::traits::Consideration;
use frame_support::{
    migrations::VersionedMigration, storage_alias, traits::UncheckedOnRuntimeUpgrade, Identity,
};
use pallet_verifiers::VkEntry;
use sp_core::Get;
use sp_core::H256;

use crate::Ultrahonk;

/// Implements [`UncheckedOnRuntimeUpgrade`], migrating the storage from V1 to V2.
///
/// Drains all `Vks` and `Tickets` entries for the UltraHonk verifier. The old VK
/// type (`[u8; VK_SIZE]`) is incompatible with the new [`VersionedVk`](crate::VersionedVk),
/// so existing entries must be removed. Users will need to re-register their
/// verification keys after the migration.
pub struct InnerMigrateV1ToV2<T>(core::marker::PhantomData<T>);

#[storage_alias]
type OldVk<T: crate::Config + pallet_verifiers::Config<Ultrahonk<T>>> =
    StorageMap<crate::Pallet<T>, Identity, H256, VkEntry<Vec<u8>>>;

impl<T> UncheckedOnRuntimeUpgrade for InnerMigrateV1ToV2<T>
where
    T: pallet_verifiers::Config<crate::Ultrahonk<T>> + crate::Config,
{
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let mut reads: u64 = 0;
        let mut writes: u64 = 0;
        let vks = OldVk::<T>::drain().count() as u64;
        let tickets = pallet_verifiers::Tickets::<T, crate::Ultrahonk<T>>::drain()
            .map(|((account_id, _), ticket)| {
                if ticket.drop(&account_id).is_err() {
                    log::warn!("Error encountered when dropping ticket");
                }
            })
            .count() as u64;

        reads += vks + tickets;
        writes += vks + tickets;

        log::info!(
            target: "runtime::ultrahonk",
            "UltraHonk migration V1->V2: drained {} Vks and {} Tickets entries",
            vks,
            tickets,
        );

        T::DbWeight::get().reads_writes(reads, writes)
    }
}

/// [`UncheckedOnRuntimeUpgrade`] implementation [`InnerMigrateV1ToV2`] wrapped in a
/// [`VersionedMigration`], which ensures that:
/// - The migration only runs once when the on-chain storage version is 1
/// - The on-chain storage version is updated to `2` after the migration executes
/// - Reads/Writes from checking/setting the on-chain storage version are accounted for
pub type MigrateV1ToV2<T> = VersionedMigration<
    1, // The migration will only execute when the on-chain storage version is 1
    2, // The on-chain storage version will be set to 2 after the migration is complete
    InnerMigrateV1ToV2<T>,
    crate::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;
