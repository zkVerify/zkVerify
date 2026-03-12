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
//! This migration re-encodes all existing VKs as the `Vk::Intel` variant.

use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::{migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade};
use sp_core::Get;

/// Old `Vk` type (struct, before the Nitro variant was added).
#[derive(Decode)]
struct OldVk {
    tcb_response: Vec<u8>,
    certificates: Vec<u8>,
}

/// SCALE-compatible with `pallet_verifiers::VkEntry<OldVk>`.
#[derive(Decode)]
struct OldVkEntry {
    vk: OldVk,
    ref_count: u64,
}

/// SCALE-compatible with `pallet_verifiers::VkEntry<crate::Vk>`.
#[derive(Encode)]
struct NewVkEntry {
    vk: crate::Vk,
    ref_count: u64,
}

pub struct InnerMigrateV1ToV2<T>(core::marker::PhantomData<T>);

impl<T> UncheckedOnRuntimeUpgrade for InnerMigrateV1ToV2<T>
where
    T: pallet_verifiers::Config<crate::Tee<T>> + crate::Config,
{
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let mut count: u64 = 0;
        pallet_verifiers::Vks::<T, crate::Tee<T>>::translate::<OldVkEntry, _>(|_key, old| {
            count += 1;
            let new = NewVkEntry {
                vk: crate::Vk::Intel {
                    tcb_response: old.vk.tcb_response,
                    certificates: old.vk.certificates,
                },
                ref_count: old.ref_count,
            };
            let encoded = new.encode();
            pallet_verifiers::VkEntry::<crate::Vk>::decode(&mut &encoded[..]).ok()
        });
        log::info!("tee-verifier migration v1->v2: migrated {count} VKs to Intel variant");
        T::DbWeight::get().reads_writes(count + 1, count + 1)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        let count = pallet_verifiers::Vks::<T, crate::Tee<T>>::iter_keys().count() as u64;
        log::info!("tee-verifier pre_upgrade v1->v2: {count} VKs found");
        Ok(count.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        let expected =
            u64::decode(&mut state.as_slice()).map_err(|_| "Failed to decode pre_upgrade state")?;
        let actual = pallet_verifiers::Vks::<T, crate::Tee<T>>::iter().count() as u64;
        frame_support::ensure!(
            actual == expected,
            "tee-verifier post_upgrade v1->v2: VK count mismatch (expected {expected}, got {actual})"
        );
        log::info!("tee-verifier post_upgrade v1->v2: OK, {actual} VKs decoded successfully");
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
