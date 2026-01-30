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

//! Migrate from storage v3 to v4.

mod v3;

use alloc::vec::Vec;
use core::fmt::Debug;
use frame_support::{migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade};
use sp_core::Get;

/// Implements [`UncheckedOnRuntimeUpgrade`], migrating the state of this pallet from V3 to V4.
///
/// In V3 of the template, the value of the [`crate::Domains`] `StorageMap` is the old `Domain`
/// where the [`Destination`] enum had an Hyperbridge variant that isn't present in V4.
///
/// We migrate the domain by just Remove the destination: is not the best way to handle it becuase
/// we didn't take care of the removing state machine, but it's a way to migrate the storage and take
/// it coherent: we need to take care to remove the old domains before do a migration and leverage
/// on that just root can add Hyperbridge domains.
pub struct InnerMigrateV3ToV4<T>(core::marker::PhantomData<T>);

impl<B: Debug + PartialEq> From<v3::Delivery<B>> for crate::data::Delivery<B> {
    fn from(v3::Delivery::<B> { fee, owner_tip, .. }: v3::Delivery<B>) -> Self {
        Self {
            destination: hp_dispatch::Destination::None,
            fee,
            owner_tip,
        }
    }
}

impl<A, B: Debug + PartialEq> From<v3::DeliveryParams<A, B>> for crate::data::DeliveryParams<A, B> {
    fn from(v3::DeliveryParams::<A, B> { owner, data }: v3::DeliveryParams<A, B>) -> Self {
        Self::new(owner, data.into())
    }
}

impl<T: crate::Config> From<v3::Domain<T>> for crate::Domain<T> {
    fn from(
        v3::Domain::<T> {
            id,
            owner,
            state,
            next,
            max_aggregation_size,
            should_publish,
            publish_queue_size,
            ticket_domain,
            ticket_allowlist,
            aggregate_rules,
            proof_rules,
            delivery,
        }: v3::Domain<T>,
    ) -> Self {
        crate::Domain::<T>(crate::data::DomainEntry {
            id,
            owner,
            state,
            next,
            max_aggregation_size,
            should_publish,
            publish_queue_size,
            ticket_domain,
            ticket_allowlist,
            aggregate_rules,
            proof_rules,
            delivery: delivery.into(),
        })
    }
}

impl<T: crate::Config> UncheckedOnRuntimeUpgrade for InnerMigrateV3ToV4<T> {
    /// Migrate the storage from V3 to v4.
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let old_storage = v3::Domains::<T>::drain().collect::<Vec<_>>();
        let (reads, mut writes) = (old_storage.len() as u64, old_storage.len() as u64);
        let converted = old_storage
            .into_iter()
            .map(|(domain_id, domain)| (domain_id, domain.into()))
            .collect::<Vec<(_, crate::Domain<T>)>>();
        writes += converted.len() as u64;
        for (domain_id, migrated_domain) in converted.into_iter() {
            crate::Domains::<T>::insert(domain_id, migrated_domain);
        }
        T::DbWeight::get().reads_writes(reads, writes)
    }
}

/// [`UncheckedOnRuntimeUpgrade`] implementation [`InnerMigrateV3ToV4`] wrapped in a
/// [`VersionedMigration`](VersionedMigration), which ensures that:
/// - The migration only runs once when the on-chain storage version is 3
/// - The on-chain storage version is updated to `4` after the migration executes
/// - Reads/Writes from checking/settings the on-chain storage version is accounted for
pub type MigrateV3ToV4<T> = VersionedMigration<
    3, // The migration will only execute when the on-chain storage version is 3
    4, // The on-chain storage version will be set to 4 after the migration is complete
    InnerMigrateV3ToV4<T>,
    crate::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;

#[cfg(test)]
mod tests {
    use super::v3 as old_v;
    use super::v3::{
        BoundedStateMachine, Delivery, DeliveryParams, Destination, HyperbridgeDispatchParameters,
    };
    use super::*;
    use crate::mock::*;
    use crate::ProofSecurityRules;
    use frame_support::weights::RuntimeDbWeight;
    use frame_support::{BoundedBTreeMap, BoundedVec};
    use sp_core::{Get, H160};

    fn create_old_domain(
        id: u32,
        owner: old_v::User<u64>,
        state: old_v::DomainState,
        destination: Destination,
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
            aggregate_rules: old_v::AggregateSecurityRules::Untrusted,
            ticket_domain: None,
            ticket_allowlist: None,
            proof_rules: ProofSecurityRules::Untrusted,
            delivery: DeliveryParams {
                owner: 123,
                data: Delivery {
                    destination,
                    fee: 100,
                    owner_tip: 33,
                },
            },
        }
    }

    #[test]
    fn successful_migration() {
        test().execute_with(|| {
            // CLEAN THE TEST STORAGE
            old_v::Domains::<Test>::drain().count();

            old_v::Domains::<Test>::insert(
                23,
                create_old_domain(
                    23,
                    old_v::User::from(123),
                    old_v::DomainState::Ready,
                    Destination::Hyperbridge(HyperbridgeDispatchParameters {
                        destination_chain: BoundedStateMachine::Evm(1),
                        destination_module: H160::from_low_u64_be(1234567890),
                        timeout: 1000,
                    }),
                ),
            );
            old_v::Domains::<Test>::insert(
                42,
                create_old_domain(
                    42,
                    old_v::User::from(321),
                    old_v::DomainState::Hold,
                    Destination::None,
                ),
            );
            old_v::Domains::<Test>::insert(
                2,
                create_old_domain(
                    2,
                    old_v::User::from(42),
                    old_v::DomainState::Removable,
                    Destination::Hyperbridge(HyperbridgeDispatchParameters {
                        destination_chain: BoundedStateMachine::Polkadot(42),
                        destination_module: H160::from_low_u64_be(9876543210),
                        timeout: 2000,
                    }),
                ),
            );

            // Perform runtime upgrade
            let weight = InnerMigrateV3ToV4::<Test>::on_runtime_upgrade();

            // Check that `Domains` contains the expected number
            assert_eq!(crate::Domains::<Test>::iter().count(), 3);

            for (_, d) in crate::Domains::<Test>::iter() {
                assert_eq!(&hp_dispatch::Destination::None, d.delivery.destination());
                assert_eq!((d.delivery.fee(), d.delivery.owner_tip()), (&100, &33));
            }
            // Sanity check
            let domain_data = |id| {
                let crate::Domain::<Test>(crate::data::DomainEntry { owner, state, .. }) =
                    crate::Domains::<Test>::take(id).unwrap();
                (owner, state)
            };
            use crate::data::{DomainState, User};
            assert_eq!(domain_data(23), (User::from(123), DomainState::Ready,));
            assert_eq!(domain_data(42), (User::from(321), DomainState::Hold,));
            assert_eq!(domain_data(2), (User::from(42), DomainState::Removable,));

            // Check that weight is as expected
            assert_eq!(
                weight,
                <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get()
                    .reads_writes(3, 6)
            );
        })
    }
}
