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
mod tests;

pub trait WeightInfo {
    fn publish_attestation() -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
    use super::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_support::sp_runtime::traits::Keccak256;
    use frame_support::traits::{BalanceStatus, Currency, EstimateCallFee, ReservableCurrency};
    use frame_support::{sp_runtime::testing::H256, BoundedVec};
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::OriginFor;

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
        /// The (max) size of attestations.
        type AttestationSize: Get<u32>;
        /// The upperbound on the number of attestations that can be published per block.
        type MaxPublishedPerBlock: Get<u32>;
        /// The upperbound on the number of attestations that can stay in _to be published_ state
        /// for a single domain for wait a publish call.
        type MaxPendingPublishQueueSize: Get<u32>;
        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;
        /// What should we use to estimate pubblish attestaion cost (pallet-transaction-payment implement it)
        type EstimateCallFee: EstimateCallFee<Call<Self>, BalanceOf<Self>>;
        /// How to compute the fee for publishing an attestation.
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
            log::info!("Proof: [{account:?}]-{domain_id:?} {statement:?}");
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
                let Some(domain) = domain else {
                    log::warn!("No account, skip");
                    Self::deposit_event(Event::<T>::CannotAggregate {
                        statement,
                        cause: CannotAggregateCause::DomainNotRegistered { domain_id },
                    });

                    return;
                };
                let estimated = estimate_publish_attestation_fee::<T>();
                let reserve = (estimated
                    + <T as Config>::ComputeFeeFor::compute_fee(estimated).unwrap_or_default())
                    / domain.next_attestation.size.into();
                match <T as Config>::Currency::reserve(&account, reserve) {
                    Ok(_) => (),
                    Err(err) => {
                        Self::deposit_event(Event::<T>::CannotAggregate {
                            statement,
                            cause: CannotAggregateCause::InsufficientFound,
                        });

                        log::debug!("Failed to reserve balance: {:?}", err);
                        return;
                    }
                }
                Self::deposit_event(Event::<T>::NewElement {
                    value: statement,
                    domain_id,
                    attestation_id: domain.next_attestation.id,
                });
                let to_publish = append_statement::<T>(domain, account.clone(), reserve, statement);
                if let Some(attestation) = to_publish {
                    available_attestation::<T>(domain, attestation);
                }
            });
        }
    }

    fn append_statement<T: Config>(
        domain: &mut Domain<T>,
        account: T::AccountId,
        reserve: BalanceOf<T>,
        pubs_hash: H256,
    ) -> Option<Attestation<T>> {
        let attestation = &mut domain.next_attestation;
        attestation
            .statements
            .force_push(StatementEntry::new(account.clone(), reserve, pubs_hash));
        if attestation.size <= attestation.statements.len() as u32 {
            Some(sp_std::mem::replace(
                attestation,
                attestation.create_next(attestation.size),
            ))
        } else {
            None
        }
    }

    fn available_attestation<T: Config>(domain: &mut Domain<T>, attestation: Attestation<T>) {
        Pallet::<T>::deposit_event(Event::<T>::AvailableAttestation {
            domain_id: domain.id,
            id: attestation.id,
        });
        domain
            .should_publish
            .try_insert(attestation.id, attestation);
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// This domain id is unknown.
        UnknownDomainId,
        /// This attestation cannot be published or it's already published.
        InvalidAttestationId,
        /// Too much attestations in a block.
        TooMuchAttestations,
    }

    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum CannotAggregateCause {
        NoAccount,
        NoDomain,
        DomainNotRegistered { domain_id: u32 },
        InsufficientFound,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewElement {
            value: H256,
            domain_id: u32,
            attestation_id: u64,
        },
        AvailableAttestation {
            domain_id: u32,
            id: u64,
        },
        NewAttestation {
            domain_id: u32,
            id: u64,
            attestation: H256,
        },
        CannotAggregate {
            statement: H256,
            cause: CannotAggregateCause,
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
    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(S))]
    pub struct AttestationEntry<A, B, S: Get<u32>> {
        pub(crate) id: u64,
        pub(crate) size: u32,
        pub(crate) statements: BoundedVec<StatementEntry<A, B>, S>,
    }

    impl<A, B, S: Get<u32>> AttestationEntry<A, B, S> {
        fn create(id: u64, size: u32) -> Self {
            assert!(size <= S::get(), "Attestation size is out of bound");
            Self {
                id,
                size,
                statements: BoundedVec::new(),
            }
        }

        fn create_next(&self, size: u32) -> Self {
            Self::create(self.id + 1, size)
        }

        fn is_complete(&self) -> bool {
            self.statements.len() == BoundedVec::<(A, B, H256), S>::bound()
        }

        fn compute(&self) -> H256 {
            binary_merkle_tree::merkle_root::<Keccak256, _>(
                self.statements.iter().map(|s| s.statement.as_ref()),
            )
        }
    }

    impl<A, B, S: Get<u32>> Default for AttestationEntry<A, B, S> {
        fn default() -> Self {
            Self::create(1, S::get())
        }
    }

    type Attestation<T> = AttestationEntry<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as Config>::AttestationSize,
    >;

    /// A complete Verification Key or its hash.
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(S, M))]
    pub(crate) struct DomainEntry<A, B, S: Get<u32>, M: Get<u32>> {
        pub id: u32,
        pub next_attestation: AttestationEntry<A, B, S>,
        pub max_attestation_size: u32,
        pub should_publish: BoundedBTreeMap<u64, AttestationEntry<A, B, S>, M>,
    }

    impl<A, B, S: Get<u32>, M: Get<u32>> DomainEntry<A, B, S, M> {
        pub fn create(id: u32, next_attestation_id: u64, max_attestation_size: u32) -> Self {
            assert!(
                max_attestation_size <= S::get(),
                "Max attestation size must be less than or equal to Config::AttestationSize"
            );
            Self {
                id,
                next_attestation: AttestationEntry::create(
                    next_attestation_id,
                    max_attestation_size,
                ),
                max_attestation_size,
                should_publish: Default::default(),
            }
        }
    }

    pub(crate) type Domain<T> = DomainEntry<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as Config>::AttestationSize,
        <T as Config>::MaxPendingPublishQueueSize,
    >;

    #[pallet::storage]
    pub(crate) type Domains<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = u32, Value = Domain<T>>;

    #[pallet::storage]
    #[pallet::getter(fn published)]
    pub type Published<T: Config> =
        StorageValue<_, BoundedVec<Attestation<T>, T::MaxPublishedPerBlock>, ValueQuery>;

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Publish the attestation.
        #[pallet::call_index(0)]
        pub fn publish_attestation(
            origin: OriginFor<T>,
            domain_id: u32,
            id: u64,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;
            let root = Domains::<T>::try_mutate(domain_id, |domain| {
                let domain = domain.as_mut().ok_or(Error::<T>::TooMuchAttestations)?;
                let attestation = domain
                    .should_publish
                    .remove(&id)
                    .ok_or(Error::<T>::InvalidAttestationId)?;

                let root = attestation.compute();
                Published::<T>::mutate(|published: &mut _| published.try_push(attestation))
                    .map_err(|_| Error::<T>::TooMuchAttestations)?;

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
                Self::deposit_event(Event::NewAttestation {
                    domain_id,
                    id,
                    attestation: root,
                });
            }
            Ok(().into())
        }
    }

    fn estimate_publish_attestation_fee<T: Config>() -> BalanceOf<T> {
        T::EstimateCallFee::estimate_call_fee(
            &Call::publish_attestation {
                domain_id: 0,
                id: 0,
            },
            Default::default(),
        )
    }
}
