// Copyright 2026, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Migrate from storage v1 to v2.
//!
//! In v1, the TEE verifier's `Vk` type was a struct:
//! ```ignore
//! struct Vk {
//!     tcb_response: Vec<u8>,
//!     certificates: Vec<u8>,
//! }
//! ```
//!
//! In v2, it became an enum to support multiple TEE backends:
//! ```ignore
//! enum Vk {
//!     Intel { tcb_response: Vec<u8>, certificates: Vec<u8> },
//!     Nitro,
//! }
//! ```
//!
//! This migration removes all existing VK and Ticket storage items, releasing
//! any held funds, since the old v1 VK encoding is incompatible with the new
//! enum representation.

#[cfg(feature = "try-runtime")]
use alloc::vec::Vec;
#[cfg(feature = "try-runtime")]
use codec::{Decode, Encode};

use frame_support::{
    migrations::VersionedMigration,
    traits::{Consideration, UncheckedOnRuntimeUpgrade},
};
use sp_core::Get;

pub struct InnerMigrateV1ToV2<T>(core::marker::PhantomData<T>);

mod v1 {
    use crate::Tee;
    use alloc::vec::Vec;
    use codec::{Decode, Encode};
    use frame_support::{storage_alias, Identity};
    use sp_core::H256;

    #[derive(Decode, Encode)]
    pub struct OldVk {
        tcb_response: Vec<u8>,
        certificates: Vec<u8>,
    }

    #[derive(Decode, Encode)]
    pub struct OldVkEntry {
        vk: OldVk,
        ref_count: u64,
    }

    #[storage_alias]
    pub type Vks<T: crate::Config + pallet_verifiers::Config<Tee<T>>> =
        StorageMap<crate::Pallet<T>, Identity, H256, OldVkEntry>;
}

impl<T> UncheckedOnRuntimeUpgrade for InnerMigrateV1ToV2<T>
where
    T: pallet_verifiers::Config<crate::Tee<T>> + crate::Config,
{
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let mut reads: u64 = 0;
        let mut writes: u64 = 0;
        let vks = v1::Vks::<T>::drain().count() as u64;
        let tickets = pallet_verifiers::Tickets::<T, crate::Tee<T>>::drain()
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
            "Tee migration V1->V2: drained {} Vks and {} Tickets entries",
            vks,
            tickets,
        );

        T::DbWeight::get().reads_writes(reads, writes)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        let vk_count = pallet_verifiers::Vks::<T, crate::Tee<T>>::iter_keys().count() as u64;
        let ticket_count =
            pallet_verifiers::Tickets::<T, crate::Tee<T>>::iter_keys().count() as u64;
        log::info!(
            "tee-verifier pre_upgrade v1->v2: {vk_count} VKs and {ticket_count} tickets to remove"
        );
        Ok((vk_count, ticket_count).encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        let (pre_vk_count, pre_ticket_count) = <(u64, u64)>::decode(&mut state.as_slice())
            .map_err(|_| "Failed to decode pre_upgrade state")?;

        let vk_count = pallet_verifiers::Vks::<T, crate::Tee<T>>::iter_keys().count() as u64;
        let ticket_count =
            pallet_verifiers::Tickets::<T, crate::Tee<T>>::iter_keys().count() as u64;

        frame_support::ensure!(
            vk_count == 0,
            "tee-verifier post_upgrade v1->v2: expected 0 VKs, got {vk_count} (had {pre_vk_count})"
        );
        frame_support::ensure!(
            ticket_count == 0,
            "tee-verifier post_upgrade v1->v2: expected 0 tickets, got {ticket_count} (had {pre_ticket_count})"
        );

        log::info!(
            "tee-verifier post_upgrade v1->v2: OK, removed {pre_vk_count} VKs and {pre_ticket_count} tickets"
        );
        Ok(())
    }
}

/// [`UncheckedOnRuntimeUpgrade`] implementation [`InnerMigrateV1ToV2`] wrapped in a
/// [`VersionedMigration`], which ensures that:
/// - The migration only runs once when the on-chain storage version is 1
/// - The on-chain storage version is updated to `2` after the migration executes
/// - Reads/Writes from checking/setting the on-chain storage version are accounted for
pub type MigrateV1ToV2<T> = VersionedMigration<
    1,
    2,
    InnerMigrateV1ToV2<T>,
    crate::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;
