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

//! This pallet allows to perform token giveaways to selected beneficiaries with a manual claiming process.
//! Only a **ManagerOrigin** is allowed to start and end a claiming process, as well as adding the corresponding
//! beneficiaries and remove leftover ones. Only one claim can be held at a time.
//! The claiming process for beneficiaries is completely feeless, and handled via unsigned extrinsics.
//! A Beneficiary can be either a Substrate addresses (of type **T::AccountId**) or Ethereum addresses,
//! and the procedure for claiming is slightly different:
//!
//! Substrate beneficiaries need to provide a signature on a claiming message, established at the
//! time of claim start, from the same address that is loaded in the beneficiaries list. The
//! tokens will be sent to that very same address. All types of Substrate signatures are supported
//! (sr25519, ed25519, ecdsa).
//! Signature can be generated either locally (e.g. via sub-key tool) or via PolkadotJS 'Sign&Verify' tool.
//!
//! Ethereum beneficiaries need to provide a Substrate destination address on which they want the tokens
//! to be sent. In order to be allowed to do that, they need to provide a signature on a claiming message,
//! established at the time of claim start, followed by a separator (currently '\n') and a byte encoding of
//! the destination address as defined by **T::AccountIdBytesToSign**.
//!
//! When claim ends, unclaimed funds will be transferred to **T::UnclaimedDestination**, and the beneficiaries
//! list will be cleared.
//! Please note that is possible to add/remove up to **T::MaxOpBeneficiaries** at a time.
//! Larger batches require multiple, separate calls to **add_beneficiaries** and **remove_beneficiaries**.
//! It is possible to add new beneficiaries only when a claim has started (or at the moment of start) and if
//! the account associated to the pallet has enough funds to cover all of them, and remove them only when
//! a claim has ended.

#![allow(clippy::borrow_interior_mutable_const)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod beneficiary;
pub use beneficiary::{AccountId32ToSs58BytesToSign, AccountIdToBytesLiteral, Beneficiary};
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod utils;
mod weight;
use core::marker::PhantomData;

extern crate alloc;

use alloc::{collections::btree_map::BTreeMap, vec, vec::Vec};
use sp_runtime::traits::{AccountIdConversion, Zero};

use frame_support::{
    defensive,
    dispatch::DispatchResult,
    pallet_prelude::{InvalidTransaction, TransactionValidityError},
    traits::{
        fungible::{Inspect, Mutate},
        tokens::{Fortitude, Preservation},
        Get,
    },
    BoundedVec, PalletId,
};

pub use sp_core::{ecdsa::Signature as EthereumSignature, H160 as EthereumAddress};

type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

type ClaimMessage<T> = BoundedVec<u8, <T as Config>::MaxClaimMessageLength>;

impl<T: Config> From<Error<T>> for TransactionValidityError {
    fn from(error: Error<T>) -> TransactionValidityError {
        let e = match error {
            Error::AlreadyEnded => InvalidTransaction::Stale,
            Error::NotEligible => InvalidTransaction::BadSigner,
            Error::BadSignature => InvalidTransaction::BadProof,
            _ => {
                defensive!();
                InvalidTransaction::Custom(0u8)
            }
        };
        TransactionValidityError::Invalid(e)
    }
}

