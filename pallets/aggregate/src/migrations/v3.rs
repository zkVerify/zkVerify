//! Migrate from storage v2 to v3.

use alloc::vec::Vec;
use frame_support::{migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade};
use sp_core::Get;

/// Implements [`UncheckedOnRuntimeUpgrade`], migrating the state of this pallet from V1 to V2.
///
/// In V1 of the template, the value of the [`crate::Domains`] `StorageMap` is the old `Domain`
/// without owner tip.
///
/// We migrate every domain by adding a zero owner tip.
pub struct InnerMigrateV2ToV3<T>(core::marker::PhantomData<T>);

impl<T: crate::Config> UncheckedOnRuntimeUpgrade for InnerMigrateV2ToV3<T> {
    /// Migrate the storage from V1 to v2.
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        todo!("Migration code here")
    }
}

/// [`UncheckedOnRuntimeUpgrade`] implementation [`InnerMigrateV0ToV1`] wrapped in a
/// [`VersionedMigration`](frame_support::migrations::VersionedMigration), which ensures that:
/// - The migration only runs once when the on-chain storage version is 1
/// - The on-chain storage version is updated to `2` after the migration executes
/// - Reads/Writes from checking/settings the on-chain storage version are accounted for
pub type MigrateV2ToV3<T> = VersionedMigration<
    2, // The migration will only execute when the on-chain storage version is 1
    3, // The on-chain storage version will be set to 2 after the migration is complete
    InnerMigrateV2ToV3<T>,
    crate::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;
