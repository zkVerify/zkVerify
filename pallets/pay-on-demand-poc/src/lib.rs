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
            chain_id: Option<u32>,
            statement: H256,
        ) {
            log::info!("Proof: [{account:?}]-{chain_id:?} {statement:?}");
            if let Some(account) = account {
                let estimated = estimate_publish_attestation_fee::<T>();
                let reserve = (estimated
                    + <T as Config>::ComputeFeeFor::compute_fee(estimated).unwrap_or_default())
                    / T::AttestationSize::get().into();
                match <T as Config>::Currency::reserve(&account, reserve) {
                    Ok(_) => (),
                    Err(err) => {
                        Self::deposit_event(Event::<T>::CannotAttestStatement {
                            statement,
                            cause: CannotAttestCause::InsufficientFound,
                        });

                        log::debug!("Failed to reserve balance: {:?}", err);
                        return;
                    }
                }
                let to_publish = append_statement::<T>(account.clone(), reserve, statement);
                Self::deposit_event(Event::<T>::NewElement {
                    value: statement,
                    attestation_id: NextAttestation::<T>::get().id,
                });
                if let Some(attestation) = to_publish {
                    available_attestation::<T>(attestation);
                }
            } else {
                log::warn!("No account, skip");
            }
        }
    }

    fn append_statement<T: Config>(
        account: T::AccountId,
        reserve: BalanceOf<T>,
        pubs_hash: H256,
    ) -> Option<Attestation<T>> {
        NextAttestation::<T>::mutate(|attestation: &mut _| {
            attestation.statements.force_push(StatementEntry::new(
                account.clone(),
                reserve,
                pubs_hash,
            ));
            if attestation.is_complete() {
                Some(sp_std::mem::replace(attestation, attestation.next()))
            } else {
                None
            }
        })
    }

    fn available_attestation<T: Config>(attestation: Attestation<T>) {
        Pallet::<T>::deposit_event(Event::<T>::AvailableAttestation { id: attestation.id });
        ShouldPublished::<T>::insert(attestation.id, attestation);
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// This attestation cannot be published or it's already published.
        InvalidAttestationId,
        /// Too much attestations in a block.
        TooMuchAttestations,
    }

    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum CannotAttestCause {
        InsufficientFound,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewElement {
            value: H256,
            attestation_id: u64,
        },
        AvailableAttestation {
            id: u64,
        },
        NewAttestation {
            id: u64,
            attestation: H256,
        },
        CannotAttestStatement {
            statement: H256,
            cause: CannotAttestCause,
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
        pub(crate) statements: BoundedVec<StatementEntry<A, B>, S>,
    }

    impl<A, B, S: Get<u32>> AttestationEntry<A, B, S> {
        fn next(&self) -> Self {
            Self {
                id: self.id + 1,
                statements: BoundedVec::new(),
            }
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
            Self {
                id: 1,
                statements: Default::default(),
            }
        }
    }

    type Attestation<T> = AttestationEntry<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as Config>::AttestationSize,
    >;

    #[pallet::storage]
    pub type NextAttestation<T: Config> = StorageValue<_, Attestation<T>, ValueQuery>;

    #[pallet::storage]
    pub type ShouldPublished<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = u64, Value = Attestation<T>>;

    #[pallet::storage]
    #[pallet::getter(fn published)]
    pub type Published<T: Config> =
        StorageValue<_, BoundedVec<Attestation<T>, T::MaxPublishedPerBlock>, ValueQuery>;

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        /// Publish the attestation.
        #[pallet::call_index(0)]
        pub fn publish_attestation(origin: OriginFor<T>, id: u64) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            let attestation =
                ShouldPublished::<T>::take(id).ok_or(Error::<T>::InvalidAttestationId)?;

            let root = attestation.compute();
            for s in attestation.statements.iter() {
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

            Published::<T>::mutate(|published: &mut _| published.try_push(attestation))
                .map_err(|_| Error::<T>::TooMuchAttestations)?;

            Self::deposit_event(Event::NewAttestation {
                id,
                attestation: root,
            });

            Ok(().into())
        }
    }

    fn estimate_publish_attestation_fee<T: Config>() -> BalanceOf<T> {
        T::EstimateCallFee::estimate_call_fee(
            &Call::publish_attestation { id: 0 },
            Default::default(),
        )
    }
}
