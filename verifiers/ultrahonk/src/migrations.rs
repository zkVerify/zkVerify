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
//! supporting multiple protocol versions (`V0_84`, `V3_0`, and `Legacy`).
//!
//! This migration wraps all existing V1 VKs in
//! [`VersionedVk::Legacy`](crate::VersionedVk::Legacy), preserving the original VK
//! hash (SHA2-256 of raw bytes). Because the Legacy variant uses the same hash
//! computation as V1, the storage keys are unchanged and `Tickets` entries remain
//! valid without modification.

#[cfg(feature = "try-runtime")]
use codec::{Decode, Encode};
use frame_support::{migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade};
use sp_core::Get;

/// Implements [`UncheckedOnRuntimeUpgrade`], migrating the storage from V1 to V2.
///
/// Wraps all existing `[u8; VK_SIZE]` VKs in
/// [`VersionedVk::Legacy`](crate::VersionedVk::Legacy). The Legacy variant preserves
/// the original hash computation (SHA2-256 of raw bytes), so the storage key and
/// `Tickets` references remain valid.
pub struct InnerMigrateV1ToV2<T>(core::marker::PhantomData<T>);

mod v1 {
    use codec::{Decode, Encode};
    use frame_support::{storage_alias, Identity};
    use sp_core::H256;

    /// Migration-only struct mirroring `VkEntry<[u8; VK_SIZE]>` with accessible fields.
    #[derive(Decode, Encode)]
    pub struct OldVkEntry {
        pub vk: [u8; ultrahonk_no_std_v0_84::VK_SIZE],
        pub ref_count: u64,
    }

    #[storage_alias]
    pub type Vks<T: crate::Config + pallet_verifiers::Config<crate::Ultrahonk<T>>> =
        StorageMap<crate::Pallet<T>, Identity, H256, OldVkEntry>;
}

mod v2 {
    use crate::VersionedVk;
    use codec::{Decode, Encode};
    use frame_support::{storage_alias, Identity};
    use sp_core::H256;

    /// Migration-only struct mirroring `VkEntry<VersionedVk>` with accessible fields.
    #[derive(Encode, Decode)]
    pub struct NewVkEntry {
        pub vk: VersionedVk,
        pub ref_count: u64,
    }

    #[storage_alias]
    pub type Vks<T: crate::Config + pallet_verifiers::Config<crate::Ultrahonk<T>>> =
        StorageMap<crate::Pallet<T>, Identity, H256, NewVkEntry>;
}

