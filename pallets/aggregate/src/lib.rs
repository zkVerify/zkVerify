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
#![deny(missing_docs)]

//! This pallet provides a mechanism for tracking and aggregating statements (i.e. proof
//! verification submissions) from users. It is possible to define different aggregation
//! sizes and thresholds for different domains.
//!
//! Every proof should indicate in which domain should be aggregated. The publish extrinsic
//! `aggregate` a permission-less call and there is a tip for the user calling it:
//! this tip (should) cover all costs about executing aggregate and a configurable optional extra.
//!
//! Register a new domain with `register_domain` needs to hold some balance to cover the cost of storage space
//! used by all proofs hash that living in this domain while waiting for the `aggregate` call. All hold
//! balance will be freed after the `unregister_domain` call (if any): the `unregister_domain` can be done
//! only after call `hold_domain` extrinsic and there are no pending aggregations. When you put a domain in `Hold`
//! state it cannot even receive no more statements and it's just possible to aggregate all pending aggregations.
//! The domain state become `Removable` when there are no more pending aggregation and only now is to possible
//! call `unregister_domain` and free the held balance.
//!

pub use pallet::*;
pub use weight::WeightInfo;

mod benchmarking;
mod mock;
mod should;

mod data;
mod weight;

// Export the benchmarking utils.
#[cfg(feature = "runtime-benchmarks")]
pub use benchmarking::utils::*;

#[frame_support::pallet]
pub mod pallet {
    use core::ops::Deref;

    pub use crate::data::AggregationSize;
    use crate::data::{DomainState, StatementEntry, User};

