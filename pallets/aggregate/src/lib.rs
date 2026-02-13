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

//! This pallet provides a mechanism for tracking, aggregating statements (i.e., proof
//! verification submissions) from users and dispatch them to another chain. It is possible
//! to define different aggregation
//! sizes and thresholds for different domains, moreover, for every domain it's possible to
//! define a _channel_: and endpoint for dispatching the aggregations to other chains.
//!
//! Every proof should indicate in which domain should be aggregated and then dispatched.
//! It provides `aggregate` extrinsic, a semi-permission-less that aggregates the proofs and provides
//! a little tip to the one who calls it: this tip (should) cover all costs about executing
//! aggregate and a configurable optional extra. If a domain also defines a destination chain for
//! dispatching the aggregations, the aggregations will be delivered to this chain and every
//! submitter should pay for this job.
//!
//! Register a new domain with `register_domain` needs to hold some balance to cover the cost
//! of storage space used by all proofs hash that living in this domain while waiting for the
//! `aggregate` call. All hold balances will be freed after the `unregister_domain` call (if any):
//! the `unregister_domain` can be done only after call `hold_domain` extrinsic, and there are no
//! pending aggregations. When you put a domain in the `Hold ` state, it cannot receive anymore
//! statements, and it's just possible to aggregate all pending aggregations. The domain state
//! becomes `Removable` when there are no more pending aggregations and allowlist submitter is
//! empty: only now is it possible to
//! call `unregister_domain` and free the held balance.
//!
//! The domain owner can change the total delivery fee for dispatching the aggregations by invoking
//! `set_total_delivery_fee` extrinsic. All the statements added to the domain compute their total delivery fee
//! according to this fee divided by the aggregation size declared in the domain.
//!
//! The `aggregate` extrinsic is a semi-permission-less call because a domain owner could decide
//! if:
//!
//! - Everyone can call it
//! - Everyone can call it only on the completed aggregations (otherwise only owner and manager can call it)
//! - Just owner and manager can call it
//!

extern crate alloc;

pub use pallet::*;
pub use weight::WeightInfo;

mod data;
pub mod migrations;

mod benchmarking;
mod mock;
mod should;

mod weight;

// Export the benchmarking utils.
#[cfg(feature = "runtime-benchmarks")]
pub use benchmarking::utils::*;

#[frame_support::pallet]
pub mod pallet {
    use core::{
        fmt::Debug,
        ops::{Deref, DerefMut},
    };

    pub use crate::data::{AggregateSecurityRules, AggregationSize, ProofSecurityRules};
    use crate::data::{
        CountableTicket, Delivery, DeliveryParams, DomainState, Reserve, StatementEntry, User,
    };

    use super::WeightInfo;
    use crate::Error::InvalidDomainParams;
    use alloc::vec::Vec;
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
    use hp_dispatch::{Destination, DispatchAggregation};
    use hp_on_proof_verified::OnProofVerified;
    use sp_core::H256;
    use sp_runtime::{
        traits::{BadOrigin, Keccak256},
        SaturatedConversion,
    };

    /// Given a `Configuration` return the Account type.
    pub type AccountOf<T> = <T as frame_system::Config>::AccountId;
    /// Given a `Configuration` return the Balance type.
    pub type BalanceOf<T> =
        <<T as Config>::Hold as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
    /// Return the call (extrinsic) type for that pallet.
    pub type CallOf<T> = Call<T>;
    /// Given a `Configuration` return the ConsiderationDomain type.
    pub(crate) type TicketDomainOf<T> = <T as Config>::ConsiderationDomain;
    /// Given a `Configuration` return the ConsiderationAllowList type.
    pub(crate) type TicketAllowListOf<T> = <T as Config>::ConsiderationAllowList;

    /// The in-code storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    /// The pallet component.
    pub struct Pallet<T>(_);

    /// This trait defines how the pallet should compute the tip for the publisher.
    /// This tip will be added to the cost amount estimation of the transaction.
    pub trait ComputePublisherTip<B> {
        /// Given an estimated cost of a transaction, return an optional tip to the publisher.
        fn compute_tip(estimated: B) -> Option<B>;
    }

