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
    pub use crate::data::AggregationSize;
    use crate::data::{StatementEntry, User};

    use super::WeightInfo;
    #[cfg(feature = "runtime-benchmarks")]
    use frame_support::traits::ReservableCurrency;
    use frame_support::{
        dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::{
            fungible::{Inspect, InspectHold, MutateHold},
            tokens::{Fortitude, Precision, Restriction},
            Consideration, Defensive, EstimateCallFee, Footprint, VariantCount,
        },
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use sp_core::H256;
    use sp_runtime::traits::{BadOrigin, Keccak256, Saturating};
    use sp_std::vec::Vec;

    pub type AccountOf<T> = <T as frame_system::Config>::AccountId;
    pub type BalanceOf<T> =
        <<T as Config>::Hold as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
    type TicketOf<T> = <T as Config>::Consideration;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    pub trait ComputePublisherTip<B> {
        fn compute_tip(estimated: B) -> Option<B>;
    }

    impl<B> ComputePublisherTip<B> for () {
        fn compute_tip(estimated: B) -> Option<B> {
            Some(estimated)
        }
    }

    pub type CallOf<T> = Call<T>;

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
        type Consideration: Consideration<Self::AccountId>;
        /// What should we use to estimate publish aggregation cost (pallet-transaction-payment implement it)
        type EstimateCallFee: EstimateCallFee<Call<Self>, BalanceOf<Self>>;
        /// How to compute the fee for publishing an aggregation.
        type ComputePublisherTip: ComputePublisherTip<BalanceOf<Self>>;
        /// The weight definition for this pallet
        type WeightInfo: WeightInfo;
        /// The (max) size of aggregations used in benchmarks. NEED to be equals to AggregationSize::get()
        #[cfg(feature = "runtime-benchmarks")]
        const AGGREGATION_SIZE: u32;
        /// The weight definition for this pallet
        #[cfg(feature = "runtime-benchmarks")]
        type Currency: ReservableCurrency<AccountOf<Self>>;
    }

    impl<T: Config> hp_on_proof_verified::OnProofVerified<<T as frame_system::Config>::AccountId>
        for Pallet<T>
    {
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
                Self::deposit_event(Event::<T>::NewProof {
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
        /// The domain params are invalid.
        InvalidDomainParams,
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
        NewDomain {
            id: u32,
        },
        NewProof {
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
        AggregationRemoved {
            domain_id: u32,
            aggregation_id: u64,
        },
        CannotAggregate {
            statement: H256,
            cause: CannotAggregateCause,
        },
        DomainFull {
            domain_id: u32,
        },
    }

    /// Shortcut to get the Aggregation type from config.
    pub type Aggregation<T> =
        crate::data::AggregationEntry<AccountOf<T>, BalanceOf<T>, <T as Config>::AggregationSize>;

    /// Shortcut to get the Domain type from config.
    pub(crate) type Domain<T> = crate::data::DomainEntry<
        AccountOf<T>,
        BalanceOf<T>,
        <T as Config>::AggregationSize,
        <T as Config>::MaxPendingPublishQueueSize,
        TicketOf<T>,
    >;

    /// Compute and reserve the currency for further publication
    fn reserve_currency_for_publication<T: Config>(
        domain: &mut Domain<T>,
        account: &AccountOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let estimated = estimate_publish_attestation_fee::<T>(domain.max_aggregation_size);
        let hold = (estimated.saturating_add(
            <T as Config>::ComputePublisherTip::compute_tip(estimated).unwrap_or_default(),
        )) / domain.next.size.into();
        T::Hold::hold(&HoldReason::Aggregation.into(), account, hold).map(|_| hold)
    }

    /// Clean Domain
    fn clean_domain<T: Config>(domain: &mut Domain<T>) {
        sp_std::mem::take(&mut domain.should_publish)
            .into_iter()
            .chain(sp_std::iter::once((
                domain.next.id,
                sp_std::mem::take(&mut domain.next),
            )))
            .for_each(|(id, aggregation)| {
                Pallet::<T>::release_aggregation_founds(&aggregation);
                Pallet::<T>::deposit_event(Event::AggregationRemoved {
                    domain_id: domain.id,
                    aggregation_id: id,
                });
            });
        domain.next = domain.next.create_next(domain.max_aggregation_size);
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
    pub type Published<T: Config> = StorageValue<_, Vec<(u32, Aggregation<T>)>, ValueQuery>;

    impl<T: Config> Pallet<T> {
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

        fn ensure_domain_params(
            aggregation_size: AggregationSize,
            queue_size: u32,
        ) -> Result<(), DispatchError> {
            if aggregation_size == 0
                || aggregation_size > T::AggregationSize::get()
                || queue_size > T::MaxPendingPublishQueueSize::get()
            {
                Err(Error::<T>::InvalidDomainParams)?;
            }
            Ok(())
        }

        fn release_aggregation_founds(aggregation: &Aggregation<T>) {
            for s in &aggregation.statements {
                let _ = T::Hold::release(
                    &HoldReason::Aggregation.into(),
                    &s.account,
                    s.reserve,
                    Precision::BestEffort,
                )
                .defensive_proof("In besteffort mode should ever release funds: qed");
            }
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
        /// Publish the aggregation.
        #[pallet::weight(T::WeightInfo::aggregate(T::AggregationSize::get() as u32))]
        #[pallet::call_index(0)]
        pub fn aggregate(
            origin: OriginFor<T>,
            domain_id: u32,
            id: u64,
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
                let aggregation = domain.should_publish.remove(&id).ok_or_else(|| {
                    dispatch_post_error(
                        T::WeightInfo::aggregate_on_invalid_id(),
                        Error::<T>::InvalidAggregationId,
                    )
                })?;

                let root = aggregation.compute();
                let size = aggregation.size;
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
                                "Cannot refund all founds from {account:?} to {origin:?}: missed {remain:?}"
                            )
                        }
                    }
                }

                Result::<_, DispatchErrorWithPostInfo>::Ok((root, size))
            })?;
            Self::deposit_event(Event::NewAggregationReceipt {
                domain_id,
                aggregation_id: id,
                receipt: root,
            });
            Ok(Some(T::WeightInfo::aggregate(size as u32)).into())
        }

        #[pallet::call_index(1)]
        pub fn register_domain(
            origin: OriginFor<T>,
            aggregation_size: AggregationSize,
            queue_size: Option<u32>,
        ) -> DispatchResultWithPostInfo {
            let id = Self::next_domain_id();
            let owner = User::<T::AccountId>::from_origin::<T>(origin)?;
            let queue_size = queue_size.unwrap_or(T::MaxPendingPublishQueueSize::get());
            Self::ensure_domain_params(aggregation_size, queue_size)?;

            // T::Consideration::new()
            Self::deposit_event(Event::NewDomain { id });
            let ticket = owner
                .clone()
                .owner()
                .map(|a| {
                    T::Consideration::new(
                        a,
                        Footprint::from_parts(
                            1,
                            Domain::<T>::encoded_size(aggregation_size, queue_size),
                        ),
                    )
                })
                .transpose()?;
            let domain =
                Domain::<T>::create(id, owner.clone(), 1, aggregation_size, queue_size, ticket);
            Domains::<T>::insert(id, domain);
            NextDomainId::<T>::put(id + 1);

            Ok(owner.post_info(None))
        }

        #[pallet::call_index(2)]
        pub fn unregister_domain(
            origin: OriginFor<T>,
            domain_id: u32,
        ) -> DispatchResultWithPostInfo {
            let owner = User::<T::AccountId>::from_origin::<T>(origin)?;
            Domains::<T>::try_mutate_exists(domain_id, |domain| {
                *domain = match domain {
                    Some(domain) if owner.can_remove_domain::<T>(&domain) => {
                        clean_domain::<T>(domain);
                        match (owner.owner(), domain.ticket.take()) {
                            (Some(o), Some(t)) => {
                                let _ =
                                    t.drop(o).defensive_proof("Drop should always succeed: qed");
                            }
                            _ => {}
                        };
                        None
                    }
                    Some(_) => Err(BadOrigin)?,
                    None => Err(Error::<T>::UnknownDomainId)?,
                };
                Ok::<_, DispatchError>(())
            })?;

            Ok(owner.post_info(None))
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
    pub enum PathRequestError {
        NotFound(u32, u64, sp_core::H256),
        ReceiptNotPublished(u32, u64),
    }

    fn estimate_publish_attestation_fee<T: Config>(size: AggregationSize) -> BalanceOf<T> {
        T::EstimateCallFee::estimate_call_fee(
            &Call::aggregate {
                domain_id: 0,
                id: 0,
            },
            Some(T::WeightInfo::aggregate(size as u32)).into(),
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

        pub fn can_remove_domain<T: Config<AccountId = A>>(&self, domain: &Domain<T>) -> bool
        where
            A: PartialEq,
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