impl<T> UncheckedOnRuntimeUpgrade for InnerMigrateV1ToV2<T>
where
    T: pallet_verifiers::Config<crate::Ultrahonk<T>> + crate::Config,
{
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let mut count: u64 = 0;

        for (hash, old_entry) in v1::Vks::<T>::drain() {
            let new_entry = v2::NewVkEntry {
                vk: crate::VersionedVk::Legacy(old_entry.vk),
                ref_count: old_entry.ref_count,
            };
            v2::Vks::<T>::insert(hash, new_entry);
            count += 1;
        }

        log::info!(
            target: "runtime::ultrahonk",
            "UltraHonk migration V1->V2: migrated {} VK entries to Legacy variant",
            count,
        );

        // Per entry: 1 read (drain) + 1 write (drain delete) + 1 write (insert)
        T::DbWeight::get().reads_writes(count, count.saturating_mul(2))
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<alloc::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
        let vk_count = v1::Vks::<T>::iter_keys().count() as u64;
        log::info!(
            target: "runtime::ultrahonk",
            "ultrahonk pre_upgrade v1->v2: {vk_count} VKs to migrate to Legacy"
        );
        Ok(vk_count.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: alloc::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        let pre_vk_count =
            u64::decode(&mut state.as_slice()).map_err(|_| "Failed to decode pre_upgrade state")?;

        let post_vk_count = v2::Vks::<T>::iter()
            .inspect(|(_, entry)| {
                assert!(
                    matches!(entry.vk, crate::VersionedVk::Legacy(_)),
                    "All migrated VKs should be Legacy variant"
                );
            })
            .count() as u64;

        frame_support::ensure!(
            post_vk_count == pre_vk_count,
            "ultrahonk post_upgrade v1->v2: expected {pre_vk_count} VKs, got {post_vk_count}"
        );

        log::info!(
            target: "runtime::ultrahonk",
            "ultrahonk post_upgrade v1->v2: OK, migrated {post_vk_count} VKs to Legacy"
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
    1, // The migration will only execute when the on-chain storage version is 1
    2, // The on-chain storage version will be set to 2 after the migration is complete
    InnerMigrateV1ToV2<T>,
    crate::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ultrahonk, VersionedVk};
    use frame_support::{
        derive_impl, parameter_types,
        sp_runtime::{traits::IdentityLookup, BuildStorage},
        traits::{fungible::HoldConsideration, LinearStoragePrice, UncheckedOnRuntimeUpgrade},
    };
    use pallet_verifiers::traits::Verifier;
    use sp_core::{ConstU128, ConstU32, H256};

    type Balance = u128;
    type AccountId = u64;

    frame_support::construct_runtime!(
        pub enum Test {
            System: frame_system,
            Balances: pallet_balances,
            CommonVerifiersPallet: pallet_verifiers::common,
            UltrahonkPallet: crate,
        }
    );

    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlockU32<Test>;
        type AccountId = AccountId;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type WeightInfo = ();
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxLocks = ConstU32<10>;
        type MaxReserves = ConstU32<10>;
        type MaxFreezes = ConstU32<10>;
        type DoneSlashHandler = ();
    }

    impl pallet_verifiers::common::Config for Test {
        type CommonWeightInfo = Test;
    }

    parameter_types! {
        pub const BaseDeposit: Balance = 1;
        pub const PerByteDeposit: Balance = 2;
        pub const HoldReasonVkRegistration: RuntimeHoldReason =
            RuntimeHoldReason::CommonVerifiersPallet(
                pallet_verifiers::common::HoldReason::VkRegistration
            );
    }

    impl pallet_verifiers::Config<Ultrahonk<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::UltrahonkWeight<()>;
        type Ticket = HoldConsideration<
            AccountId,
            Balances,
            HoldReasonVkRegistration,
            LinearStoragePrice<BaseDeposit, PerByteDeposit, Balance>,
        >;
        #[cfg(feature = "runtime-benchmarks")]
        type Currency = Balances;
    }

    impl crate::Config for Test {
        type MaxPubs = ConstU32<32>;
        type WeightInfo = ();
    }

    fn test_ext() -> sp_io::TestExternalities {
        let mut ext = sp_io::TestExternalities::from(
            frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .unwrap(),
        );
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    /// Build raw VK bytes from test data resources.
    fn test_vk_bytes() -> [u8; ultrahonk_no_std_v0_84::VK_SIZE] {
        use crate::resources::{get_parameterized_test_data, TestParams};
        use crate::ProofType;
        let test_data = get_parameterized_test_data(TestParams::new_legacy(ProofType::Plain))
            .expect("test data should be available");
        match test_data.versioned_vk {
            VersionedVk::Legacy(bytes) => bytes,
            _ => panic!("Expected Legacy variant from test data"),
        }
    }

    /// Compute the V1 hash for raw VK bytes: SHA2-256 of the raw bytes
    /// (matches pre-versioning code at commit 113a728c).
    fn v1_vk_hash(raw_vk: &[u8; ultrahonk_no_std_v0_84::VK_SIZE]) -> H256 {
        sp_io::hashing::sha2_256(raw_vk).into()
    }

    #[test]
    fn migrates_vk_to_legacy_variant() {
        test_ext().execute_with(|| {
            let raw_vk = test_vk_bytes();
            let hash = v1_vk_hash(&raw_vk);

            // Insert one VK entry in V1 format
            v1::Vks::<Test>::insert(
                hash,
                v1::OldVkEntry {
                    vk: raw_vk,
                    ref_count: 3,
                },
            );

            // Run migration
            InnerMigrateV1ToV2::<Test>::on_runtime_upgrade();

            // V2 storage should have the migrated entry
            let new_entry =
                v2::Vks::<Test>::get(hash).expect("VK should be present after migration");
            assert_eq!(new_entry.vk, VersionedVk::Legacy(raw_vk));
            assert_eq!(new_entry.ref_count, 3);
        });
    }

    #[test]
    fn migration_preserves_vk_hash_so_vk_is_retrievable() {
        test_ext().execute_with(|| {
            let raw_vk = test_vk_bytes();
            let v1_hash = v1_vk_hash(&raw_vk);

            // Insert in V1 format keyed by the V1 hash
            v1::Vks::<Test>::insert(
                v1_hash,
                v1::OldVkEntry {
                    vk: raw_vk,
                    ref_count: 1,
                },
            );

            // Run migration
            InnerMigrateV1ToV2::<Test>::on_runtime_upgrade();

            // Compute the Legacy vk_hash (what the pallet will use after migration)
            let legacy_vk = VersionedVk::Legacy(raw_vk);
            let legacy_hash = Ultrahonk::<Test>::vk_hash(&legacy_vk);

            // The hashes must be equal: same key retrieves the migrated VK
            assert_eq!(v1_hash, legacy_hash);

            // Verify the VK is retrievable by the Legacy hash via v2 alias
            let entry =
                v2::Vks::<Test>::get(legacy_hash).expect("VK should be retrievable by Legacy hash");
            assert_eq!(entry.vk, legacy_vk);

            // Verify the pallet's own storage can also decode and find the entry
            assert!(
                pallet_verifiers::Vks::<Test, Ultrahonk<Test>>::contains_key(legacy_hash),
                "VK should be retrievable through pallet_verifiers::Vks using the same hash"
            );
        });
    }

    #[test]
    fn migrates_multiple_vks_with_correct_weight() {
        test_ext().execute_with(|| {
            let vk1 = test_vk_bytes();
            let hash1 = v1_vk_hash(&vk1);

            // Create a second VK by mutating the first
            let mut vk2 = vk1;
            vk2[0] ^= 0xff;
            let hash2 = v1_vk_hash(&vk2);

            // Insert both
            v1::Vks::<Test>::insert(
                hash1,
                v1::OldVkEntry {
                    vk: vk1,
                    ref_count: 1,
                },
            );
            v1::Vks::<Test>::insert(
                hash2,
                v1::OldVkEntry {
                    vk: vk2,
                    ref_count: 5,
                },
            );

            // Run migration
            let weight = InnerMigrateV1ToV2::<Test>::on_runtime_upgrade();

            // Both should be migrated
            assert_eq!(v2::Vks::<Test>::iter().count(), 2);

            let entry1 = v2::Vks::<Test>::get(hash1).unwrap();
            assert_eq!(entry1.vk, VersionedVk::Legacy(vk1));
            assert_eq!(entry1.ref_count, 1);

            let entry2 = v2::Vks::<Test>::get(hash2).unwrap();
            assert_eq!(entry2.vk, VersionedVk::Legacy(vk2));
            assert_eq!(entry2.ref_count, 5);

            // Weight: 2 reads + 4 writes (2 drain deletes + 2 inserts)
            assert_eq!(
                weight,
                <<Test as frame_system::Config>::DbWeight as Get<
                    frame_support::weights::RuntimeDbWeight,
                >>::get()
                .reads_writes(2, 4)
            );
        });
    }

    #[test]
    fn empty_storage_migration_is_noop() {
        test_ext().execute_with(|| {
            let weight = InnerMigrateV1ToV2::<Test>::on_runtime_upgrade();

            assert_eq!(v2::Vks::<Test>::iter().count(), 0);
            assert_eq!(
                weight,
                <<Test as frame_system::Config>::DbWeight as Get<
                    frame_support::weights::RuntimeDbWeight,
                >>::get()
                .reads_writes(0, 0)
            );
        });
    }
}
