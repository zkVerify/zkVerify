//! Migrate from storage v0 to v1.

mod v0;

use frame_support::{migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade};
use sp_core::Get;
use sp_std::vec::Vec;

/// Implements [`UncheckedOnRuntimeUpgrade`], migrating the state of this pallet from V0 to V1.
///
/// In V0 of the template, the value of the [`crate::Domains`] `StorageMap` is the old `Domain`
/// without delivery configurations and aggregation rules.
///
/// We migrate every domain by add a [`crate::data::AggregateSecurityRules::Untrusted`] rules
/// and a [`crate::data::DeliveryParams`] with the same domain owner, none destination and zero
/// price.
///
/// If we cannot find any delivery owner, we'll remove domain from the storage.
pub struct InnerMigrateV0ToV1<T>(core::marker::PhantomData<T>);

impl<T: crate::Config> UncheckedOnRuntimeUpgrade for InnerMigrateV0ToV1<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        // Count the _migrable_entries
        let counts: u64 = v0::Domains::<T>::iter()
            .filter_map(|(_, d)| d.owner.account().map(|_| Some(())))
            .count() as u64;
        use codec::Encode;
        Ok(counts.encode().to_vec())
    }

    /// Migrate the storage from V0 to V1.
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let old_storage = v0::Domains::<T>::drain().collect::<Vec<_>>();
        let (reads, mut writes) = (old_storage.len() as u64, old_storage.len() as u64);
        let converted = old_storage
            .into_iter()
            .filter_map(|(domain_id, domain)| {
                // Only the domain with a delivery owner can be migrated
                domain
                    .owner
                    .account()
                    .cloned()
                    .map(|owner_id| (domain_id, domain, owner_id))
            })
            .map(|(domain_id, domain, owner_id)| {
                (
                    domain_id,
                    crate::Domain::<T>(crate::data::DomainEntry {
                        id: domain.id,
                        owner: domain.owner,
                        state: domain.state,
                        next: domain.next,
                        max_aggregation_size: domain.max_aggregation_size,
                        should_publish: domain.should_publish,
                        publish_queue_size: domain.publish_queue_size,
                        ticket: domain.ticket,
                        aggregate_rules: crate::data::AggregateSecurityRules::Untrusted,
                        delivery: crate::data::DeliveryParams::new(
                            owner_id,
                            crate::data::Delivery::new(
                                hp_dispatch::Destination::None,
                                0_u32.into(),
                            ),
                        ),
                    }),
                )
            })
            .collect::<Vec<_>>();
        writes += converted.len() as u64;
        for (domain_id, migrated_domain) in converted.into_iter() {
            crate::Domains::<T>::insert(domain_id, migrated_domain);
        }
        T::DbWeight::get().reads_writes(reads, writes)
    }

    /// Verifies the storage was migrated correctly.
    #[cfg(feature = "try-runtime")]
    fn post_upgrade(encoded_numbers: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        use codec::Decode;
        let expected: u64 =
            u64::decode(&mut encoded_numbers.as_slice()).map_err(|_| "Cannot decode expected")?;
        frame_support::ensure!(
            crate::Domains::<T>::iter().count() as u64 == expected,
            "Unexpected number of domains"
        );
        Ok(())
    }
}

/// [`UncheckedOnRuntimeUpgrade`] implementation [`InnerMigrateV0ToV1`] wrapped in a
/// [`VersionedMigration`](frame_support::migrations::VersionedMigration), which ensures that:
/// - The migration only runs once when the on-chain storage version is 0
/// - The on-chain storage version is updated to `1` after the migration executes
/// - Reads/Writes from checking/settings the on-chain storage version are accounted for
pub type MigrateV0ToV1<T> = VersionedMigration<
    0, // The migration will only execute when the on-chain storage version is 0
    1, // The on-chain storage version will be set to 1 after the migration is complete
    InnerMigrateV0ToV1<T>,
    crate::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;

#[cfg(test)]
mod test {
    // use self::InnerMigrateV0ToV1;
    use super::*;
    use crate::mock::*;
    use frame_support::assert_ok;
    use frame_support::{pallet_prelude::*, weights::RuntimeDbWeight};

    fn create_old_domain(
        id: u32,
        owner: v0::User<u64>,
        state: v0::DomainState,
        ticket_owner: Option<u64>,
    ) -> v0::Domain<Test> {
        v0::Domain::<Test> {
            id,
            owner,
            state,
            next: v0::AggregationEntry {
                id: 42,
                size: 16,
                statements: BoundedVec::default(),
            },
            max_aggregation_size: 32,
            should_publish: BoundedBTreeMap::new(),
            publish_queue_size: 5,
            ticket: ticket_owner.map(|who| MockConsideration {
                who,
                count: 10,
                size: 1000,
            }),
        }
    }

    #[test]
    fn successful_migration() {
        test().execute_with(|| {
            // CLEAN THE TEST STORAGE
            v0::Domains::<Test>::drain().count();

            v0::Domains::<Test>::insert(
                23,
                create_old_domain(23, v0::User::from(123), v0::DomainState::Ready, Some(321)),
            );
            v0::Domains::<Test>::insert(
                42,
                create_old_domain(42, v0::User::from(321), v0::DomainState::Hold, Some(123)),
            );
            // This one is create by manager: could not be migrated
            v0::Domains::<Test>::insert(
                1,
                create_old_domain(1, v0::User::Manager, v0::DomainState::Removable, None),
            );
            v0::Domains::<Test>::insert(
                2,
                create_old_domain(2, v0::User::from(42), v0::DomainState::Removable, Some(33)),
            );

            #[cfg(feature = "try-runtime")]
            let expected_encoded = InnerMigrateV0ToV1::<Test>::pre_upgrade()
                .map_err(|e| format!("pre_upgrade failed: {:?}", e))
                .unwrap();

            // Perform runtime upgrade
            let weight = InnerMigrateV0ToV1::<Test>::on_runtime_upgrade();

            #[cfg(feature = "try-runtime")]
            assert_ok!(InnerMigrateV0ToV1::<Test>::post_upgrade(expected_encoded));

            // Check that `Domains` contains the expected number
            assert_eq!(crate::Domains::<Test>::iter().count(), 3);

            assert!(
                crate::Domains::<Test>::get(1).is_none(),
                "Domain 1 should not exist because created by the manager"
            );

            let domain_data = |id| {
                let crate::Domain::<Test>(crate::data::DomainEntry {
                    owner,
                    state,
                    ticket,
                    ..
                }) = crate::Domains::<Test>::take(id).unwrap();
                (owner, state, ticket.unwrap().who)
            };
            use crate::data::{DomainState, User};
            assert_eq!(domain_data(23), (User::from(123), DomainState::Ready, 321));
            assert_eq!(domain_data(42), (User::from(321), DomainState::Hold, 123));
            assert_eq!(domain_data(2), (User::from(42), DomainState::Removable, 33));

            // Check that weight are as expected
            assert_eq!(
                weight,
                <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get()
                    .reads_writes(4, 7)
            );
        })
    }
}