    use super::WeightInfo;
    use frame_support::{
        dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::{
            fungible::{Inspect, InspectHold, MutateHold},
            tokens::{Fortitude, Precision, Restriction},
            Consideration, Defensive, DefensiveSaturating, EstimateCallFee, Footprint,
            VariantCount,
        },
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use hp_on_proof_verified::OnProofVerified;
    use sp_core::H256;
    use sp_runtime::traits::{BadOrigin, Keccak256};
    use sp_std::vec::Vec;

    /// Given a `Configuration` return the Account type.
    pub type AccountOf<T> = <T as frame_system::Config>::AccountId;
    /// Given a `Configuration` return the Balance type.
    pub type BalanceOf<T> =
        <<T as Config>::Hold as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
    /// Return the call (extrinsic) type for that pallet.
    pub type CallOf<T> = Call<T>;
    /// Given a `Configuration` return the Consideration type.
    pub(crate) type TicketOf<T> = <T as Config>::Consideration;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// This trait define how the pallet should compute the tip for the publisher.
    /// This tip will be added to the estimation of the total cost of the transaction.
    pub trait ComputePublisherTip<B> {
        /// Given an estimated cost of a transaction, return an optional tip for the publisher.
        fn compute_tip(estimated: B) -> Option<B>;
    }

    impl<B> ComputePublisherTip<B> for () {
        fn compute_tip(estimated: B) -> Option<B> {
            Some(estimated)
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The overarching hold reason.
        type RuntimeHoldReason: From<HoldReason>
            + Parameter
            + Member
            + MaxEncodedLen
            + Copy
            + VariantCount;
        /// The (max) size of aggregations.
        #[pallet::constant]
        type AggregationSize: Get<AggregationSize>;
        /// The upperbound on the number of aggregations that can stay in _to be published_ state
        /// for a single domain to wait a publish_aggregation call.
        #[pallet::constant]
        type MaxPendingPublishQueueSize: Get<u32>;
        /// An origin that can request a domain be registered on-chain without a deposit or fee, or
        /// manage existing not owned domains.
        type ManagerOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// The Hold trait.
        type Hold: MutateHold<Self::AccountId>
            + InspectHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;
        /// A means of providing some cost while data is stored on-chain.
        type Consideration: Consideration<Self::AccountId, Footprint>;
        /// What should we use to estimate aggregate cost (pallet-transaction-payment implement it)
        type EstimateCallFee: EstimateCallFee<Call<Self>, BalanceOf<Self>>;
        /// How to compute the fee for publishing an aggregation.
        type ComputePublisherTip: ComputePublisherTip<BalanceOf<Self>>;
        /// The weight definition for this pallet
        type WeightInfo: WeightInfo;
        /// The (max) size of aggregations used in benchmarks. NEEDS to be equal to AggregationSize::get()
        #[cfg(feature = "runtime-benchmarks")]
        const AGGREGATION_SIZE: u32;
        /// The weight definition for this pallet
        #[cfg(feature = "runtime-benchmarks")]
        type Currency: frame_support::traits::fungible::Mutate<AccountOf<Self>>;
    }

    impl<T: Config> OnProofVerified<<T as frame_system::Config>::AccountId> for Pallet<T> {
        fn on_proof_verified(
            account: Option<<T as frame_system::Config>::AccountId>,
            domain_id: Option<u32>,
            statement: H256,
        ) {
            log::trace!("Proof: [{account:?}]-{domain_id:?} {statement:?}");
            // Preconditions: You should provide
            // - An account for reserve funds
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
                log::trace!("No domain, skip");
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
                // Check domain state
                if DomainState::Ready != domain.state {
                    log::debug!("The requested domain cannot accept any other proofs, skip");
                    Self::deposit_event(Event::<T>::CannotAggregate {
                        statement,
                        cause: CannotAggregateCause::InvalidDomainState {
                            domain_id,
                            state: domain.state,
                        },
                    });

                    return;
                }
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
                let Ok(reserve) = domain
                    .reserve_currency_for_publication(&account)
                    .inspect_err(|err| {
                        Self::deposit_event(Event::<T>::CannotAggregate {
                            statement,
                            cause: CannotAggregateCause::InsufficientFunds,
                        });

                        log::debug!("Failed to reserve balance {err:?}");
                    })
                else {
                    return;
                };

                // We can add the statement and check if we should also move the aggregation in the should publish set
                Self::deposit_event(Event::<T>::NewProof {
                    statement,
                    domain_id,
                    aggregation_id: domain.next.id,
                });
                let to_publish = domain.append_statement(account.clone(), reserve, statement);
                if let Some(aggregation) = to_publish {
                    domain.available_aggregation(aggregation);
                }
                domain.handle_hold_state();
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
        /// The domain params are invalid.
        InvalidDomainParams,
        /// Try to remove or hold a domain in a invalid state.
        InvalidDomainState,
    }

    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    /// The cause of a missed aggregation.
    pub enum CannotAggregateCause {
        /// No account
        NoAccount,
        /// The requested domain doesn't exist.
        DomainNotRegistered {
            /// The domain identifier.
            domain_id: u32,
        },
        /// The domain's should publish queue is full.
        DomainStorageFull {
            /// The domain identifier.
            domain_id: u32,
        },
        /// The user doesn't have enough funds to hold balance for publication.
        InsufficientFunds,
        /// The domain's state is not valid.
        InvalidDomainState {
            /// The domain identifier.
            domain_id: u32,
            /// The domain state.
            state: DomainState,
        },
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    /// The emitted events.
    pub enum Event<T: Config> {
        /// A new domain has been registered.
        NewDomain {
            /// The domain identifier.
            id: u32,
        },
        /// Domain state changed.
        DomainStateChanged {
            /// The domain identifier.
            id: u32,
            /// The new state of the domain.
            state: DomainState,
        },
        /// A new proof has been received.
        NewProof {
            /// The statement hash that describe the proof.
            statement: H256,
            /// The domain identifier.
            domain_id: u32,
            /// The identifier of the aggregation .
            aggregation_id: u64,
        },
        /// The aggregation is complete.
        AggregationComplete {
            /// The domain identifier.
            domain_id: u32,
            /// The identifier of the aggregation .
            aggregation_id: u64,
        },
        /// A new aggregation receipt has been emitted.
        NewAggregationReceipt {
            /// The domain identifier.
            domain_id: u32,
            /// The identifier of the aggregation .
            aggregation_id: u64,
            /// The aggregation receipt hash.
            receipt: H256,
        },
        /// Some error occurred in [`OnProofVerified::on_proof_verified`] execution.
        CannotAggregate {
            /// The statement hash that describe the proof.
            statement: H256,
            /// The cause of the error.
            cause: CannotAggregateCause,
        },
        /// A domain should published queue is full: you cannot add any other proof to this domain till
        /// at least on proof is aggregated on this domain.
        DomainFull {
            /// The domain identifier.
            domain_id: u32,
        },
    }

    /// Shortcut to get the Aggregation type from config.
    pub type Aggregation<T> =
        crate::data::AggregationEntry<AccountOf<T>, BalanceOf<T>, <T as Config>::AggregationSize>;

    /// A domain with the account, balance, aggregation size, and max number of pending
    /// publications, and ticket type, as configured in T.
    type DomainType<T> = crate::data::DomainEntry<
        AccountOf<T>,
        BalanceOf<T>,
        <T as Config>::AggregationSize,
        <T as Config>::MaxPendingPublishQueueSize,
        TicketOf<T>,
    >;

    /// Shortcut to get the Domain type from config.
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub(crate) struct Domain<T: Config>(DomainType<T>);

    impl<T: Config> Domain<T> {
        /// Create a new domain
        ///
        /// - `id`: id of the new domain.
        /// - `owner`: the account that owns (i.e. can hold/unregister) the domain.
        /// - `next_aggregation_id`: the id of the first aggregation of the domain
        /// - `max_aggregation_size`: the maximum number of statements per aggregation for this domain.
        /// - `publish_queue_size`: the maximum number of unpublished aggregations for this domain.
        /// - `ticket`: a tracker for the deposit associated with this domain.
        pub fn try_create(
            id: u32,
            owner: User<AccountOf<T>>,
            next_aggregation_id: u64,
            max_aggregation_size: AggregationSize,
            publish_queue_size: u32,
            ticket: Option<TicketOf<T>>,
        ) -> Result<Self, Error<T>> {
            if max_aggregation_size == 0
                || publish_queue_size == 0
                || max_aggregation_size > T::AggregationSize::get()
                || publish_queue_size > T::MaxPendingPublishQueueSize::get()
            {
                Err(Error::<T>::InvalidDomainParams)
            } else {
                Ok(Self(crate::data::DomainEntry::create(
                    id,
                    owner,
                    next_aggregation_id,
                    max_aggregation_size,
                    publish_queue_size,
                    ticket,
                )))
            }
        }

        /// Compute and reserve the currency for further publication
        fn reserve_currency_for_publication(
            &self,
            account: &AccountOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let estimated = estimate_publish_aggregation_fee::<T>(self.max_aggregation_size);
            let hold = (estimated.defensive_saturating_add(
                <T as Config>::ComputePublisherTip::compute_tip(estimated).unwrap_or_default(),
            )) / self.next.size.into();
            T::Hold::hold(&HoldReason::Aggregation.into(), account, hold).map(|_| hold)
        }

        /// Return the aggregation `id`, removing it from the queue of aggregations to be
        /// published.
        fn take_aggregation(&mut self, id: u64) -> Option<Aggregation<T>> {
            if self.next.id == id {
                self.pop_next_aggregation()
            } else {
                self.should_publish.remove(&id)
            }
        }

        /// Return the size in bytes for this domain that should be reserved in the storage.
        ///
        /// - `max_aggregation_size`: The maximum size of the aggregations for this domain.
        /// - `publish_queue_size`: the publish queue size for this domain.
        pub fn compute_encoded_size(
            max_aggregation_size: AggregationSize,
            publish_queue_size: u32,
        ) -> usize {
            DomainType::<T>::compute_encoded_size(max_aggregation_size, publish_queue_size)
        }

        /// Return the next non-empty aggregation to be published, or none if the aggregation is empty.
        /// If successful, a new aggregation is created as next to be published.
        fn pop_next_aggregation(&mut self) -> Option<Aggregation<T>> {
            if self.next.statements.is_empty() {
                None
            } else {
                let aggregation = &mut self.next;
                // Create a new aggregation as next, and return the old one
                Some(sp_std::mem::replace(
                    aggregation,
                    aggregation.create_next(aggregation.size),
                ))
            }
        }

        /// Return true iff the next aggregation has reached its limit in terms of statements.
        fn is_next_aggregation_complete(&self) -> bool {
            self.next.size as usize <= self.next.statements.len()
        }

        /// Append a new statement to the next aggregation to be published.
        fn append_statement(
            &mut self,
            account: T::AccountId,
            reserve: BalanceOf<T>,
            statement: H256,
        ) -> Option<Aggregation<T>> {
            self.next
                .statements
                .try_push(StatementEntry::new(account.clone(), reserve, statement))
                .expect("Should not append statement if domain is full; qed");
            if self.is_next_aggregation_complete() {
                self.pop_next_aggregation()
            } else {
                None
            }
        }

        /// Handle the availability of a new aggregation for this domain.
        fn available_aggregation(&mut self, aggregation: Aggregation<T>) {
            Pallet::<T>::deposit_event(Event::<T>::AggregationComplete {
                domain_id: self.id,
                aggregation_id: aggregation.id,
            });
            self.should_publish
                .try_insert(aggregation.id, aggregation)
                .expect("Should not publish aggregation if it's not possible; qed");
            // If it is full send an alert event
            if self.should_publish.len() >= self.publish_queue_size as usize {
                Pallet::<T>::deposit_event(Event::<T>::DomainFull { domain_id: self.id });
            }
        }

        /// Implement the hold state machine and emits the state if change.
        fn handle_hold_state(&mut self) {
            if self.state == DomainState::Ready {
                return;
            }
            let old_state = self.state;
            self.update_hold_state();
            if old_state != self.state {
                self.emit_state_changed_event();
            }
        }

        /// Emit a DomainStateChanged event with the id and state of this domain.
        fn emit_state_changed_event(&self) {
            Pallet::<T>::deposit_event(Event::<T>::DomainStateChanged {
                id: self.id,
                state: self.state,
            });
        }
    }

    impl<T: Config> Deref for Domain<T> {
        type Target = DomainType<T>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Config> sp_std::ops::DerefMut for Domain<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    /// A reason for this pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// The funds are held as storage deposit for a aggregation pay.
        Aggregation,
        /// The funds are held as storage deposit for a domain registration.
        Domain,
    }

    /// Domains storage
    #[pallet::storage]
    #[pallet::getter(fn next_domain_id)]
    pub(crate) type NextDomainId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Domains storage
    #[pallet::storage]
    pub(crate) type Domains<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = u32, Value = Domain<T>>;

    #[pallet::storage]
    #[pallet::getter(fn published)]
    #[pallet::unbounded]
    /// Vector of published aggregations. This will stay just in one block because we remove
    /// this vector at the start of every block (on_initialize hook).
    /// It is implicitly limited by the number of aggregate extrinsics that can fit in block.
    pub type Published<T: Config> = StorageValue<_, Vec<(u32, Aggregation<T>)>, ValueQuery>;

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
    /// Cannot generate the proof of the aggregated statement.
    pub enum PathRequestError {
        /// The statement is not found in the aggregation.
        NotFound(u32, u64, sp_core::H256),
        /// The receipt is not published for the given domain and aggregation.
        ReceiptNotPublished(u32, u64),
    }

    impl<T: Config> Pallet<T> {
        /// Compute the statement Merkle path giving a proof of the aggregated statement.
        /// - domain_id: The domain identifier.
        /// - aggregation_id: The identifier of the aggregation.
        /// - statement: The statement hash that describe the proof for which we would provide a proof.
        pub fn get_statement_path(
            domain_id: u32,
            aggregation_id: u64,
            statement: H256,
        ) -> Result<binary_merkle_tree::MerkleProof<H256, H256>, PathRequestError> {
            let published = Self::published();
            let (_, aggregation) = published
                .iter()
                .find(|&(id, a)| id == &domain_id && a.id == aggregation_id)
                .ok_or(PathRequestError::ReceiptNotPublished(
                    domain_id,
                    aggregation_id,
                ))?;
            let index = aggregation
                .statements
                .iter()
                .position(|s| s.statement == statement)
                .ok_or(PathRequestError::NotFound(
                    domain_id,
                    aggregation_id,
                    statement,
                ))?;
            let leaves = aggregation.statements.iter().map(|s| s.statement);

            // Evaluate the Merkle proof and return a MerkleProof structure to the caller
            Ok(binary_merkle_tree::merkle_proof::<Keccak256, _, _>(
                leaves, index,
            ))
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Published::<T>::take();
            T::DbWeight::get().writes(1_u64)
        }
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Publish the aggregation. This call is used to publish a new aggregation that is in
        /// the domain to be published queue or is still not completed. Can be called *just by signed account* and
        /// if conditions are met, move the funds held for this publication to the caller account. If the aggregation
        /// id is not valid (in _to be published_ queue or in filling stage), the call will fail but the weight cost
        /// will be still the one needed to do the check.
        ///
        /// If conditions are met a `Event::NewAggregationReceipt` is emitted.
        ///
        /// Arguments:
        /// - domain_id: The domain identifier.
        /// - id: The identifier of the aggregation.
        #[pallet::weight(T::WeightInfo::aggregate(T::AggregationSize::get()))]
        #[pallet::call_index(0)]
        pub fn aggregate(
            origin: OriginFor<T>,
            domain_id: u32,
            aggregation_id: u64,
        ) -> DispatchResultWithPostInfo {
            use frame_support::traits::DefensiveSaturating;
            let origin = ensure_signed(origin)?;
            let (root, size) = Domains::<T>::try_mutate(domain_id, |domain| {
                let domain = domain.as_mut().ok_or_else(|| {
                    dispatch_post_error(
                        T::WeightInfo::aggregate_on_invalid_domain(),
                        Error::<T>::UnknownDomainId,
                    )
                })?;
                let aggregation = domain.take_aggregation(aggregation_id).ok_or_else(|| {
                    dispatch_post_error(
                        T::WeightInfo::aggregate_on_invalid_id(),
                        Error::<T>::InvalidAggregationId,
                    )
                })?;

                let root = aggregation.compute();
                let size = aggregation.statements.len() as u32;
                Published::<T>::mutate(|published: &mut _| {
                    published.push((domain_id, aggregation))
                });

                if let Some((_, published)) = Published::<T>::get().last() {
                    for s in published.statements.iter() {
                        let account = &s.account;
                        let remain = T::Hold::transfer_on_hold(
                            &HoldReason::Aggregation.into(),
                            account,
                            &origin,
                            s.reserve,
                            Precision::BestEffort,
                            Restriction::Free,
                            Fortitude::Polite,
                        )
                        .expect("Call user should exists. qed")
                        .defensive_saturating_sub(s.reserve);
                        if remain > 0_u32.into() {
                            log::warn!(
                                "Cannot refund all funds from {account:?} to {origin:?}: missed {remain:?}"
                            )
                        }
                    }
                }

                domain.handle_hold_state();

                Result::<_, DispatchErrorWithPostInfo>::Ok((root, size))
            })?;
            Self::deposit_event(Event::NewAggregationReceipt {
                domain_id,
                aggregation_id,
                receipt: root,
            });
            Ok(Some(T::WeightInfo::aggregate(size)).into())
        }

        #[pallet::call_index(1)]
        /// Register a new domain. It holds a deposit for all the storage that the domain need. The account that
        /// requested this domain will be the owner and is the only one that can unregister it. Unregister the domain
        /// will unlock the deposit and remove the domain from the system.
        ///
        /// - aggregation_size: The size of the aggregation, in other words how many statements any aggregation have.
        /// - queue_size: The maximum number of aggregations that can be in the queue for this domain.
        pub fn register_domain(
            origin: OriginFor<T>,
            aggregation_size: AggregationSize,
            queue_size: Option<u32>,
        ) -> DispatchResultWithPostInfo {
            let owner = User::<T::AccountId>::from_origin::<T>(origin)?;
            let id = Self::next_domain_id();
            let queue_size = queue_size.unwrap_or(T::MaxPendingPublishQueueSize::get());

            let ticket = owner
                .clone()
                .owner()
                .and_then(|a| {
                    T::Consideration::new(
                        a,
                        Footprint::from_parts(
                            1,
                            Domain::<T>::compute_encoded_size(aggregation_size, queue_size),
                        ),
                    )
                    .transpose()
                })
                .transpose()?;
            let domain = Domain::<T>::try_create(
                id,
                owner.clone(),
                1,
                aggregation_size,
                queue_size,
                ticket,
            )?;
            Domains::<T>::insert(id, domain);
            NextDomainId::<T>::put(id + 1);
            Self::deposit_event(Event::NewDomain { id });

            Ok(owner.post_info(None))
        }

        /// Hold a domain. Put the domain in `Hold` or `Removable` state. Only the domain owner
        /// and the manager can do it.
        ///
        /// Once you call this function the domain state could be:
        /// - `Hold`: There are some aggregations that should be aggregated
        /// - `Removable`: the domain is ready to be removed because there are no more aggregations to be
        /// aggregated.
        ///
        /// Once the domain go in hold state cannot receive new proofs at all and cannot become in the `Ready`
        /// state again.
        ///
        /// **Only when the domain is in `Removable` state** you can call `unregister_domain` extrinsic to
        /// actually remove it.
        ///
        /// The `DomainStateChanged` event is emitted when the domain change its state.
        ///
        /// This call fails if the domain is not in `Ready` state or if the use cannot manage this domain.
        ///
        /// Arguments
        /// - domain_id: The domain identifier.
        #[pallet::call_index(2)]
        pub fn hold_domain(origin: OriginFor<T>, domain_id: u32) -> DispatchResultWithPostInfo {
            let owner = User::<T::AccountId>::from_origin::<T>(origin)?;
            Domains::<T>::try_mutate_exists(domain_id, |domain| {
                match domain {
                    Some(domain) if owner.can_handle_domain::<T>(domain) => {
                        if domain.state == DomainState::Ready {
                            domain.update_hold_state();
                            domain.emit_state_changed_event();
                        } else {
                            Err(Error::<T>::InvalidDomainState)?
                        }
                    }
                    Some(_) => Err(BadOrigin)?,
                    None => Err(Error::<T>::UnknownDomainId)?,
                };
                Ok::<_, DispatchError>(())
            })?;

            Ok(owner.post_info(None))
        }

        /// Unregister an empty domain that was put on hold previously and is in `Removable` state. Only
        /// the domain owner and the manager can do it. This will remove the domain from the system and
        /// unhold all the funds that the owner had bond.
        ///
        /// If you want to remove a domain you should put call `hold_domain` before and waiting that become
        /// `Removable`.
        ///
        /// If the domain can be removed a `DomainStateChanged` event with the `Removed` state is emitted.
        ///
        /// Arguments
        /// - domain_id: The domain identifier.
        ///
        #[pallet::call_index(3)]
        pub fn unregister_domain(
            origin: OriginFor<T>,
            domain_id: u32,
        ) -> DispatchResultWithPostInfo {
            let owner = User::<T::AccountId>::from_origin::<T>(origin)?;
            Domains::<T>::try_mutate_exists(domain_id, |domain| {
                *domain = match domain {
                    Some(domain) if owner.can_handle_domain::<T>(domain) => {
                        if domain.state != DomainState::Removable {
                            Err(Error::<T>::InvalidDomainState)?
                        } else {
                            if let (Some(o), Some(t)) = (owner.owner(), domain.ticket.take()) {
                                let _ =
                                    t.drop(o).defensive_proof("Drop should always succeed: qed");
                            }
                            domain.state = DomainState::Removed;
                            domain.emit_state_changed_event();
                            None
                        }
                    }
                    Some(_) => Err(BadOrigin)?,
                    None => Err(Error::<T>::UnknownDomainId)?,
                };
                Ok::<_, DispatchError>(())
            })?;

            Ok(owner.post_info(None))
        }
    }

    fn estimate_publish_aggregation_fee<T: Config>(size: AggregationSize) -> BalanceOf<T> {
        T::EstimateCallFee::estimate_call_fee(
            &Call::aggregate {
                domain_id: 0,
                aggregation_id: 0,
            },
            Some(T::WeightInfo::aggregate(size)).into(),
        )
    }

    fn dispatch_post_error<T: Config>(
        weight: Weight,
        error: Error<T>,
    ) -> DispatchErrorWithPostInfo {
        DispatchErrorWithPostInfo {
            post_info: Some(weight).into(),
            error: error.into(),
        }
    }

    impl<A> User<A> {
        pub fn from_origin<T: Config<AccountId = A>>(
            origin: OriginFor<T>,
        ) -> Result<Self, BadOrigin> {
            match T::ManagerOrigin::ensure_origin(origin.clone()) {
                Ok(_) => Ok(User::Manager),
                Err(_) => ensure_signed(origin.clone()).map(User::Owner),
            }
        }

        pub fn can_handle_domain<T: Config<AccountId = A>>(&self, domain: &Domain<T>) -> bool
        where
            A: PartialEq + sp_std::fmt::Debug,
        {
            match self {
                User::Owner(_) => &domain.owner == self,
                User::Manager => true,
            }
        }

        pub fn post_info(&self, actual_weight: Option<Weight>) -> PostDispatchInfo {
            PostDispatchInfo {
                actual_weight,
                pays_fee: self.pays(),
            }
        }

        pub fn pays(&self) -> Pays {
            match self {
                User::Owner(_owner) => Pays::Yes,
                _ => Pays::No,
            }
        }
    }
}
