// Copyright 2024, Horizen Labs, Inc.
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

//! Migrate from storage v2 to v3.

mod v2;

use crate::ProofSecurityRules;
use alloc::vec::Vec;
use frame_support::{migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade};
use sp_core::Get;

/// Implements [`UncheckedOnRuntimeUpgrade`], migrating the state of this pallet from V2 to V3.
///
/// In V2 of the template, the value of the [`crate::Domains`] `StorageMap` is the old `Domain`
/// without an allowlist consideration ticket.
///
/// We migrate every domain by adding a `None` to allowlist consideration ticket.
pub struct InnerMigrateV2ToV3<T>(core::marker::PhantomData<T>);

impl<T: crate::Config> UncheckedOnRuntimeUpgrade for InnerMigrateV2ToV3<T> {
    /// Migrate the storage from V2 to v3.
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let old_storage = v2::Domains::<T>::drain().collect::<Vec<_>>();
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
                        ticket_domain: domain.ticket,
                        ticket_allowlist: None,
                        aggregate_rules: domain.aggregate_rules,
                        proof_rules: ProofSecurityRules::Untrusted,
                        delivery: domain.delivery,
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

/// [`UncheckedOnRuntimeUpgrade`] implementation [`InnerMigrateV2ToV3`] wrapped in a
/// [`VersionedMigration`](VersionedMigration), which ensures that:
/// - The migration only runs once when the on-chain storage version is 2
/// - The on-chain storage version is updated to `3` after the migration executes
/// - Reads/Writes from checking/settings the on-chain storage version is accounted for
pub type MigrateV2ToV3<T> = VersionedMigration<
    2, // The migration will only execute when the on-chain storage version is 3
    3, // The on-chain storage version will be set to 4 after the migration is complete
    InnerMigrateV2ToV3<T>,
    crate::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;

#[cfg(test)]
mod tests {
    use super::v2 as old_v;
    use super::*;
    use crate::data::{Delivery, DeliveryParams};
    use crate::mock::*;
    use crate::ProofSecurityRules;
    use frame_support::weights::RuntimeDbWeight;
    use frame_support::{BoundedBTreeMap, BoundedVec};
    use hp_dispatch::Destination;
    use sp_core::Get;

    fn create_old_domain(
        id: u32,
        owner: old_v::User<u64>,
        state: old_v::DomainState,
    ) -> old_v::Domain<Test> {
        old_v::Domain::<Test> {
            id,
            owner,
            state,
            next: old_v::AggregationEntry {
                id: 42,
                size: 16,
                statements: BoundedVec::default(),
            },
            max_aggregation_size: 32,
            should_publish: BoundedBTreeMap::new(),
            publish_queue_size: 5,
            ticket: None,
            aggregate_rules: old_v::AggregateSecurityRules::Untrusted,
            delivery: DeliveryParams::new(
                123,
                Delivery {
                    destination: Destination::None,
                    fee: 100,
                    owner_tip: 33,
                },
            ),
        }
    }

    #[test]
    fn successful_migration() {
        test().execute_with(|| {
            // CLEAN THE TEST STORAGE
            old_v::Domains::<Test>::drain().count();

            old_v::Domains::<Test>::insert(
                23,
                create_old_domain(23, old_v::User::from(123), old_v::DomainState::Ready),
            );
            old_v::Domains::<Test>::insert(
                42,
                create_old_domain(42, old_v::User::from(321), old_v::DomainState::Hold),
            );
            old_v::Domains::<Test>::insert(
                2,
                create_old_domain(2, old_v::User::from(42), old_v::DomainState::Removable),
            );

            // Perform runtime upgrade
            let weight = InnerMigrateV2ToV3::<Test>::on_runtime_upgrade();

            // Check that `Domains` contains the expected number
            assert_eq!(crate::Domains::<Test>::iter().count(), 3);

            let domain_data = |id| {
                let crate::Domain::<Test>(crate::data::DomainEntry {
                    owner,
                    state,
                    ticket_allowlist,
                    proof_rules,
                    ..
                }) = crate::Domains::<Test>::take(id).unwrap();
                (owner, state, ticket_allowlist, proof_rules)
            };
            use crate::data::{DomainState, User};
            assert_eq!(
                domain_data(23),
                (
                    User::from(123),
                    DomainState::Ready,
                    None,
                    ProofSecurityRules::Untrusted
                )
            );
            assert_eq!(
                domain_data(42),
                (
                    User::from(321),
                    DomainState::Hold,
                    None,
                    ProofSecurityRules::Untrusted
                )
            );
            assert_eq!(
                domain_data(2),
                (
                    User::from(42),
                    DomainState::Removable,
                    None,
                    ProofSecurityRules::Untrusted
                )
            );

            // Check that weight is as expected
            assert_eq!(
                weight,
                <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get()
                    .reads_writes(3, 6)
            );
        })
    }
}
