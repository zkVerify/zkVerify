//! Migrate from storage v1 to v2.

mod v1;

use alloc::vec::Vec;
use frame_support::{migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade};
use sp_core::Get;

/// Implements [`UncheckedOnRuntimeUpgrade`], migrating the state of this pallet from V1 to V2.
///
/// In V1 of the template, the value of the [`crate::Domains`] `StorageMap` is the old `Domain`
/// without owner tip.
///
/// We migrate every domain by adding a zero owner tip.
pub struct InnerMigrateV1ToV2<T>(core::marker::PhantomData<T>);

impl<T: crate::Config> UncheckedOnRuntimeUpgrade for InnerMigrateV1ToV2<T> {
    /// Migrate the storage from V1 to v2.
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let old_storage = v1::Domains::<T>::drain().collect::<Vec<_>>();
        let (reads, mut writes) = (old_storage.len() as u64, old_storage.len() as u64);
        let converted = old_storage
            .into_iter()
            .map(|(domain_id, domain)| {
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
                        aggregate_rules: domain.aggregate_rules,
                        delivery: crate::data::DeliveryParams::new(
                            domain.delivery.owner,
                            crate::data::Delivery::new(
                                domain.delivery.data.destination,
                                domain.delivery.data.fee,
                                0_u32.into(), // 0 as owner tip initially
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
}

/// [`UncheckedOnRuntimeUpgrade`] implementation [`InnerMigrateV0ToV1`] wrapped in a
/// [`VersionedMigration`](frame_support::migrations::VersionedMigration), which ensures that:
/// - The migration only runs once when the on-chain storage version is 1
/// - The on-chain storage version is updated to `2` after the migration executes
/// - Reads/Writes from checking/settings the on-chain storage version are accounted for
pub type MigrateV1ToV2<T> = VersionedMigration<
    1, // The migration will only execute when the on-chain storage version is 1
    2, // The on-chain storage version will be set to 2 after the migration is complete
    InnerMigrateV1ToV2<T>,
    crate::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;

#[cfg(test)]
mod test {
    // use self::InnerMigrateV0ToV1;
    use super::*;
    use crate::mock::*;
    use frame_support::{pallet_prelude::*, weights::RuntimeDbWeight};
    use hp_dispatch::Destination;

    fn create_old_domain(
        id: u32,
        owner: v1::User<u64>,
        state: v1::DomainState,
        ticket_owner: Option<u64>,
    ) -> v1::Domain<Test> {
        v1::Domain::<Test> {
            id,
            owner,
            state,
            next: v1::AggregationEntry {
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
            aggregate_rules: v1::AggregateSecurityRules::Untrusted,
            delivery: v1::DeliveryParams {
                owner: 123_u64,
                data: v1::Delivery {
                    destination: Destination::None,
                    fee: 100,
                },
            },
        }
    }

    #[test]
    fn successful_migration() {
        test().execute_with(|| {
            // CLEAN THE TEST STORAGE
            v1::Domains::<Test>::drain().count();

            v1::Domains::<Test>::insert(
                23,
                create_old_domain(23, v1::User::from(123), v1::DomainState::Ready, Some(321)),
            );
            v1::Domains::<Test>::insert(
                42,
                create_old_domain(42, v1::User::from(321), v1::DomainState::Hold, Some(123)),
            );
            v1::Domains::<Test>::insert(
                2,
                create_old_domain(2, v1::User::from(42), v1::DomainState::Removable, Some(33)),
            );

            // Perform runtime upgrade
            let weight = InnerMigrateV1ToV2::<Test>::on_runtime_upgrade();

            // Check that `Domains` contains the expected number
            assert_eq!(crate::Domains::<Test>::iter().count(), 3);

            let domain_data = |id| {
                let crate::Domain::<Test>(crate::data::DomainEntry {
                    owner,
                    state,
                    ticket,
                    delivery,
                    ..
                }) = crate::Domains::<Test>::take(id).unwrap();
                (
                    owner,
                    state,
                    ticket.unwrap().who,
                    *delivery.fee(),
                    *delivery.owner_tip(),
                )
            };
            use crate::data::{DomainState, User};
            assert_eq!(
                domain_data(23),
                (User::from(123), DomainState::Ready, 321, 100, 0)
            );
            assert_eq!(
                domain_data(42),
                (User::from(321), DomainState::Hold, 123, 100, 0)
            );
            assert_eq!(
                domain_data(2),
                (User::from(42), DomainState::Removable, 33, 100, 0)
            );

            // Check that weight are as expected
            assert_eq!(
                weight,
                <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get()
                    .reads_writes(3, 6)
            );
        })
    }
}