    impl<B> ComputePublisherTip<B> for () {
        fn compute_tip(estimated: B) -> Option<B> {
            Some(estimated)
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
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
        type ConsiderationDomain: Consideration<Self::AccountId, Footprint>;
        /// A means of providing some cost while allowlisted is stored on-chain.
        type ConsiderationAllowList: Consideration<Self::AccountId, Footprint>;
        /// What should we use to estimate aggregate cost, pallet-transaction-payment implements it.
        type EstimateCallFee: EstimateCallFee<Call<Self>, BalanceOf<Self>>;
        /// How to compute the fee for publishing an aggregation.
        type ComputePublisherTip: ComputePublisherTip<BalanceOf<Self>>;
        /// The weight definition for this pallet
        type WeightInfo: WeightInfo;
        /// The (max) size of aggregations used in benchmarks. NEEDS to be equal to AggregationSize::get().
        /// Used in benchmarks
        #[cfg(feature = "runtime-benchmarks")]
        const AGGREGATION_SIZE: u32;
        /// The currency trait, used in benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        type Currency: frame_support::traits::fungible::Mutate<AccountOf<Self>>;
        /// Handler for when an aggregation is completed
        type DispatchAggregation: DispatchAggregation<BalanceOf<Self>, AccountOf<Self>>;
        /// The (max) size of the submitter list used in benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        const SUBMITTER_LIST_MAX_SIZE: u32;
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
                if !domain.is_authorized_to_add_proof(&account) {
                    log::warn!("Invalid proof submitter, skip");
                    Self::deposit_event(Event::<T>::CannotAggregate {
                        statement,
                        cause: CannotAggregateCause::UnauthorizedUser,
                    });

                    return;
                }

                // Reserve balance for publication: if not raise a fail event
                let Ok(reserve) = domain.reserve_currency(&account).inspect_err(|err| {
                    Self::deposit_event(Event::<T>::CannotAggregate {
                        statement,
                        cause: CannotAggregateCause::InsufficientFunds,
                    });

                    log::debug!("Failed to reserve balance {err:?} [aggregation]");
                }) else {
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

        fn weight(domain_id: &Option<u32>) -> Weight {
            match domain_id {
                Some(_) => T::WeightInfo::on_proof_verified(),
                None => Default::default(),
            }
        }
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// This domain id is unknown.
        UnknownDomainId,
        /// This aggregation cannot be published, or it's already published.
        InvalidAggregationId,
        /// The domain params are invalid or overflow in allowlist consideration.
        InvalidDomainParams,
        /// Try to remove or hold a domain in an invalid state.
        InvalidDomainState,
        /// Try to register a domain without any defined ownership (maybe a manager that didn't provide the delivery owner).
        MissedDeliveryOwnership,
        /// Cannot create a new, unique, aggregation id anymore
        NextAggregationIdUnavailable,
    }

    #[derive(
        Debug, Clone, PartialEq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
    )]
    /// The cause of a missed aggregation.
    pub enum CannotAggregateCause {
        /// No account
        NoAccount,
        /// The requested domain doesn't exist.
        DomainNotRegistered {
            /// The domain identifier.
            domain_id: u32,
        },
        /// The domain's should-publish-queue is full.
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
        /// The user that submitted proof is not authorized on this domain.
        UnauthorizedUser,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    /// Emitted events.
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
            /// The identifier of the aggregation.
            aggregation_id: u64,
        },
        /// The aggregation is complete.
        AggregationComplete {
            /// The domain identifier.
            domain_id: u32,
            /// The identifier of the aggregation.
            aggregation_id: u64,
        },
        /// A new aggregation receipt has been emitted.
        NewAggregationReceipt {
            /// The domain identifier.
            domain_id: u32,
            /// The identifier of the aggregation.
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
        TicketDomainOf<T>,
        TicketAllowListOf<T>,
    >;

    /// Shortcut to get the Domain type from config.
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub(crate) struct Domain<T: Config>(pub(crate) DomainType<T>);

    impl<T: Config> Domain<T> {
        /// Create a new domain
        ///
        /// - `id`: id of the new domain.
        /// - `owner`: the account that owns (i.e. can hold/unregister) the domain.
        /// - `next_aggregation_id`: the id of the first aggregation of the domain
        /// - `max_aggregation_size`: the maximum number of statements per aggregation for this domain.
        /// - `publish_queue_size`: the maximum number of unpublished aggregations for this domain.
        /// - `aggregate_rules`: the rules for securing the aggregate.
        /// - `ticket`: a tracker for the deposit associated with this domain.
        /// - `delivery`: delivery parameters for the domain.
        #[allow(clippy::too_many_arguments)]
        pub fn try_create(
            id: u32,
            owner: User<AccountOf<T>>,
            next_aggregation_id: u64,
            max_aggregation_size: AggregationSize,
            publish_queue_size: u32,
            aggregate_rules: AggregateSecurityRules,
            proof_rules: ProofSecurityRules,
            ticket_domain: Option<TicketDomainOf<T>>,
            ticket_allowlist: Option<CountableTicket<TicketAllowListOf<T>>>,
            delivery: DeliveryParams<AccountOf<T>, BalanceOf<T>>,
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
                    aggregate_rules,
                    proof_rules,
                    ticket_domain,
                    ticket_allowlist,
                    delivery,
                )))
            }
        }

        /// Compute and reserve the currency for further publication
        fn reserve_currency(
            &self,
            account: &AccountOf<T>,
        ) -> Result<Reserve<BalanceOf<T>>, DispatchError> {
            let estimated = estimate_publish_aggregation_fee::<T>(self.max_aggregation_size);
            let aggregate = (estimated.defensive_saturating_add(
                <T as Config>::ComputePublisherTip::compute_tip(estimated).unwrap_or_default(),
            )) / self.next.size.into();
            let total_fee = self.delivery.total_fee() / self.next.size.into();

            T::Hold::hold(&HoldReason::Aggregation.into(), account, aggregate)?;
            T::Hold::hold(&HoldReason::Delivery.into(), account, total_fee).inspect_err(|_| {
                T::Hold::release(
                    &HoldReason::Aggregation.into(),
                    account,
                    aggregate,
                    Precision::Exact,
                )
                .expect("Should be present because we hold it just before: qed");
            })?;

            Ok(Reserve::new(aggregate, total_fee))
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
        /// - `publish_queue_size`: The should-publish-queue size for this domain.
        /// - `destination`: The destination chain to delivery aggregations.
        pub fn compute_encoded_size(
            max_aggregation_size: AggregationSize,
            publish_queue_size: u32,
            destination: &Destination,
        ) -> usize {
            DomainType::<T>::compute_encoded_size(
                max_aggregation_size,
                publish_queue_size,
                destination,
            )
        }

        /// Add a list of submitter to a given domain iff the domain is configured with the right rules
        pub fn try_add_submitters(
            &mut self,
            submitters: &[AccountOf<T>],
        ) -> Result<(), DispatchError> {
            if self.proof_rules != ProofSecurityRules::OnlyAllowlisted {
                Err(Error::<T>::InvalidDomainParams)?
            };
            self.add_submitters(submitters)
        }

        /// Remove a list of submitter to a given domain iff the domain is configured with the right rules
        pub fn try_remove_submitters(
            &mut self,
            submitters: &[AccountOf<T>],
        ) -> Result<(), DispatchError> {
            if self.proof_rules != ProofSecurityRules::OnlyAllowlisted {
                Err(Error::<T>::InvalidDomainParams)?
            };
            self.remove_submitters(submitters)
        }

        /// Update the hold state according to the domain state and the allowlist status.
        pub fn update_hold_state(&mut self) {
            self.state = if self.should_publish.is_empty()
                && self.next.statements.is_empty()
                && self.is_allowlist_empty()
            {
                DomainState::Removable
            } else {
                DomainState::Hold
            };
        }

        /// Return the next non-empty aggregation to be published, or none if the aggregation is empty.
        /// If successful, a new aggregation is created as the next to be published.
        fn pop_next_aggregation(&mut self) -> Option<Aggregation<T>> {
            if self.next.statements.is_empty() {
                None
            } else {
                let new_aggregation = self.next.create_next(self.next.size).unwrap_or_else(|| {
                    // Cannot create a new aggregation. Must hold the domain.
                    self.state = DomainState::Hold;
                    self.emit_state_changed_event();
                    // Return a dummy aggregation with which replacing the old one.
                    // Domain is in the Hold state; so no-one can submit proofs or call aggregate
                    // on top of this new one.
                    crate::data::AggregationEntry::create(0, self.next.size)
                });

                Some(core::mem::replace(&mut self.next, new_aggregation))
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
            reserve: Reserve<BalanceOf<T>>,
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
            // If it is full, send an alert event
            if self.should_publish.len() >= self.publish_queue_size as usize {
                Pallet::<T>::deposit_event(Event::<T>::DomainFull { domain_id: self.id });
            }
        }

        /// Implement the hold state machine and emits the state if changed.
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

        /// Return true iff the submitting account is allowed to add proofs to this domain, according
        /// to the configured ProofSecurityRules.
        fn is_authorized_to_add_proof(&self, account: &T::AccountId) -> bool {
            match self.proof_rules {
                ProofSecurityRules::Untrusted => true,
                ProofSecurityRules::OnlyOwner => {
                    account
                        == self
                            .owner
                            .account()
                            .expect("The domain does not have an owner; qed")
                }
                ProofSecurityRules::OnlyAllowlisted => {
                    SubmittersAllowlist::<T>::get(self.id, account).is_some()
                }
            }
        }

        fn add_submitters(&mut self, submitters: &[AccountOf<T>]) -> Result<(), DispatchError> {
            let count = submitters
                .iter()
                .filter(|s| !SubmittersAllowlist::<T>::contains_key(self.id, s))
                .cloned()
                .map(|s| SubmittersAllowlist::<T>::insert(self.id, s, ()))
                .count();
            self.increase_footprint_count(count.saturated_into())
        }

        fn remove_submitters(&mut self, submitters: &[AccountOf<T>]) -> Result<(), DispatchError> {
            let count = submitters
                .iter()
                .filter(|s| SubmittersAllowlist::<T>::contains_key(self.id, s))
                .cloned()
                .map(|s| SubmittersAllowlist::<T>::remove(self.id, s))
                .count();
            self.decrease_footprint_count(count.saturated_into())
        }

        fn is_allowlist_empty(&self) -> bool {
            use frame_support::StorageDoubleMap;
            self.proof_rules != ProofSecurityRules::OnlyAllowlisted
                || !SubmittersAllowlist::<T>::contains_prefix(self.id)
        }

        /// Increase the footprint count of the domain.
        ///
        /// We need to take track of the count elements because the size is `0` and we would bound
        /// some amount for each entry, and the `LinearStoragePrice` cannot be used in this case
        /// (`0` size means `0` amount).
        ///
        /// On the other side, the consideration ticket doesn't save the footprint itself, so we
        /// need to introduce a `CountableTicket` that just holds the count and the ticket.
        ///
        /// The `update` ticket call will update the bound on the owner according to the new
        /// amount computation.
        /// The amount computation is delegated to the consideration implementation.
        ///
        fn increase_footprint_count(&mut self, amount: u64) -> Result<(), DispatchError> {
            // If the owner is _not an account_ cannot own any ticket.
            let owner = match self.owner.account() {
                Some(owner) => owner.clone(),
                None => return Ok(()),
            };
            self.ticket_allowlist = match self.ticket_allowlist.take() {
                Some(CountableTicket { count, ticket }) => {
                    // Here we update the ticket that bounds the amount of the owner accordingly to
                    // the consideration implementation and return the new ticket.
                    let count = count
                        .checked_add(amount.saturated_into())
                        .ok_or(Error::<T>::InvalidDomainParams)?;
                    let ticket = ticket.update(&owner, Footprint::from_parts(count as usize, 0))?;
                    Some(CountableTicket { count, ticket })
                }
                None => return Ok(()),
            };
            Ok(())
        }

        /// Decrease the footprint count of the domain.
        ///
        /// See [`increase_footprint_count`] for more details about implentation.
        ///
        fn decrease_footprint_count(&mut self, amount: u64) -> Result<(), DispatchError> {
            let owner = match self.owner.account() {
                Some(owner) => owner.clone(),
                None => return Ok(()),
            };
            self.ticket_allowlist = match self.ticket_allowlist.take() {
                Some(CountableTicket { count, ticket }) => {
                    let count = count
                        .checked_sub(amount.saturated_into())
                        .ok_or(Error::<T>::InvalidDomainParams)?;
                    let ticket = ticket.update(&owner, Footprint::from_parts(count as usize, 0))?;
                    Some(CountableTicket { count, ticket })
                }
                None => return Ok(()),
            };
            Ok(())
        }
    }

    impl<T: Config> Deref for Domain<T> {
        type Target = DomainType<T>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Config> DerefMut for Domain<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    /// A reason for this pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// The funds are held for aggregation pay.
        Aggregation,
        /// The funds are held as storage deposit for anything related to a domain.
        Domain,
        /// The funds are held for delivery.
        Delivery,
        /// Allowlisted Elements
        Allowlist,
    }

    /// Domains storage
    #[pallet::storage]
    #[pallet::getter(fn next_domain_id)]
    pub(crate) type NextDomainId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Domains storage
    #[pallet::storage]
    pub(crate) type Domains<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = u32, Value = Domain<T>>;

    /// Allowed Submitters
    #[pallet::storage]
    #[pallet::unbounded]
    pub(crate) type SubmittersAllowlist<T: Config> = StorageDoubleMap<
        Hasher1 = Blake2_128Concat,
        Key1 = u32,
        Hasher2 = Blake2_128Concat,
        Key2 = T::AccountId,
        Value = (),
    >;

    #[pallet::storage]
    #[pallet::getter(fn published)]
    #[pallet::unbounded]
    /// Vector of published aggregations. This will stay just in one block because we remove
    /// this vector at the start of every block (on_initialize hook).
    /// It is implicitly limited by the number of aggregate extrinsics that can fit in the block.
    pub type Published<T: Config> = StorageValue<_, Vec<(u32, Aggregation<T>)>, ValueQuery>;

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
    /// Cannot generate the proof of the aggregated statement.
    pub enum PathRequestError {
        /// The statement is not found in the aggregation.
        NotFound(u32, u64, H256),
        /// The receipt is not published for the given domain and aggregation.
        ReceiptNotPublished(u32, u64),
        /// The index of the statement exceeds the maximum that can be handled
        IndexOutOfBounds,
    }

    impl<T: Config> Pallet<T> {
        /// Compute the statement Merkle path giving a proof of the aggregated statement.
        /// - domain_id: The domain identifier.
        /// - aggregation_id: The identifier of the aggregation.
        /// - statement: The statement hashes that describe the proof for which we would provide a
        ///   proof.
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
                leaves,
                index
                    .try_into()
                    .map_err(|_| PathRequestError::IndexOutOfBounds)?,
            ))
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Published::<T>::kill();
            T::DbWeight::get().writes(1_u64)
        }
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Publish the aggregation. This call is used to publish a new aggregation in
        /// the domain to be published queue or is still not completed. Can be called according to the
        /// [`AggregateSecurityRules`] configured for the domain and, if conditions are met
        ///
        /// - move the funds held for aggregation to the caller account or return them to the submitters
        ///   if the caller is a manager;
        /// - move the funds held for delivery to the delivery owner.
        ///
        /// If the aggregation id is not valid (in _to be published_ queue or in filling stage), the call will
        /// fail, but the weight cost will be still the one needed to do the check.
        ///
        /// If conditions are met, an `Event::NewAggregationReceipt` is emitted.
        ///
        /// Arguments:
        /// - `domain_id`: The domain identifier.
        /// - `aggregation_id`: The identifier of the aggregation.
        ///
        /// Errors:
        /// - `BadOrigin`: If the origin is not valid, or it's not authorized to do it according to
        ///   the domain's [`AggregateSecurityRules`].
        /// - `UnknownDomainId`: If the domain id doesn't exist.
        /// - `InvalidAggregationId`: If the aggregation id doesn't exist.
        /// - Any error related to the delivery channel.
        #[pallet::weight(T::WeightInfo::aggregate(T::AggregationSize::get()) + T::DispatchAggregation::max_weight()
        )]
        #[pallet::call_index(0)]
        pub fn aggregate(
            origin: OriginFor<T>,
            domain_id: u32,
            aggregation_id: u64,
        ) -> DispatchResultWithPostInfo {
            let aggregator = User::<T::AccountId>::from_origin::<T>(origin)?;
            let (root, size, destination, delivery_owner, delivery_fee) =
                Domains::<T>::try_mutate(domain_id, |domain| {
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
                    if !domain.aggregate_rules.can_user_aggregate_it::<T>(
                        &aggregator,
                        &domain.owner,
                        &domain.delivery.owner,
                        &aggregation,
                    ) {
                        Err(BadOrigin)?
                    }
                    let root = aggregation.compute_receipt();
                    let size = aggregation.statements.len() as u32;
                    for s in aggregation.statements.iter() {
                        handle_held_funds::<T>(
                            HoldReason::Aggregation,
                            &s.account,
                            aggregator.account(),
                            s.reserve.aggregate,
                        );
                        handle_held_funds::<T>(
                            HoldReason::Delivery,
                            &s.account,
                            Some(&domain.delivery.owner),
                            s.reserve.delivery,
                        );
                    }
                    Published::<T>::mutate(|published: &mut _| {
                        published.push((domain_id, aggregation))
                    });

                    domain.handle_hold_state();

                    Result::<_, DispatchErrorWithPostInfo>::Ok((
                        root,
                        size,
                        domain.delivery.destination().clone(),
                        domain.delivery.owner.clone(),
                        *domain.delivery.fee(),
                    ))
                })?;
            Self::deposit_event(Event::NewAggregationReceipt {
                domain_id,
                aggregation_id,
                receipt: root,
            });

            let dispatch_weight = T::DispatchAggregation::dispatch_weight(&destination);

            T::DispatchAggregation::dispatch_aggregation(
                domain_id,
                aggregation_id,
                root,
                destination,
                delivery_fee,
                delivery_owner,
            )?;

            Ok(aggregator.post_info((T::WeightInfo::aggregate(size) + dispatch_weight).into()))
        }

        #[pallet::call_index(1)]
        #[allow(clippy::too_many_arguments)]
        /// Register a new domain. It holds a deposit for all the storage that the domain needs.
        /// The account that requested this domain will be the owner and is the only one that can
        /// unregister it. Unregister the domain will unlock the deposit and remove the domain
        /// from the system.
        ///
        /// Just a manager can register a domain that uses bridge delivery.
        ///
        /// Arguments
        /// - aggregation_size: The size of the aggregation, in other words how many statements any aggregation have.
        /// - queue_size: The maximum number of aggregations that can be in the queue for this domain.
        /// - aggregate_rules: The rules permission to call `aggregate` on this domain (see [`AggregateSecurityRules`])
        /// - proof_rules: The rules permission to call `on_proof_verified` callback on this domain (see [`ProofSecurityRules`]);
        ///   if `OnlyAllowlisted` is selected, the domain will be created with an empty allowlist, use
        ///   [`allowlist_proof_submitters`] and [`remove_proof_submitters`] extrinsics can be used to
        ///   define the allowed submitters.
        /// - delivery: Params defining aggregation delivery (fee, destination ... [`Delivery`])
        /// - delivery_owner: An optional account that will receive the total delivery fee when the aggregations are delivered.
        ///   If not provided, the delivery owner will be the caller.
        ///
        /// Errors:
        /// - `BadOrigin`: If the origin cannot register a new domain.
        /// - `FundsUnavailable`: If the caller does not have enough funds to register the domain.
        ///
        pub fn register_domain(
            origin: OriginFor<T>,
            aggregation_size: AggregationSize,
            queue_size: Option<u32>,
            aggregate_rules: AggregateSecurityRules,
            proof_rules: ProofSecurityRules,
            delivery: Delivery<BalanceOf<T>>,
            delivery_owner: Option<AccountOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let caller = User::<T::AccountId>::from_origin::<T>(origin)?;
            let destination = delivery.destination.clone();
            if !caller.can_create_domain(&destination) {
                Err(BadOrigin)?
            }
            let delivery_owner = delivery_owner
                .or_else(|| caller.account().cloned())
                .ok_or(Error::<T>::MissedDeliveryOwnership)?;
            let id = Self::next_domain_id();
            if id == u32::MAX {
                log::error!("Reached max id: {id:?}. Cannot create new domain.");
                Err(Error::<T>::InvalidDomainParams)?
            }
            let queue_size = queue_size.unwrap_or(T::MaxPendingPublishQueueSize::get());
            let delivery = DeliveryParams::new(delivery_owner, delivery);

            let ticket_domain = caller
                .clone()
                .account()
                .map(|a| {
                    T::ConsiderationDomain::new(
                        a,
                        Footprint::from_parts(
                            1,
                            Domain::<T>::compute_encoded_size(
                                aggregation_size,
                                queue_size,
                                &destination,
                            ),
                        ),
                    )
                })
                .transpose()?;
            let ticket_allowlist = if proof_rules == ProofSecurityRules::OnlyAllowlisted {
                caller
                    .account()
                    .map(|a| T::ConsiderationAllowList::new(a, Footprint::from_parts(0, 0)))
                    .transpose()?
            } else {
                None
            };

            let ticket_allowlist =
                ticket_allowlist.map(|ticket| CountableTicket { count: 0, ticket });

            let domain = Domain::<T>::try_create(
                id,
                caller.clone(),
                1,
                aggregation_size,
                queue_size,
                aggregate_rules,
                proof_rules,
                ticket_domain,
                ticket_allowlist,
                delivery,
            )?;
            Domains::<T>::insert(id, domain);
            let next_id = id.checked_add(1).expect("Cannot overflow. QED");
            NextDomainId::<T>::put(next_id);
            Self::deposit_event(Event::NewDomain { id });

            Ok(caller.post_info(None))
        }

        /// Hold a domain. Put the domain in `Hold` or `Removable` state. Only the domain owner
        /// and the manager can do it.
        ///
        /// Once you call this function, the domain state could be:
        /// - `Hold`: There are some aggregations that should be aggregated or the allowlist submitters
        ///   set is not empty.
        /// - `Removable`: the domain is ready to be removed because there are no more aggregations to be
        /// aggregated and no allowed address in the allowlist set.
        ///
        /// The allowlist set could be populated iff the domain is configured with [`ProofSecurityRules::OnlyAllowlisted`]
        /// rule.
        ///
        /// Once the domain goes on hold, the state cannot receive new proofs at all and cannot become in the `Ready`
        /// state again.
        ///
        /// **Only when the domain is in `Removable` state** you can call `unregister_domain` extrinsic to
        /// actually remove it.
        ///
        /// The `DomainStateChanged` event is emitted when the domain changes its state.
        ///
        /// This call fails if the domain is not in the `Ready` state or if the user cannot manage this domain.
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
        /// unhold all the funds that the owner had bonded.
        ///
        /// If you want to remove a domain, you should put the call `hold_domain` before and waiting that become
        /// `Removable`. If the domain is configured to accept proof with
        /// [`ProofSecurityRules::OnlyAllowlisted`], you should take care to remove all allowed addresses
        /// from the set before removing the domain.
        ///
        /// If the domain can be removed, a `DomainStateChanged` event with the `Removed` state is emitted.
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
                            if let (Some(o), Some(t)) =
                                (owner.account(), domain.ticket_domain.take())
                            {
                                let _ =
                                    t.drop(o).defensive_proof("Drop should always succeed: qed");
                            }
                            if let (Some(o), Some(t)) = (
                                owner.account(),
                                domain.ticket_allowlist.take().map(|t| t.ticket),
                            ) {
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

        /// Set the total delivery aggregation fee. Every submitter will hold this fee (at the time of proof submission)
        /// divided by the aggregation size. When the aggregation is dispatched, all these held funds will be
        /// transferred to the delivery owner.
        ///
        /// Only a domain owner, delivery owner or manager can set the total fee.
        ///
        /// Arguments
        /// - domain_id: The domain identifier.
        /// - fee: The delivery fee.
        /// - owner_tip: The delivery owner tip.
        ///
        /// Errors:
        /// - `BadOrigin`: If the origin is not authorized.
        /// - `UnknownDomainId`: If the domain doesn't exist.
        ///
        #[pallet::call_index(4)]
        pub fn set_total_delivery_fee(
            origin: OriginFor<T>,
            domain_id: u32,
            fee: BalanceOf<T>,
            owner_tip: BalanceOf<T>,
        ) -> DispatchResult {
            let owner = User::<T::AccountId>::from_origin::<T>(origin)?;
            Domains::<T>::try_mutate_exists(domain_id, |domain| {
                match domain {
                    Some(domain) if owner.can_set_total_delivery_fee::<T>(domain) => {
                        domain.delivery.set_fee(fee);
                        domain.delivery.set_owner_tip(owner_tip);
                    }
                    Some(_) => Err(BadOrigin)?,
                    None => Err(Error::<T>::UnknownDomainId)?,
                };
                Ok::<_, DispatchError>(())
            })?;
            Ok(())
        }

        /// Add `submitters` to the set of allowlist submitters.
        /// The domain should be configured with the `ProofSecurityRules::OnlyAllowlisted` rules to
        /// handle it.
        ///
        /// Errors:
        /// - `BadOrigin`: If the origin is not authorized.
        /// - `UnknownDomainId`: If the domain doesn't exist.
        /// - `InvalidDomainParams`: If the domain is not configured with
        ///   `ProofSecurityRules::OnlyAllowlisted` or is not in the `DomainState::Ready` state.
        ///
        #[pallet::weight(T::WeightInfo::allowlist_proof_submitters(submitters.len().saturated_into()
        ))]
        #[pallet::call_index(5)]
        pub fn allowlist_proof_submitters(
            origin: OriginFor<T>,
            domain_id: u32,
            submitters: Vec<AccountOf<T>>,
        ) -> DispatchResult {
            let owner = User::<T::AccountId>::from_origin::<T>(origin)?;
            Domains::<T>::try_mutate_exists(domain_id, |maybe_domain| match maybe_domain {
                None => Err(Error::<T>::UnknownDomainId)?,
                Some(domain) if !owner.can_handle_domain::<T>(domain) => Err(BadOrigin)?,
                Some(domain) if domain.state != DomainState::Ready => {
                    Err(InvalidDomainParams::<T>)?
                }
                Some(domain) => domain
                    .try_add_submitters(submitters.as_slice())
                    .map_err(Into::<DispatchError>::into),
            })?;
            Ok(())
        }

        /// Remove `submitters` from the set of allowlist submitters.
        /// The domain should be configured with the `ProofSecurityRules::OnlyAllowlisted` rules to
        /// handle it.
        ///
        /// Errors:
        /// - `BadOrigin`: If the origin is not authorized.
        /// - `UnknownDomainId`: If the domain doesn't exist.
        /// - `InvalidDomainParams`: If the domain is not configured with
        ///   `ProofSecurityRules::OnlyAllowlisted`.
        ///
        #[pallet::weight(T::WeightInfo::remove_proof_submitters(submitters.len().saturated_into()))]
        #[pallet::call_index(6)]
        pub fn remove_proof_submitters(
            origin: OriginFor<T>,
            domain_id: u32,
            submitters: Vec<AccountOf<T>>,
        ) -> DispatchResult {
            let owner = User::<T::AccountId>::from_origin::<T>(origin)?;
            Domains::<T>::try_mutate_exists(domain_id, |maybe_domain| match maybe_domain {
                None => Err(Error::<T>::UnknownDomainId)?,
                Some(domain) if !owner.can_handle_domain::<T>(domain) => Err(BadOrigin)?,
                Some(domain) => domain
                    .try_remove_submitters(submitters.as_slice())
                    .map(|_| domain.handle_hold_state())
                    .map_err(Into::<DispatchError>::into),
            })?;

            Ok(())
        }
    }

    fn handle_held_funds<T: Config>(
        reason: HoldReason,
        account: &AccountOf<T>,
        dest: Option<&AccountOf<T>>,
        amount: BalanceOf<T>,
    ) {
        let transfer = if let Some(dest) = dest {
            T::Hold::transfer_on_hold(
                &reason.into(),
                account,
                dest,
                amount,
                Precision::BestEffort,
                Restriction::Free,
                Fortitude::Polite,
            )
        } else {
            T::Hold::release(&reason.into(), account, amount, Precision::BestEffort)
        }
        .expect("Call user should exists. qed");

        let remain = amount.defensive_saturating_sub(transfer);

        if remain > 0_u32.into() {
            log::warn!("Cannot refund all funds from {account:?} to {dest:?}: missed {remain:?}")
        };
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

    fn dispatch_post_error(
        weight: Weight,
        error: impl Into<DispatchError>,
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
                Err(_) => ensure_signed(origin).map(User::Account),
            }
        }

        pub fn is_manager(&self) -> bool {
            matches!(self, User::Manager)
        }

        pub fn can_handle_domain<T: Config<AccountId = A>>(&self, domain: &Domain<T>) -> bool
        where
            A: PartialEq + Debug + Ord + Clone + Encode,
        {
            match self {
                User::Account(_) => &domain.owner == self,
                User::Manager => true,
            }
        }

        pub fn can_set_total_delivery_fee<T: Config<AccountId = A>>(
            &self,
            domain: &Domain<T>,
        ) -> bool
        where
            A: PartialEq + Debug + Ord + Clone + Encode,
        {
            match self {
                User::Account(account) => {
                    &domain.owner == self || &domain.delivery.owner == account
                }
                User::Manager => true,
            }
        }

        pub fn can_create_domain(&self, _destination: &Destination) -> bool {
            // With only Destination::None available, any user can create a domain
            true
        }

        pub fn post_info(&self, actual_weight: Option<Weight>) -> PostDispatchInfo {
            PostDispatchInfo {
                actual_weight,
                pays_fee: self.pays(),
            }
        }

        pub fn pays(&self) -> Pays {
            match self {
                User::Manager => Pays::No,
                _ => Pays::Yes,
            }
        }
    }

    impl AggregateSecurityRules {
        fn can_user_aggregate_it<T: Config>(
            &self,
            aggregator: &User<T::AccountId>,
            domain_owner: &User<T::AccountId>,
            delivery_owner: &T::AccountId,
            aggregation: &Aggregation<T>,
        ) -> bool {
            let is_owner_auth = || {
                aggregator.account() == Some(delivery_owner)
                    || aggregator == domain_owner
                    || aggregator.is_manager()
            };
            match self {
                AggregateSecurityRules::Untrusted => true,
                AggregateSecurityRules::OnlyOwner => is_owner_auth(),
                AggregateSecurityRules::OnlyOwnerUncompleted => {
                    aggregation.completed() || is_owner_auth()
                }
            }
        }
    }
}
