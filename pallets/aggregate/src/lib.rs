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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::weights::Weight;
pub use pallet::*;

mod mock;
mod should;

pub trait WeightInfo {
    fn aggregate() -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
    use super::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_support::sp_runtime::traits::Keccak256;
    use frame_support::traits::{BalanceStatus, Currency, EstimateCallFee, ReservableCurrency};
    use frame_support::{sp_runtime::testing::H256, BoundedVec};
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};

    pub type AccountOf<T> = <T as frame_system::Config>::AccountId;
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    pub trait ComputeFeeFor<B> {
        fn compute_fee(estimated: B) -> Option<B>;
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The (max) size of aggregations.
        #[pallet::constant]
        type AggregationSize: Get<u32>;
        /// The upperbound on the number of aggregations that can stay in _to be published_ state
        /// for a single domain to wait a publish_aggregation call.
        #[pallet::constant]
        type MaxPendingPublishQueueSize: Get<u32>;
        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;
        /// What should we use to estimate publish aggregation cost (pallet-transaction-payment implement it)
        type EstimateCallFee: EstimateCallFee<Call<Self>, BalanceOf<Self>>;
        /// How to compute the fee for publishing an aggregation.
        type ComputeFeeFor: ComputeFeeFor<BalanceOf<Self>>;
        /// The weight definition for this pallet
        type WeightInfo: WeightInfo;
    }

    impl<T: Config> hp_poe::OnProofVerified<<T as frame_system::Config>::AccountId> for Pallet<T> {
        fn on_proof_verified(
            account: Option<<T as frame_system::Config>::AccountId>,
            domain_id: Option<u32>,
            statement: H256,
        ) {
            log::trace!("Proof: [{account:?}]-{domain_id:?} {statement:?}");
            // Preconditions: You should provide
            // - An account for reserve found.
            // - A valid domain id
            let Some(account) = account else {
                log::warn!("No account, skip");
                Self::deposit_event(Event::<T>::CannotAggregate {
                    statement,
                    cause: CannotAggregateCause::NoAccount,
                });

                return;
            };
            let Some(domain_id) = domain_id else {
                log::debug!("No domain, skip");
                Self::deposit_event(Event::<T>::CannotAggregate {
                    statement,
                    cause: CannotAggregateCause::NoDomain,
                });

                return;
            };
            Domains::<T>::mutate(domain_id, |domain| {
                // Check if the domain is registered
                let Some(domain) = domain else {
                    log::debug!("The requested domain is not registered, skip");
                    Self::deposit_event(Event::<T>::CannotAggregate {
                        statement,
                        cause: CannotAggregateCause::DomainNotRegistered { domain_id },
                    });

                    return;
                };
                // Check if we can add a new statement
                if !domain.can_add_statement() {
                    log::warn!("Storage complete, skip");
                    Self::deposit_event(Event::<T>::CannotAggregate {
                        statement,
                        cause: CannotAggregateCause::DomainStorageFull { domain_id },
                    });

                    return;
                }

                // Reserve balance for publication: if not raise a fail event
                let Ok(reserve) = reserve_currency_for_publication::<T>(domain, &account)
                    .inspect_err(|err| {
                        Self::deposit_event(Event::<T>::CannotAggregate {
                            statement,
                            cause: CannotAggregateCause::InsufficientFound,
                        });

                        log::debug!("Failed to reserve balance {err:?}");
                    })
                else {
                    return;
                };

                // We can add the statement and check if we should also move the aggregation in the should publish set
                Self::deposit_event(Event::<T>::ProofVerified {
                    statement,
                    domain_id,
                    aggregation_id: domain.next.id,
                });
                let to_publish = append_statement::<T>(domain, account.clone(), reserve, statement);
                if let Some(aggregation) = to_publish {
                    available_aggregation::<T>(domain, aggregation);
                }
            });
        }
    }

    fn append_statement<T: Config>(
        domain: &mut Domain<T>,
        account: T::AccountId,
        reserve: BalanceOf<T>,
        statement: H256,
    ) -> Option<Aggregation<T>> {
        let aggregation = &mut domain.next;
        aggregation
            .statements
            .force_push(StatementEntry::new(account.clone(), reserve, statement));
        if aggregation.size as usize <= aggregation.statements.len() {
            Some(sp_std::mem::replace(
                aggregation,
                aggregation.create_next(aggregation.size),
            ))
        } else {
            None
        }
    }

    fn available_aggregation<T: Config>(domain: &mut Domain<T>, aggregation: Aggregation<T>) {
        Pallet::<T>::deposit_event(Event::<T>::ReadyToAggregate {
            domain_id: domain.id,
            aggregation_id: aggregation.id,
        });
        domain
            .should_publish
            .try_insert(aggregation.id, aggregation)
            .expect("Should not publish aggregation if it's not possible: qed");
        // If is full send an alert event
        if domain.should_publish.len() >= domain.publish_queue_size as usize {
            Pallet::<T>::deposit_event(Event::<T>::DomainFull {
                domain_id: domain.id,
            });
        }
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// This domain id is unknown.
        UnknownDomainId,
        /// This aggregation cannot be published or it's already published.
        InvalidAggregationId,
    }

    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum CannotAggregateCause {
        NoAccount,
        NoDomain,
        DomainNotRegistered { domain_id: u32 },
        DomainStorageFull { domain_id: u32 },
        InsufficientFound,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProofVerified {
            statement: H256,
            domain_id: u32,
            aggregation_id: u64,
        },
        ReadyToAggregate {
            domain_id: u32,
            aggregation_id: u64,
        },
        NewAggregationReceipt {
            domain_id: u32,
            aggregation_id: u64,
            receipt: H256,
        },
        CannotAggregate {
            statement: H256,
            cause: CannotAggregateCause,
        },
        DomainFull {
            domain_id: u32,
        },
    }

    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub(crate) struct StatementEntry<A, B> {
        pub(crate) account: A,
        pub(crate) reserve: B,
        pub(crate) statement: H256,
    }

    impl<A, B> StatementEntry<A, B> {
        pub fn new(account: A, reserve: B, statement: H256) -> Self {
            Self {
                account,
                reserve,
                statement,
            }
        }
    }

    /// A complete Verification Key or its hash.
    #[derive(Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(S))]
    pub struct AggregationEntry<A, B, S: Get<u32>> {
        pub(crate) id: u64,
        pub(crate) size: u32,
        pub(crate) statements: BoundedVec<StatementEntry<A, B>, S>,
    }

    impl<A: sp_std::fmt::Debug, B: sp_std::fmt::Debug, S: Get<u32>> sp_std::fmt::Debug
        for AggregationEntry<A, B, S>
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("AggregationEntry")
                .field("id", &self.id)
                .field("size", &self.size)
                .field("statements", &self.statements)
                .finish()
        }
    }

    impl<A, B, S: Get<u32>> AggregationEntry<A, B, S> {
        fn new(id: u64, size: u32, statements: BoundedVec<StatementEntry<A, B>, S>) -> Self {
            assert!(size <= S::get(), "Aggregation size is out of bound");
            Self {
                id,
                size,
                statements,
            }
        }

        pub(crate) fn create(id: u64, size: u32) -> Self {
            Self::new(id, size, BoundedVec::new())
        }

        fn create_next(&self, size: u32) -> Self {
            Self::create(self.id + 1, size)
        }

        fn space_left(&self) -> usize {
            (self.size as usize).saturating_sub(self.statements.len())
        }

        fn compute(&self) -> H256 {
            binary_merkle_tree::merkle_root::<Keccak256, _>(
                self.statements.iter().map(|s| s.statement.as_ref()),
            )
        }
    }

    impl<A, B, S: Get<u32>> Default for AggregationEntry<A, B, S> {
        fn default() -> Self {
            Self::create(1, S::get())
        }
    }

    pub type Aggregation<T> =
        AggregationEntry<AccountOf<T>, BalanceOf<T>, <T as Config>::AggregationSize>;

    /// A complete Verification Key or its hash.
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(S, M))]
    pub(crate) struct DomainEntry<A, B, S: Get<u32>, M: Get<u32>> {
        pub id: u32,
        pub next: AggregationEntry<A, B, S>,
        pub max_aggregation_size: u32,
        pub should_publish: BoundedBTreeMap<u64, AggregationEntry<A, B, S>, M>,
        pub publish_queue_size: u32,
    }

    impl<A, B, S: Get<u32>, M: Get<u32>> DomainEntry<A, B, S, M> {
        /// Create a new domain.
        ///
        /// PANIC: If max_aggregation_size is greater than Config::AggregationSize or
        /// publish_queue_size is greater than Config::MaxPendingPublishQueueSize.
        pub fn create(
            id: u32,
            next_attestation_id: u64,
            max_attestation_size: u32,
            publish_queue_size: u32,
        ) -> Self {
            assert!(
                max_attestation_size <= S::get(),
                "Max aggregation size must be less or equal than Config::AggregationSize"
            );
            assert!(
                publish_queue_size <= M::get(),
                "Publish queue size must be less or equal than Config::MaxPendingPublishQueueSize"
            );
            Self {
                id,
                next: AggregationEntry::create(next_attestation_id, max_attestation_size),
                max_aggregation_size: max_attestation_size,
                should_publish: Default::default(),
                publish_queue_size,
            }
        }

        /// Return true iff it's possible to add a new statement.
        pub fn can_add_statement(&self) -> bool {
            (self.publish_queue_size as usize).saturating_sub(self.should_publish.len()) > 0
                || self.next.space_left() > 1
        }
    }

    /// Compute and reserve the currency for further publication
    fn reserve_currency_for_publication<T: Config>(
        domain: &mut Domain<T>,
        account: &AccountOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let estimated = estimate_publish_attestation_fee::<T>();
        let reserve = (estimated
            + <T as Config>::ComputeFeeFor::compute_fee(estimated).unwrap_or_default())
            / domain.next.size.into();
        T::Currency::reserve(account, reserve).map(|_| reserve)
    }

    /// Shortcut to get the Domain type from config.
    pub(crate) type Domain<T> = DomainEntry<
        AccountOf<T>,
        BalanceOf<T>,
        <T as Config>::AggregationSize,
        <T as Config>::MaxPendingPublishQueueSize,
    >;

    /// Domains storage
    #[pallet::storage]
    pub(crate) type Domains<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = u32, Value = Domain<T>>;

    #[pallet::storage]
    #[pallet::getter(fn published)]
    #[pallet::unbounded]
    /// Vector of published aggregations. This will stay just in one block because we remove
    /// this vector at the start of every block (on_initialize hook).
    pub type Published<T: Config> = StorageValue<_, Vec<Aggregation<T>>, ValueQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Published::<T>::take();
            T::DbWeight::get().writes(1_u64)
        }
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Publish the aggregation.
        #[pallet::call_index(0)]
        pub fn aggregate(
            origin: OriginFor<T>,
            domain_id: u32,
            id: u64,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let root = Domains::<T>::try_mutate(domain_id, |domain| {
                let domain = domain.as_mut().ok_or(Error::<T>::UnknownDomainId)?;
                let aggregation = domain
                    .should_publish
                    .remove(&id)
                    .ok_or(Error::<T>::InvalidAggregationId)?;

                let root = aggregation.compute();
                Published::<T>::mutate(|published: &mut _| published.push(aggregation));

                if let Some(published) = Published::<T>::get().last() {
                    for s in published.statements.iter() {
                        let account = &s.account;
                        let missed = T::Currency::repatriate_reserved(
                            account,
                            &origin,
                            s.reserve,
                            BalanceStatus::Free,
                        )
                        .expect("Call user should exists. qed");
                        if missed > 0_u32.into() {
                            log::warn!(
                                "Cannot refund all founds from {account} to {origin}: missed {missed:?}"
                            )
                        }
                    }
                }

                Result::<_, Error<T>>::Ok(Some(root))
            })?;
            if let Some(root) = root {
                Self::deposit_event(Event::NewAggregationReceipt {
                    domain_id,
                    aggregation_id: id,
                    receipt: root,
                });
            }
            Ok(().into())
        }
    }

    fn estimate_publish_attestation_fee<T: Config>() -> BalanceOf<T> {
        T::EstimateCallFee::estimate_call_fee(
            &Call::aggregate {
                domain_id: 0,
                id: 0,
            },
            Default::default(),
        )
    }
}