pub use pallet::*;
pub use weight::WeightInfo;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use crate::beneficiary::{Beneficiary, ClaimSignature};
    use codec::Encode;
    use frame_support::{pallet_prelude::*, traits::DefensiveSaturating};
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::{IdentifyAccount, Verify},
        TokenError,
    };

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The claim's pallet id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Manager allowed to begin/end claims and add/remove beneficiaries
        type ManagerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The currency type.
        type Currency: Mutate<Self::AccountId>;

        /// Destination for unclaimed assets
        type UnclaimedDestination: Get<Self::AccountId>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The signer of the message for claiming assigned tokens, for Substrate beneficiaries.
        type Signer: IdentifyAccount<AccountId = Self::AccountId> + Parameter;

        /// The type of signature to be supplied for claiming the tokens for Substrate beneiciaries
        type Signature: Verify<Signer = Self::Signer> + Parameter;

        /// Means of converting an account ID to bytes suitable to be signed on Ethereum-side
        type AccountIdBytesToSign: crate::beneficiary::AccountIdToBytesLiteral<
            Self,
            AccountId = Self::AccountId,
        >;

        /// The maximum number of allowed beneficiaries.
        #[pallet::constant]
        type MaxBeneficiaries: Get<u32>;

        /// The maximum length of the message to sign for claiming tokens
        #[pallet::constant]
        type MaxClaimMessageLength: Get<u32>;

        /// The maximum number of beneficiaries allowed to be updated within a single operation. Used to restrict extrinsic weights.
        const MAX_OP_BENEFICIARIES: u32;

        /// Helper to create a signature to be benchmarked.
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: crate::benchmarking::BenchmarkHelper<Self::Signature, Self::Signer>;
    }

    /// Candidates eligible to receive a claim with the associated balance they have right to
    #[pallet::storage]
    pub type Beneficiaries<T: Config> =
        CountedStorageMap<_, Twox64Concat, Beneficiary<T>, BalanceOf<T>>;

    /// Total tokens claimable from the current claim.  
    #[pallet::storage]
    pub type TotalClaimable<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Whether there is an active claim or not
    #[pallet::storage]
    pub type ClaimActive<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Id of the current (or the last) claim
    #[pallet::storage]
    pub type ClaimId<T: Config> = StorageValue<_, (u64, ClaimMessage<T>)>;

    /// Account id of this pallet
    #[pallet::storage]
    pub type PalletAccountId<T: Config> = StorageValue<_, T::AccountId>;

    /// Genesis config for this pallet
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Genesis beneficiaries
        pub beneficiaries: Vec<(Beneficiary<T>, BalanceOf<T>)>,
        /// Genesis claim message
        pub claim_message: ClaimMessage<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            use frame_support::assert_ok;

            // Sanity check
            assert!(T::MAX_OP_BENEFICIARIES <= T::MaxBeneficiaries::get());

            TotalClaimable::<T>::put(BalanceOf::<T>::zero());

            // Add beneficiaries
            let num_beneficiaries = self.beneficiaries.len();

            if num_beneficiaries > 0 {
                assert!(!self.claim_message.is_empty(), "InvalidClaimMessage");
                // Start adding beneficiaries if specified
                // Note: Considering it's a genesis build there is no need here
                //       to enforce a check on MaxOpBeneficiaries
                assert_ok!(<Pallet<T>>::check_max_beneficiaries(num_beneficiaries));
                assert_ok!(<Pallet<T>>::do_add_beneficiaries(
                    self.beneficiaries.clone().into_iter().collect(),
                ));

                // Initialize other storage variables
                ClaimActive::<T>::put(true);
                ClaimId::<T>::put((0, self.claim_message.clone()));
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Beginning of a new claim campaign
        ClaimStarted {
            /// The id of the claim that has just started
            claim_id: u64,
            /// The claim message for the claim that has just started
            claim_message: ClaimMessage<T>,
        },
        /// Some amount has been claimed by the beneficiary
        Claimed {
            /// Who claimed the tokens
            beneficiary: Beneficiary<T>,
            /// How many tokens were claimed
            amount: BalanceOf<T>,
        },
        /// Ending of the claim campaign
        ClaimEnded {
            /// The id of the claim that has just ended
            claim_id: u64,
            /// The claim message for the claim that has just ended
            claim_message: ClaimMessage<T>,
        },
        /// Some beneficiaries have been removed
        BeneficiariesRemoved {
            /// The number of beneficiaries remaining in storage
            remaining: u32,
        },
        /// No more beneficiaries to remove
        NoMoreBeneficiaries,
    }

    /// Error for the claim pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// Attempt to perform an action that is invalid when there is a claim campaign active.
        AlreadyStarted,
        /// Account requested a claim but it is not present among the Beneficiaries
        NotEligible,
        /// Added a beneficiary without balance to claim
        NothingToClaim,
        /// Maximum number of beneficiaries reached
        MaxNumBeneficiariesReached,
        /// Too many beneficiaries for this single operation
        TooManyBeneficiaries,
        /// Attempt to modify the balance of an already added beneficiary
        AlreadyPresent,
        /// Attempt to perform an action that is invalid when there is no claim campaign active.
        AlreadyEnded,
        /// Attempt to start a claim while there are still beneficiaries in storage from a previous one
        NonEmptyBeneficiaries,
        /// Signature verification failed for a given beneficiary
        BadSignature,
        /// Supplied an invalid claim message
        InvalidClaimMessage,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
            let account = Self::account_id();
            if T::Currency::balance(&account).is_zero() {
                // Mint existential deposit
                let _ = T::Currency::mint_into(&account, T::Currency::minimum_balance());
            }
            Weight::zero()
        }
    }

    impl<T: Config> Pallet<T> {
        /// The account ID of the claim pot.
        pub fn account_id() -> T::AccountId {
            // Check the memorized storage value.
            if let Some(id) = PalletAccountId::<T>::get() {
                return id;
            }

            // Create account if not present
            let id = T::PalletId::get().into_account_truncating();
            PalletAccountId::<T>::put(&id);
            id
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

        fn do_claim(dest: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            T::Currency::transfer(&Self::account_id(), &dest, amount, Preservation::Preserve)?;
            // Subtract the claimed token from the TotalClaimable
            TotalClaimable::<T>::mutate(|required_amount| {
                *required_amount = required_amount.defensive_saturating_sub(amount)
            });
            Ok(())
        }

        fn process_claim(
            beneficiary: Beneficiary<T>,
        ) -> core::result::Result<BalanceOf<T>, DispatchError> {
            // See if account is eligible to get a claim
            Beneficiaries::<T>::try_mutate_exists(beneficiary.clone(), |amount| {
                let mut ret_amount = BalanceOf::<T>::zero();
                *amount = match amount {
                    // Account is eligible to get a claim
                    Some(amount) => {
                        // Execute payment
                        let available = Self::pot();
                        if *amount > available {
                            log::warn!("Claimable amount {amount:?} bigger than total available {available:?}");
                            Err(TokenError::FundsUnavailable)?; // Prevent going under the existential deposit of the account
                        }
                        log::trace!("Claimed {amount:?} for {beneficiary:?}");
                        Self::deposit_event(Event::<T>::Claimed {
                            beneficiary,
                            amount: *amount,
                        });
                        ret_amount = *amount;
                        None
                    }
                    // Account is not eligible to receive funds
                    _ => Err(Error::<T>::NotEligible)?,
                };
                Ok::<_, DispatchError>(ret_amount)
            })
        }

        fn do_add_beneficiaries(
            beneficiaries: BTreeMap<Beneficiary<T>, BalanceOf<T>>,
        ) -> DispatchResult {
            // Check that the pot has enough funds to cover for all the beneficiaries
            let available_amount = Self::pot();
            let mut required_amount = TotalClaimable::<T>::get();

            beneficiaries
                .iter()
                .try_for_each::<_, DispatchResult>(|(beneficiary, amount)| {
                    if Beneficiaries::<T>::contains_key(beneficiary) {
                        // Account already exists
                        log::warn!("Beneficiary {beneficiary:?} already added.");
                        Err(Error::<T>::AlreadyPresent)?;
                    }
                    if amount.is_zero() {
                        // Attempting to add a beneficiary with nothing to claim
                        log::warn!("Beneficiary {beneficiary:?} with nothing to claim.");
                        Err(Error::<T>::NothingToClaim)?;
                    }

                    // Account doesn't exist. Add its token amount to the required amount this pallet's account should have
                    required_amount = required_amount.defensive_saturating_add(*amount);
                    log::trace!("Added beneficiary {beneficiary:?}. To claim: {amount:?}");

                    // Cannot cover for all the tokens, raise an error
                    if required_amount > available_amount {
                        Err(TokenError::FundsUnavailable)?;
                    }

                    Beneficiaries::<T>::insert(beneficiary, amount);

                    Ok(())
                })?;

            // Update total claimable
            TotalClaimable::<T>::put(required_amount);

            Ok(())
        }

        fn check_claim_status(should_be_active: bool) -> Result<(), Error<T>> {
            match (ClaimActive::<T>::get(), should_be_active) {
                (false, true) => Err(Error::<T>::AlreadyEnded),
                (true, false) => Err(Error::<T>::AlreadyStarted),
                _ => Ok(()),
            }
        }

        fn check_max_op_beneficiaries(new_beneficiaries_len: usize) -> DispatchResult {
            if new_beneficiaries_len > T::MAX_OP_BENEFICIARIES as usize {
                log::warn!(
                    "Too many beneficiaries for this single operation: {new_beneficiaries_len:?}."
                );
                Err(Error::<T>::TooManyBeneficiaries)?;
            }

            Ok(())
        }

        fn check_max_beneficiaries(new_beneficiaries_len: usize) -> DispatchResult {
            // Check we have space for all the beneficiaries we are trying to add
            let actual_beneficiaries_len = Beneficiaries::<T>::count();
            if actual_beneficiaries_len + new_beneficiaries_len as u32 > T::MaxBeneficiaries::get()
            {
                log::warn!(
                    "This operation would exceed the maximum amount of supported beneficiaries.
                    \nCurrent: {actual_beneficiaries_len}. Attempting to add: {new_beneficiaries_len:?}."
                );
                Err(Error::<T>::MaxNumBeneficiariesReached)?;
            }

            Ok(())
        }

        fn check_beneficiaries_len(new_beneficiaries_len: usize) -> DispatchResult {
            Self::check_max_op_beneficiaries(new_beneficiaries_len)?;
            Self::check_max_beneficiaries(new_beneficiaries_len)?;
            Ok(())
        }

        fn do_remove_beneficiaries() {
            // Start removing remaining beneficiaries if present

            let num_beneficiaries = Beneficiaries::<T>::count();
            if num_beneficiaries > 0 {
                let result = Beneficiaries::<T>::clear(T::MAX_OP_BENEFICIARIES, None);

                if result.maybe_cursor.is_some() {
                    Self::deposit_event(Event::<T>::BeneficiariesRemoved {
                        remaining: num_beneficiaries - result.unique,
                    });
                } else {
                    Self::deposit_event(Event::<T>::NoMoreBeneficiaries);
                }
            }
        }

        fn check_claimant(
            beneficiary: &Beneficiary<T>,
            signature: ClaimSignature<T>,
        ) -> Result<(), Error<T>> {
            // Pre-requisites
            // 1. Check claim is active
            Self::check_claim_status(true)?;

            // 2. Check beneficiary is eligible
            if !Beneficiaries::<T>::contains_key(beneficiary) {
                Err(Error::<T>::NotEligible)?;
            }

            // Check signature
            let claim_message = ClaimId::<T>::get().unwrap().1;
            if !signature.verify(claim_message.as_slice(), beneficiary) {
                Err(Error::<T>::BadSignature)?
            }

            Ok(())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Declare the beginning of a new claim and start adding beneficiaries (if specified).
        /// Raise an Error if:
        /// - There is an already active claim
        /// - The claim message is empty
        /// - An error has occurred during insertion of the beneficiaries (insufficient amount,
        ///   duplicates, pallet doesn't have enough funds)
        /// - Trying to add too many beneficiaries (more than **T::MaxOpBeneficiaries**)
        /// The add_beneficiaries operation is atomic. If one insertion fails, the whole extrinsic fails.
        /// Origin must be the ManagerOrigin.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::add_beneficiaries(u32::min(beneficiaries.len() as u32, T::MAX_OP_BENEFICIARIES))
        .saturating_add(T::WeightInfo::begin_claim())
        )]
        pub fn begin_claim(
            origin: OriginFor<T>,
            beneficiaries: BTreeMap<Beneficiary<T>, BalanceOf<T>>,
            claim_message: ClaimMessage<T>,
        ) -> DispatchResult {
            T::ManagerOrigin::ensure_origin(origin)?;

            // Sanity check: we've removed all the beneficiaries
            if Beneficiaries::<T>::count() > 0 {
                Err(Error::<T>::NonEmptyBeneficiaries)?;
            }

            if claim_message.is_empty() {
                Err(Error::<T>::InvalidClaimMessage)?;
            }

            // Set claim as active
            Self::check_claim_status(false)?;
            ClaimActive::<T>::put(true);

            let num_beneficiaries = beneficiaries.len();

            if num_beneficiaries > 0 {
                // Check that we are not adding too many here
                // Note: we don't need to check for MaxBeneficiaries considering that:
                // - When starting a claim there are no beneficiaries in storage
                // - It always holds that T::MaxOpBeneficiaries <= T::MaxBeneficiaries
                // Thus the following check, alone, it's enough
                Self::check_max_op_beneficiaries(num_beneficiaries)?;

                // Start adding beneficiaries if specified
                Self::do_add_beneficiaries(beneficiaries)?;
            }

            // Increase claim id
            // Note: Theoretically we should hold some funds from the manager account for
            // storing claim_id and claim_message.
            // However, considering that it's only one message, limited in size,
            // we avoid introducing this unnecessary complexity here.
            ClaimId::<T>::mutate(|t| {
                let new_t = t.clone().map_or((0, claim_message.clone()), |v| {
                    (v.0 + 1, claim_message.clone())
                });
                *t = Some(new_t.clone());
                Self::deposit_event(Event::<T>::ClaimStarted {
                    claim_id: new_t.0,
                    claim_message: new_t.1,
                });
            });

            Ok(())
        }

        /// Claim tokens for a 'beneficiary' with a Substrate address, provided a
        /// 'signature' on the actual claim message.
        /// 'origin' must be none.
        /// Fails if:
        /// - 'beneficiary' is not entitled to any token
        /// - The supplied 'signature' is invalid.
        /// - There is no active airdrop
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn claim(
            origin: OriginFor<T>,
            beneficiary: T::Signer,
            signature: T::Signature,
        ) -> DispatchResult {
            ensure_none(origin)?;
            let beneficiary_account = beneficiary.into_account();
            let beneficiary = Beneficiary::<T>::Substrate(beneficiary_account.clone());
            let signature = ClaimSignature::<T>::Substrate(signature);
            Self::check_claimant(&beneficiary, signature)?;
            let amount = Self::process_claim(beneficiary)?;
            Self::do_claim(beneficiary_account, amount)
        }

        /// Allows ManagerOrigin to claim tokens in place of a Substrate beneficiary 'dest' (taking care of the fees).
        /// Fails if:
        /// - 'dest' is not entitled to any token
        /// - There is no active airdrop
        /// 'origin' must be signed
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::claim_for())]
        pub fn claim_for(origin: OriginFor<T>, dest: T::AccountId) -> DispatchResult {
            T::ManagerOrigin::ensure_origin(origin)?;
            Self::check_claim_status(true)?;
            let amount = Self::process_claim(Beneficiary::<T>::Substrate(dest.clone()))?;
            Self::do_claim(dest, amount)
        }

        /// Add beneficiaries.
        /// Raise an Error if:
        /// - There is no active airdrop
        /// - There isn't enough balance in the pallets' account to cover for the tokens belonging to the supplied beneficiaries
        /// - Attempt to modify the claimable amount of an already existing beneficiary or adding a duplicate
        /// - Attempt to assign 0 tokens to a beneficiary.
        /// This is an atomic operation.
        /// Origin must be the ManagerOrigin.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::add_beneficiaries(beneficiaries.len() as u32))]
        pub fn add_beneficiaries(
            origin: OriginFor<T>,
            beneficiaries: BTreeMap<Beneficiary<T>, BalanceOf<T>>,
        ) -> DispatchResult {
            Self::check_claim_status(true)?;
            T::ManagerOrigin::ensure_origin(origin)?;

            let num_beneficiaries = beneficiaries.len();

            if num_beneficiaries > 0 {
                // Check that we are not adding too many here
                Self::check_beneficiaries_len(num_beneficiaries)?;

                // Start adding beneficiaries if specified
                Self::do_add_beneficiaries(beneficiaries)?;
            }

            Ok(())
        }

        /// End a claim. Storage variables will be cleared.
        /// Any unclaimed balance will be sent to the destination specified as per 'UnclaimedDestination'.
        /// Raise an Error if attempting to end an already ended claim.
        /// This extrinsic will attempt to remove as many beneficiaries as possible from storage.
        /// However, if there are more than **T::MaxOpBeneficiaries**, subsequent call(s) to 'remove_beneficiaries' must be made.
        /// Origin must be 'ManagerOrigin'.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::remove_beneficiaries(u32::min(Beneficiaries::<T>::count(), T::MAX_OP_BENEFICIARIES))
            .saturating_add(T::WeightInfo::end_claim())
        )]
        pub fn end_claim(origin: OriginFor<T>) -> DispatchResult {
            T::ManagerOrigin::ensure_origin(origin)?;

            // Set claim as inactive
            Self::check_claim_status(true)?;
            ClaimActive::<T>::put(false);

            // Deal with any remaining balance in the pallet's account
            let unclaimed_destination = T::UnclaimedDestination::get();
            if unclaimed_destination != Self::account_id() {
                let remaining_funds = Self::pot();
                T::Currency::transfer(
                    &Self::account_id(),
                    &unclaimed_destination,
                    remaining_funds,
                    Preservation::Preserve,
                )?;
                log::debug!("Sending {remaining_funds:?} to specified destination");
            }

            // Set total claimable to 0
            TotalClaimable::<T>::put(BalanceOf::<T>::zero());

            // End claim
            let (claim_id, claim_message) = ClaimId::<T>::get().unwrap();
            Self::deposit_event(Event::<T>::ClaimEnded {
                claim_id,
                claim_message,
            });

            // Start removing as many beneficiaries as possible
            Self::do_remove_beneficiaries();

            Ok(())
        }

        /// Remove as many beneficiaries as possible (up to **T::MaxOpBeneficiaries**) from storage.
        /// Fails if there is a claim in progress.
        /// Origin must be 'ManagerOrigin'.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::remove_beneficiaries(u32::min(Beneficiaries::<T>::count(), T::MAX_OP_BENEFICIARIES)).saturating_add(T::DbWeight::get().reads(1_u64)))]
        pub fn remove_beneficiaries(origin: OriginFor<T>) -> DispatchResult {
            T::ManagerOrigin::ensure_origin(origin)?;

            // Check claim inactive
            Self::check_claim_status(false)?;

            // Remove as many beneficiaries as possible
            Self::do_remove_beneficiaries();

            Ok(())
        }

        /// Claim tokens for a 'beneficiary' with an Ethereum address and send them to 'dest',
        ///  provided a 'signature' on the actual claim message and 'dest'.
        /// 'origin' must be none.
        /// Fails if:
        /// - 'beneficiary' is not entitled to any token
        /// - The supplied 'signature' is invalid.
        /// - There is no active airdrop
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::claim_ethereum())]
        pub fn claim_ethereum(
            origin: OriginFor<T>,
            beneficiary: EthereumAddress,
            signature: EthereumSignature,
            dest: T::AccountId,
        ) -> DispatchResult {
            ensure_none(origin)?;
            let beneficiary = Beneficiary::<T>::Ethereum(beneficiary);
            let signature = ClaimSignature::<T>::Ethereum((signature, dest.clone()));
            Self::check_claimant(&beneficiary, signature)?;
            let amount = Self::process_claim(beneficiary)?;
            Self::do_claim(dest, amount)
        }

        /// Allows ManagerOrigin to claim tokens in place of a Substrate beneficiary 'dest' (taking care of the fees).
        /// Fails if:
        /// - 'dest' is not entitled to any token
        /// - There is no active airdrop
        /// 'origin' must be signed
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::claim_ethereum_for())]
        pub fn claim_ethereum_for(
            origin: OriginFor<T>,
            beneficiary: EthereumAddress,
            dest: T::AccountId,
        ) -> DispatchResult {
            T::ManagerOrigin::ensure_origin(origin)?;
            Self::check_claim_status(true)?;
            let amount = Self::process_claim(Beneficiary::<T>::Ethereum(beneficiary))?;
            Self::do_claim(dest, amount)
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            const PRIORITY: u64 = 100;

            let provides = match call {
                Call::claim {
                    beneficiary,
                    signature,
                } => {
                    let beneficiary =
                        Beneficiary::<T>::Substrate(beneficiary.clone().into_account());
                    let signature = ClaimSignature::<T>::Substrate(signature.clone());
                    Self::check_claimant(&beneficiary, signature)?;
                    vec![("claim", ClaimId::<T>::get().unwrap(), beneficiary).encode()]
                }
                Call::claim_ethereum {
                    beneficiary,
                    signature,
                    dest,
                } => {
                    let beneficiary = Beneficiary::<T>::Ethereum(*beneficiary);
                    let signature = ClaimSignature::<T>::Ethereum((*signature, dest.clone()));
                    Self::check_claimant(&beneficiary, signature)?;
                    vec![(
                        "claim_ethereum",
                        ClaimId::<T>::get().unwrap(),
                        beneficiary,
                        dest,
                    )
                        .encode()]
                }
                _ => return Err(InvalidTransaction::Call.into()),
            };

            Ok(ValidTransaction {
                priority: PRIORITY,
                requires: vec![],
                provides,
                longevity: TransactionLongevity::MAX,
                propagate: true,
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
