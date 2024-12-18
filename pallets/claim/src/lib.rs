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
// #![deny(missing_docs)]
// mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod weight;
use core::marker::PhantomData;

extern crate alloc;

use alloc::collections::btree_map::BTreeMap;
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};

use frame_support::{
    dispatch::DispatchResult,
    traits::{tokens::Pay, Currency, ExistenceRequirement, Get, OnUnbalanced, WithdrawReasons},
    PalletId,
};

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

pub use pallet::*;
pub use weight::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use std::collections::BTreeSet;

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
        type Currency: Currency<Self::AccountId>;

        /// Handler for the unbalanced decrease when dealing with unclaimed assets.
        type UnclaimedDestination: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
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

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub beneficiaries: Vec<(T::AccountId, BalanceOf<T>)>,
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
            let _ = T::Currency::make_free_balance_be(
                &account_id,
                min.saturating_add(self.genesis_balance),
            );

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
        AirdropStarted { airdrop_id: u64 },
        /// Some amount has been claimed by the beneficiary
        Claimed {
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            payment_id: <T::Paymaster as Pay>::Id,
        },
        /// Ending of the airdrop campaing
        AirdropEnded { airdrop_id: u64 },
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
        /// There was some issue with the mechanism of payment.
        PayoutError,
        /// Airdrop is not in a correct status to perform a given action.
        CannotClaim,
        /// Attempt to end an already ended airdop
        AlreadyEnded,
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
        #[pallet::weight(T::WeightInfo::begin_airdrop())]
        pub fn begin_airdrop(
            origin: OriginFor<T>,
            beneficiaries: Option<BTreeMap<T::AccountId, BalanceOf<T>>>,
        ) -> DispatchResult {
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

            Ok(())
        }

        /// Claim token airdrop for 'origin' or 'dest' (if specified).
        /// Fails if 'origin' or 'dest' are not entitled to any airdrop.
        /// 'origin' must be signed.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn claim(origin: OriginFor<T>, dest: Option<T::AccountId>) -> DispatchResult {
            let origin_account = ensure_signed(origin)?;
            Self::check_airdrop_status()?;
            Self::do_claim(origin_account, dest)
        }

        /// Claim token airdrop for 'origin' or 'dest' (if specified).
        /// Fails if 'origin' or 'dest' are not entitled to any airdrop.
        /// 'origin' must be signed.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::claim_for())]
        pub fn claim_for(_origin: OriginFor<T>, dest: T::AccountId) -> DispatchResult {
            Self::check_airdrop_status()?;
            Self::do_claim(dest, None)
        }

        /// Add beneficiaries.
        /// Raise an Error if:
        /// - There isn't enough balance in the pallets' account to cover for the claim of the supplied beneficiaries (if specified)
        /// This is an atomic operation. If there isn't enough balance to cover for all the beneficiaries, then none will be added.
        /// Origin must be the ManagerOrigin.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::add_beneficiaries())]
        pub fn add_beneficiaries(
            origin: OriginFor<T>,
            beneficiaries: BTreeMap<T::AccountId, BalanceOf<T>>,
        ) -> DispatchResult {
            Self::check_airdrop_status()?;
            T::ManagerOrigin::ensure_origin(origin)?;
            Self::do_add_beneficiaries(beneficiaries)?;

            Ok(())
        }

        /// Remove beneficiaries from the storage.
        /// Origin must be the ManagerOrigin.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::remove_beneficiaries())]
        pub fn remove_beneficiaries(
            origin: OriginFor<T>,
            beneficiaries: BTreeSet<T::AccountId>,
        ) -> DispatchResult {
            Self::check_airdrop_status()?;
            T::ManagerOrigin::ensure_origin(origin)?;
            Self::do_remove_beneficiaries(beneficiaries);

            Ok(())
        }

        /// End an airdrop. Storage variables will be cleared.
        /// Any unclaimed balance will be sent to the destination specified as per 'UnclaimedDestination'.
        /// Raise an Error if attempting to end an already ended airdrop.
        /// Origin must be 'ManagerOrigin'.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::end_airdrop())]
        pub fn end_airdrop(origin: OriginFor<T>) -> DispatchResult {
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
            let unclaimed_funds = T::Currency::withdraw(
                &Self::account_id(),
                Self::pot(),
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::KeepAlive,
            )?;
            T::UnclaimedDestination::on_unbalanced(unclaimed_funds);

            // Set total claimable to 0
            TotalClaimable::<T>::put(BalanceOf::<T>::zero());

            // End airdrop
            Self::deposit_event(Event::<T>::AirdropEnded {
                airdrop_id: AirdropId::<T>::get().unwrap(),
            });

            Ok(())
        }
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
        /// The existential deposit is not part of the pot so treasury account never gets deleted.
        pub fn pot() -> BalanceOf<T> {
            T::Currency::free_balance(&Self::account_id())
                // Must never be less than 0 but better be safe.
                .saturating_sub(T::Currency::minimum_balance())
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
                        if *amount > Self::pot() {
                            Err(Error::<T>::PayoutError)?; // Prevent going under the existential deposit of the account
                        }
                        let payment_id = T::Paymaster::pay(&beneficiary, (), *amount)
                            .map_err(|_| Error::<T>::PayoutError)?;
                        // Subtract the claimed token from the TotalClaimable
                        TotalClaimable::<T>::mutate(|required_amount| {
                            *required_amount = required_amount.saturating_sub(*amount)
                        });
                        Self::deposit_event(Event::<T>::Claimed {
                            beneficiary,
                            amount: *amount,
                            payment_id,
                        });
                        None
                    }
                    // Account is not eligible to receive funds
                    None => Err(Error::<T>::NotEligible)?,
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

            beneficiaries.iter().for_each(|(account, amount)| {
                // Account already exists
                if let Some(old_amount) = Beneficiaries::<T>::get(account.clone()) {
                    // We're giving the account less token compared to before
                    if old_amount > *amount {
                        // Subtract the difference from the required balance this pallet's account should have
                        required_amount =
                            required_amount.saturating_sub(old_amount.saturating_sub(*amount));
                    } else {
                        // We're giving the account more tokens compared to before
                        // Add the difference to the required balance this pallet's account should have
                        required_amount =
                            required_amount.saturating_add(amount.saturating_sub(old_amount));
                    }
                } else {
                    // Account doesn't exist. Add its token amount to the required amount this pallet's account should have
                    required_amount = required_amount.saturating_add(*amount);
                }
            });

            // Cannot cover for all the tokens, raise an error
            if required_amount > available_amount {
                Err(Error::<T>::NotEnoughFunds)?;
            }

            // Update total claimable
            TotalClaimable::<T>::put(required_amount);

            // Add beneficiaries
            beneficiaries
                .into_iter()
                .for_each(|(account, amount)| Beneficiaries::<T>::insert(account, amount));

            Ok(())
        }

        fn do_remove_beneficiaries(beneficiaries: BTreeSet<T::AccountId>) {
            let mut required_amount = TotalClaimable::<T>::get();

            beneficiaries.into_iter().for_each(|account| {
                if let Some(amount) = Beneficiaries::<T>::take(account.clone()) {
                    required_amount -= amount;
                }
            });

            TotalClaimable::<T>::put(required_amount);
        }

        fn check_airdrop_status() -> DispatchResult {
            if !AirdropActive::<T>::get() {
                Err(Error::<T>::CannotClaim)?;
            }
            Ok(())
        }
    }
}

/// TypedGet implementation to get the AccountId of the Treasury.
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

impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for Pallet<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
        // Must resolve into existing but better to be safe.
        let _ = T::Currency::resolve_creating(&Self::account_id(), amount);
    }
}
