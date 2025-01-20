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

//! A pallet implementing the possibility of making airdrops on-chain and letting **Beneficiaries**
//! manually claim the amount of tokens they have right to.
//! Only **ManagerOrigin** is able to start and end airdrops, as well as adding beneficiaries with
//! their rightful balance.
//! Currently it possible only to held one airdrop at a time.
//! When airdrop ends, all the funds still available in the pallet's associated account
//! are transferred to **UnclaimedDestination**.
//!
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod weight;
use core::marker::PhantomData;

extern crate alloc;

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};

use frame_support::{
    dispatch::{DispatchResult, PostDispatchInfo},
    traits::{
        fungible::{Inspect, Mutate},
        tokens::{Fortitude, Pay, Preservation},
        Get,
    },
    PalletId,
};

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub use pallet::*;
pub use weight::WeightInfo;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The claim's pallet id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Manager allowed to add/remove beneficiaries
        type ManagerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Type for processing spends of [Self::AssetKind] in favor of [`Self::Beneficiary`].
        type Paymaster: Pay<
            Beneficiary = Self::AccountId,
            AssetKind = (),
            Balance = BalanceOf<Self>,
        >;

        /// The staking balance.
        type Currency: Mutate<Self::AccountId>;

        /// Destination for unclaimed assets
        type UnclaimedDestination: Get<Self::AccountId>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The maximum number of beneficiaries used in benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        const MAX_BENEFICIARIES: u32;
    }

    /// Candidates eligible to receive an airdrop with the associated balance they have right to
    #[pallet::storage]
    #[pallet::getter(fn beneficiaries)]
    pub type Beneficiaries<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

    /// Total tokens claimable from the current airdrop.  
    #[pallet::storage]
    #[pallet::getter(fn total_claimable)]
    pub type TotalClaimable<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Whether there is an active airdrop or not
    #[pallet::storage]
    #[pallet::getter(fn aidrop_active)]
    pub type AirdropActive<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Id of the current (or the last) airdrop
    #[pallet::storage]
    #[pallet::getter(fn airdrop_id)]
    pub type AirdropId<T: Config> = StorageValue<_, u64>;

    /// Genesis config for this pallet
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Genesis beneficiaries
        pub beneficiaries: Vec<(T::AccountId, BalanceOf<T>)>,
        /// Genesis balance for this pallet's account
        pub genesis_balance: BalanceOf<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            use frame_support::assert_ok;

            // Create Claim account
            let account_id = <Pallet<T>>::account_id();

            // Fill account with genesis balance
            let min = T::Currency::minimum_balance();
            let _ = T::Currency::mint_into(&account_id, min.saturating_add(self.genesis_balance));

            TotalClaimable::<T>::put(BalanceOf::<T>::zero());

            // Add beneficiaries
            if !self.beneficiaries.is_empty() {
                assert_ok!(<Pallet<T>>::do_add_beneficiaries(
                    self.beneficiaries.clone().into_iter().collect()
                ));

                // Initialize other storage variables
                AirdropActive::<T>::put(true);
                AirdropId::<T>::put(0);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Beginning of a new airdrop campaing
        AirdropStarted {
            /// The id of the airdrop that has just started
            airdrop_id: u64,
        },
        /// Some amount has been claimed by the beneficiary
        Claimed {
            /// Who claimed the tokens
            beneficiary: T::AccountId,
            /// How many tokens were claimed
            amount: BalanceOf<T>,
            /// The payment id
            payment_id: <T::Paymaster as Pay>::Id,
        },
        /// Ending of the airdrop campaing
        AirdropEnded {
            /// The id of the airdrop that has just ended
            airdrop_id: u64,
        },
    }

    /// Error for the treasury pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// Attemp to start a new airdrop while there is one already in progress
        AlreadyStarted,
        /// The pot has not enough funds available to cover for all the airdropped amounts
        NotEnoughFunds,
        /// Account requested a claim but it is not present among the Beneficiaries
        NotEligible,
        /// Added a beneficiary without balance to claim
        NothingToClaim,
        /// Attempt to modify the balance of an already added beneficiary
        AlreadyPresent,
        /// There was some issue with the mechanism of payment.
        PayoutError,
        /// Attempt to perform an action implying an open airdrop, while it has already ended
        AlreadyEnded,
    }

    impl<T: Config> Pallet<T> {
        /// The account ID of the claim pot.
        ///
        /// This actually does computation. If you need to keep using it, then make sure you cache the
        /// value and only call this once.
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Return the amount of money in the pot.
        /// The existential deposit is not part of the pot so claim account never gets deleted.
        pub fn pot() -> BalanceOf<T> {
            T::Currency::reducible_balance(
                &Self::account_id(),
                Preservation::Preserve,
                Fortitude::Polite,
            )
        }

        fn do_claim(origin: T::AccountId, beneficiary: Option<T::AccountId>) -> DispatchResult {
            // See if account is eligible to get an airdrop
            Beneficiaries::<T>::try_mutate_exists(origin.clone(), |amount| {
                *amount = match amount {
                    // Account is eligible to get an airdrop
                    Some(amount) => {
                        // Determine who is the beneficiary
                        let beneficiary = beneficiary.unwrap_or(origin);
                        // Execute payment
                        let available = Self::pot();
                        if *amount > available {
                            log::warn!("Claimable amount {amount:?} bigger than total available {available:?}");
                            Err(Error::<T>::PayoutError)?; // Prevent going under the existential deposit of the account
                        }
                        let payment_id = T::Paymaster::pay(&beneficiary, (), *amount)
                            .map_err(|_| Error::<T>::PayoutError)?;
                        // Subtract the claimed token from the TotalClaimable
                        TotalClaimable::<T>::mutate(|required_amount| {
                            *required_amount = required_amount.saturating_sub(*amount)
                        });
                        log::trace!("Claimed {amount:?} for {beneficiary:?}");
                        Self::deposit_event(Event::<T>::Claimed {
                            beneficiary,
                            amount: *amount,
                            payment_id,
                        });
                        None
                    }
                    // Account is not eligible to receive funds
                    _ => Err(Error::<T>::NotEligible)?,
                };
                Ok::<_, DispatchError>(())
            })?;
            Ok(())
        }

        fn do_add_beneficiaries(
            beneficiaries: BTreeMap<T::AccountId, BalanceOf<T>>,
        ) -> DispatchResult {
            // Check that the pot has enough funds to cover for all the beneficiaries
            let available_amount = Self::pot();
            let mut required_amount = TotalClaimable::<T>::get();

            beneficiaries
                .iter()
                .try_for_each::<_, DispatchResult>(|(account, amount)| {
                    if let Some(_) = Beneficiaries::<T>::get(account.clone()) {
                        // Account already exists
                        log::warn!("Beneficiary {account:?} already added.");
                        Err(Error::<T>::AlreadyPresent)?;
                    } else if amount.is_zero() {
                        // Attempting to add a beneficiary with nothing to claim
                        log::warn!("Beneficiary {account:?} with nothing to claim.");
                        Err(Error::<T>::NothingToClaim)?;
                    } else {
                        // Account doesn't exist. Add its token amount to the required amount this pallet's account should have
                        required_amount = required_amount.saturating_add(*amount);
                        log::trace!("Added beneficiary {account:?}. Can claim: {amount:?}");
                    }

                    // Cannot cover for all the tokens, raise an error
                    if required_amount > available_amount {
                        Err(Error::<T>::NotEnoughFunds)?;
                    }

                    Beneficiaries::<T>::insert(account, amount);

                    Ok(())
                })?;

            // Update total claimable
            TotalClaimable::<T>::put(required_amount);

            Ok(())
        }

        fn check_airdrop_status() -> DispatchResult {
            if !AirdropActive::<T>::get() {
                Err(Error::<T>::AlreadyEnded)?;
            }
            Ok(())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Declare the beginning of a new aidrop and start adding beneficiaries (if specified).
        /// Raise an Error if:
        /// - There is an already active airdrop
        /// - There isn't enough balance in the pallets' account to cover for the claim of the supplied beneficiaries (if specified)
        /// This is an atomic operation. If there isn't enough balance to cover for all the beneficiaries, then none will be added.
        /// Origin must be the ManagerOrigin.
        #[pallet::call_index(0)]
        #[pallet::weight(match &beneficiaries {
            Some(beneficiaries) => T::WeightInfo::begin_airdrop_with_beneficiaries(beneficiaries.len() as u32),
            None => T::WeightInfo::begin_airdrop_empty_beneficiaries(),
        })]
        pub fn begin_airdrop(
            origin: OriginFor<T>,
            beneficiaries: Option<BTreeMap<T::AccountId, BalanceOf<T>>>,
        ) -> DispatchResultWithPostInfo {
            T::ManagerOrigin::ensure_origin(origin)?;

            // Set airdrop as active
            AirdropActive::<T>::try_mutate(|is_active| {
                if *is_active {
                    Err(Error::<T>::AlreadyStarted)?
                } else {
                    *is_active = true;
                    Ok::<_, DispatchError>(())
                }
            })?;

            // Start adding beneficiaries if specified
            if let Some(beneficiaries) = beneficiaries {
                Self::do_add_beneficiaries(beneficiaries)?;
            }

            // Increase airdrop id
            AirdropId::<T>::mutate(|id| {
                let airdrop_id = id.map_or(0, |v| v + 1);
                *id = Some(airdrop_id);
                Self::deposit_event(Event::<T>::AirdropStarted { airdrop_id });
            });

            Ok(PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::No,
            })
        }

        /// Claim token airdrop for 'origin' or 'dest' (if specified).
        /// Fails if 'origin' or 'dest' are not entitled to any airdrop.
        /// 'origin' must be signed.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn claim(
            origin: OriginFor<T>,
            dest: Option<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let origin_account = ensure_signed(origin)?;
            Self::check_airdrop_status()?;
            Self::do_claim(origin_account, dest)?;
            Ok(PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::No,
            })
        }

        /// Claim token airdrop for 'origin' or 'dest' (if specified).
        /// Fails if 'origin' or 'dest' are not entitled to any airdrop.
        /// 'origin' must be signed.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::claim_for())]
        pub fn claim_for(_origin: OriginFor<T>, dest: T::AccountId) -> DispatchResultWithPostInfo {
            Self::check_airdrop_status()?;
            Self::do_claim(dest, None)?;
            Ok(PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::No,
            })
        }

        /// Add beneficiaries.
        /// Raise an Error if:
        /// - There isn't enough balance in the pallets' account to cover for the claim of the supplied beneficiaries (if specified)
        /// - Attempt to modify the claimable amount of an already existing beneficiary
        /// This is an atomic operation. If there isn't enough balance to cover for all the beneficiaries, then none will be added.
        /// Origin must be the ManagerOrigin.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::add_beneficiaries(beneficiaries.len() as u32))]
        pub fn add_beneficiaries(
            origin: OriginFor<T>,
            beneficiaries: BTreeMap<T::AccountId, BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            Self::check_airdrop_status()?;
            T::ManagerOrigin::ensure_origin(origin)?;
            Self::do_add_beneficiaries(beneficiaries)?;

            Ok(PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::No,
            })
        }

        /// End an airdrop. Storage variables will be cleared.
        /// Any unclaimed balance will be sent to the destination specified as per 'UnclaimedDestination'.
        /// Raise an Error if attempting to end an already ended airdrop.
        /// Origin must be 'ManagerOrigin'.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::end_airdrop(Beneficiaries::<T>::iter_keys().collect::<Vec<_>>().len() as u32))]
        pub fn end_airdrop(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::ManagerOrigin::ensure_origin(origin)?;

            // Set airdrop as inactive
            AirdropActive::<T>::try_mutate(|is_active| {
                if !*is_active {
                    Err(Error::<T>::AlreadyEnded)?
                } else {
                    *is_active = false;
                    Ok::<_, DispatchError>(())
                }
            })?;

            // Remove all beneficiaries entries
            let _ = Beneficiaries::<T>::clear(u32::MAX, None);

            // Deal with any remaining balance in the pallet's account
            // TODO: Shall we want to withdraw all the balance in the pallet's account or only up to
            // TotalClaimable ?
            let unclaimed_destination = T::UnclaimedDestination::get();
            if unclaimed_destination != Self::account_id() {
                let remaining_funds = Self::pot();
                T::Currency::transfer(
                    &Self::account_id(),
                    &T::UnclaimedDestination::get(),
                    remaining_funds,
                    Preservation::Preserve,
                )?;
                log::debug!("Sending {remaining_funds:?} to specified destination");
            }

            // Set total claimable to 0
            TotalClaimable::<T>::put(BalanceOf::<T>::zero());

            // End airdrop
            Self::deposit_event(Event::<T>::AirdropEnded {
                airdrop_id: AirdropId::<T>::get().unwrap(),
            });

            Ok(PostDispatchInfo {
                actual_weight: None,
                pays_fee: Pays::No,
            })
        }
    }
}

/// TypedGet implementation to get the AccountId of the Claim pallet.
pub struct ClaimAccountId<R>(PhantomData<R>);
impl<R> sp_runtime::traits::TypedGet for ClaimAccountId<R>
where
    R: crate::Config,
{
    type Type = <R as frame_system::Config>::AccountId;
    fn get() -> Self::Type {
        <crate::Pallet<R>>::account_id()
    }
}
