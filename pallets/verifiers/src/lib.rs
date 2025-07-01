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

//! This crate abstracts the implementation of a new verifier pallet.
//! ```ignore
//! use pallet_verifiers::verifier;
//! use hp_verifiers::{Verifier, VerifyError};
//! /// The following attribute generates a new verifier pallet in this crate.
//! #[verifier]
//! pub struct MyVerifier;
//!
//! /// Implement the `Verifier` trait: the verifier business logic.
//! impl Verifier for MyVerifier {
//!     type Proof = u64;
//!
//!     type Pubs = u64;
//!
//!     type Vk = u64;
//!
//!     fn hash_context_data() -> &'static [u8] {
//!         b"my"
//!     }
//!
//!     fn verify_proof(
//!         vk: &Self::Vk,
//!         proof: &Self::Proof,
//!         pubs: &Self::Pubs,
//!     ) -> Result<(), VerifyError> {
//!         (vk == proof && pubs == proof).then_some(()).ok_or(VerifyError::VerifyError)
//!     }
//!
//!     fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
//!         if *vk == 0 {
//!             Err(VerifyError::InvalidVerificationKey)
//!         } else {
//!             Ok(())
//!         }
//!     }
//!        
//!     fn pubs_bytes(pubs: &Self::Pubs) -> alloc::borrow::Cow<[u8]> {
//!         alloc::borrow::Cow::Owned(pubs.to_be_bytes().into())
//!     }
//! }
//! ```
//! Your crate should also implement a struct that implements the `hp_verifiers::WeightInfo<YourVerifierStruct>`
//! trait. This struct is used to define the weight of the verifier pallet and should map the generic
//! request in you weight implementation computed with your benchmark.

extern crate alloc;

// Workaround for a bug in `frame_support::pallet` procedural macro that generate some docs only code wrongly:
// they forget to add the where clause to the calls (and maybe in some other places).
#[cfg(not(doc))]
pub use pallet::*;

pub use pallet_verifiers_macros::*;

pub mod common;
#[allow(missing_docs)]
pub mod mock;

pub mod benchmarking_utils;
mod tests;

pub use hp_verifiers::WeightInfo;
#[frame_support::pallet]
pub mod pallet {

    // Workaround for a bug in `frame_support::pallet` procedural macro that generate some docs only code wrongly:
    // they forget to add the where clause to the calls (and maybe in some other places).
    #![cfg(not(doc))]

    use alloc::borrow::Cow;
    use alloc::boxed::Box;
    use codec::Encode;
    use core::default::Default;
    use core::fmt::Debug;
    #[cfg(feature = "runtime-benchmarks")]
    use frame_support::traits::fungible::Mutate;
    use frame_support::{
        dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::{Consideration, Footprint},
        Identity,
    };
    use frame_system::pallet_prelude::*;
    use hp_on_proof_verified::{Compose as _, OnProofVerified};
    use sp_core::{hexdisplay::AsBytesRef, H256};
    use sp_io::hashing::keccak_256;
    use sp_runtime::{traits::BadOrigin, ArithmeticError};

    use hp_verifiers::{Verifier, VerifyError, WeightInfo};

    /// The in-code storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Type alias for AccountId
    pub type AccountOf<T> = <T as frame_system::Config>::AccountId;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    /// The pallet component.
    pub struct Pallet<T, I = ()>(_);

    /// A complete Verification Key or its hash.
    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum VkOrHash<K>
    where
        K: Debug + Clone + PartialEq + Encode + Decode + TypeInfo + MaxEncodedLen,
    {
        /// The Vk hash
        Hash(H256),
        /// The Vk
        Vk(Box<K>),
    }

    impl<K> Default for VkOrHash<K>
    where
        K: Debug + Clone + PartialEq + Encode + Decode + TypeInfo + MaxEncodedLen,
    {
        fn default() -> Self {
            VkOrHash::Hash(H256::default())
        }
    }

    impl<K> VkOrHash<K>
    where
        K: Debug + Clone + PartialEq + Encode + Decode + TypeInfo + MaxEncodedLen,
    {
        /// Take a verification key and return a `VkOrHash`
        pub fn from_vk(vk: K) -> Self {
            VkOrHash::Vk(Box::new(vk))
        }

        /// Take an hash and return a `VkOrHash`
        pub fn from_hash(hash: H256) -> Self {
            VkOrHash::Hash(hash)
        }
    }

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config + crate::common::Config
    where
        I: Verifier,
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Proof verified call back
        type OnProofVerified: OnProofVerified<Self::AccountId>;
        /// A means of providing some cost while data is stored on-chain.
        type Ticket: Consideration<Self::AccountId, Footprint>;
        /// Weights
        type WeightInfo: hp_verifiers::WeightInfo<I>;
        /// Currency used in benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        type Currency: Mutate<AccountOf<Self>>;
    }

    /// A Vk with a reference count
    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub struct VkEntry<V> {
        vk: V,
        ref_count: u64,
    }

    impl<V> VkEntry<V> {
        /// Construct a new vk with reference count set to 1.
        pub fn new(vk: V) -> Self {
            Self { vk, ref_count: 1 }
        }
    }

    /// Compute the statement hash for a given vk, proof, and public data.
    pub fn compute_statement_hash<I: Verifier>(
        vk_or_hash: &VkOrHash<I::Vk>,
        proof: &I::Proof,
        pubs: &I::Pubs,
    ) -> H256 {
        let version_hash = I::verifier_version_hash(proof);
        let hash = match vk_or_hash {
            VkOrHash::Hash(h) => Cow::Borrowed(h),
            VkOrHash::Vk(vk) => Cow::Owned(I::vk_hash(vk)),
        };
        let ctx: &[u8] = I::hash_context_data();
        let vk_hash: &H256 = hash.as_ref();
        let pubs = I::pubs_bytes(pubs);

        let mut data_to_hash = keccak_256(ctx).to_vec();
        data_to_hash.extend_from_slice(vk_hash.as_bytes());
        data_to_hash.extend_from_slice(version_hash.as_bytes());
        data_to_hash.extend_from_slice(keccak_256(pubs.as_bytes_ref()).as_bytes_ref());
        H256(keccak_256(data_to_hash.as_slice()))
    }

    /// Compute the weight for the given proof, pubs, and domain.
    ///
    /// 1. Extract the vk weight both if it was provided by hash (db retrieves) or
    ///   directly (validate its weight)
    /// 2. Get the verify proof weight or replace it with the override one if present
    ///   `override_verify_proof`
    /// 3. Compute statement weight
    /// 4. Dispatch statement weight
    ///
    /// Even if this fuction takes proof, pubs, vk and domain, it should never use these values
    /// in some algorithm that doesn't have a constant cost.
    ///
    /// This function is designed to be used both in the preemptive weight computation (extrisic
    /// annotation by use `None` as `override_verify_proof` value) and to compute the `PostInfo`
    /// weight by passing the weight returned by `verify_proof()`
    ///
    pub(crate) fn submit_proof_weight<T: Config<I>, I: 'static + Verifier>(
        vk_or_hash: &VkOrHash<I::Vk>,
        proof: &I::Proof,
        pubs: &I::Pubs,
        domain_id: &Option<u32>,
        override_verify_proof: Option<Weight>,
    ) -> Weight {
        // Check the disabled state: we didn't consider any time cost about checking boolean
        // variable and proof size: we consider them negligible
        let base = T::DbWeight::get().reads(1);
        let vk_weight = match vk_or_hash {
            VkOrHash::Hash(_) => {
                // We considering unwrapping VkEntry negligible
                T::WeightInfo::get_vk()
            }
            VkOrHash::Vk(vk) => {
                // We considering cloning vk negligible
                T::WeightInfo::validate_vk(vk)
            }
        };
        // ensure_signed is just a struct unwrapping.
        let verify =
            override_verify_proof.unwrap_or_else(|| T::WeightInfo::verify_proof(proof, pubs));
        let statement = T::WeightInfo::compute_statement_hash(proof, pubs);
        base.compose(vk_weight)
            .compose(verify)
            .compose(statement)
            .compose(T::OnProofVerified::weight(domain_id))
    }

    /// Pallet specific events.
    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()>
    where
        I: Verifier,
    {
        /// The Vk has been registered.
        VkRegistered {
            /// Verification key hash
            hash: H256,
        },
        /// The Vk has been unregistered.
        VkUnregistered {
            /// Verification key hash
            hash: H256,
        },
        /// The proof has been verified.
        ProofVerified {
            /// Proof verified statement
            statement: H256,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Provided data has not valid public inputs.
        InvalidInput,
        /// Provided data has not valid proof.
        InvalidProofData,
        /// Verify proof failed.
        VerifyError,
        /// Provided an invalid verification key.
        InvalidVerificationKey,
        /// Provided an unregistered verification key hash.
        VerificationKeyNotFound,
        /// Current Verifier Pallet is disabled.
        DisabledVerifier,
        /// Verification key has already been registered.
        VerificationKeyAlreadyRegistered,
        /// The submitted proof is in an unsupported version.
        UnsupportedVersion,
    }

    impl<T, I> From<VerifyError> for Error<T, I> {
        fn from(e: VerifyError) -> Self {
            match e {
                VerifyError::InvalidInput => Error::<T, I>::InvalidInput,
                VerifyError::InvalidProofData => Error::<T, I>::InvalidProofData,
                VerifyError::VerifyError => Error::<T, I>::VerifyError,
                VerifyError::InvalidVerificationKey => Error::<T, I>::InvalidVerificationKey,
                VerifyError::UnsupportedVersion => Error::<T, I>::UnsupportedVersion,
            }
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn disabled)]
    pub type Disabled<T: Config<I>, I: 'static = ()>
    where
        I: Verifier,
    = StorageValue<_, bool>;

    #[pallet::storage]
    #[pallet::getter(fn vks)]
    pub type Vks<T: Config<I>, I: 'static = ()>
    where
        I: Verifier,
    = StorageMap<Hasher = Identity, Key = H256, Value = VkEntry<I::Vk>>;

    #[pallet::storage]
    #[pallet::getter(fn deposits)]
    pub type Tickets<T: Config<I>, I: 'static = ()>
    where
        I: Verifier,
    = StorageMap<Hasher = Blake2_128Concat, Key = (T::AccountId, H256), Value = T::Ticket>;

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I>
    where
        I: Verifier,
    {
        /// Submit a proof and accept it if and only if is valid.
        /// On success emit a `ProofVerified` event.
        /// Accept either a Vk or its hash. If you use the Vk hash the Vk should be already registered
        /// with `register_vk` extrinsic.
        #[pallet::call_index(0)]
        #[pallet::weight(
            submit_proof_weight::<T, I>(vk_or_hash, proof, pubs, domain_id, None)
        )]
        pub fn submit_proof(
            origin: OriginFor<T>,
            vk_or_hash: VkOrHash<I::Vk>,
            proof: Box<I::Proof>,
            pubs: Box<I::Pubs>,
            domain_id: Option<u32>,
        ) -> DispatchResultWithPostInfo
        where
            I: Verifier,
        {
            log::trace!("Submitting proof");
            ensure!(
                !Self::disabled().unwrap_or_default(),
                on_disable_error::<T, I>()
            );
            let vk = match &vk_or_hash {
                VkOrHash::Hash(h) => Vks::<T, I>::get(h)
                    .map(|vk_entry| vk_entry.vk)
                    .ok_or(Error::<T, I>::VerificationKeyNotFound)?,
                VkOrHash::Vk(vk) => {
                    I::validate_vk(vk).map_err(Error::<T, I>::from)?;
                    vk.as_ref().clone()
                }
            };
            let account = ensure_signed(origin).ok();
            let verify_proof_weight =
                I::verify_proof(&vk, &proof, &pubs).map_err(Error::<T, I>::from)?;
            let statement = compute_statement_hash::<I>(&vk_or_hash, &proof, &pubs);
            Self::deposit_event(Event::ProofVerified { statement });
            T::OnProofVerified::on_proof_verified(account, domain_id, statement);
            Ok(verify_proof_weight
                .map(|new_weight| {
                    submit_proof_weight::<T, I>(
                        &vk_or_hash,
                        &proof,
                        &pubs,
                        &domain_id,
                        Some(new_weight),
                    )
                })
                .into())
        }

        /// Register a new verification key.
        /// On success emit a `VkRegistered` event that contain the hash to use on `submit_proof`.
        /// Lock some funds, which can be unlocked by calling `unregister_vk`.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::register_vk(vk))]
        pub fn register_vk(origin: OriginFor<T>, vk: Box<I::Vk>) -> DispatchResultWithPostInfo {
            log::trace!("Register vk");
            ensure!(
                !Self::disabled().unwrap_or_default(),
                on_disable_error::<T, I>()
            );
            let account_id = ensure_signed(origin)?;
            let hash = I::vk_hash(&vk);
            ensure!(
                !Tickets::<T, I>::contains_key((&account_id, hash)),
                Error::<T, I>::VerificationKeyAlreadyRegistered
            );
            I::validate_vk(&vk).map_err(Error::<T, I>::from)?;
            let footprint = Footprint::from_encodable(&vk);
            let ticket = T::Ticket::new(&account_id, footprint)?;
            Tickets::<T, I>::insert((account_id, hash), ticket);
            Vks::<T, I>::mutate(hash, |vk_entry| {
                match vk_entry {
                    Some(VkEntry { ref_count, .. }) => {
                        *ref_count = ref_count
                            .checked_add(1)
                            .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;
                    }
                    None => {
                        *vk_entry = Some(VkEntry::new(*vk));
                    }
                }
                Ok::<_, DispatchError>(())
            })?;
            Self::deposit_event(Event::VkRegistered { hash });
            Ok(().into())
        }

        /// Disable verifier: both `register_vk` and `submit_proof` will return a
        /// `DisabledVerifier` Error.
        #[pallet::call_index(2)]
        #[pallet::weight(<T::CommonWeightInfo as crate::common::WeightInfo>::disable_verifier())]
        pub fn disable(origin: OriginFor<T>, disabled: bool) -> DispatchResult {
            log::trace!("Disable verifier: {disabled}");
            // Just root can disable/enable the verifier
            ensure_root(origin)?;

            Disabled::<T, I>::put(disabled);
            Ok(())
        }

        /// Unregister a previously registered verification key.
        /// Should be called by the same account used for registering the verification key.
        /// Unlock the funds which were locked when registering the verification key.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::unregister_vk())]
        pub fn unregister_vk(origin: OriginFor<T>, vk_hash: H256) -> DispatchResult {
            log::trace!("Unregister vk");
            let account_id = ensure_signed(origin)?;
            // Drop ticket if present
            if let Some(ticket) = Tickets::<T, I>::take((&account_id, vk_hash)) {
                ticket.drop(&account_id)?;
            } else if Vks::<T, I>::contains_key(vk_hash) {
                Err(BadOrigin)?
            } else {
                Err(Error::<T, I>::VerificationKeyNotFound)?
            }
            Vks::<T, I>::mutate_exists(vk_hash, |vk_entry| match vk_entry {
                Some(v) => {
                    v.ref_count = v.ref_count.saturating_sub(1);
                    if v.ref_count == 0 {
                        *vk_entry = None;
                        Self::deposit_event(Event::VkUnregistered { hash: vk_hash });
                    }
                }
                None => unreachable!(),
            });
            Ok(())
        }
    }

    pub(crate) fn on_disable_error<T: Config<I>, I: Verifier + 'static>(
    ) -> DispatchErrorWithPostInfo {
        use crate::common::WeightInfo;
        DispatchErrorWithPostInfo {
            post_info: PostDispatchInfo {
                actual_weight: Some(T::CommonWeightInfo::on_verify_disabled_verifier()),
                pays_fee: Pays::Yes,
            },
            error: Error::<T, I>::DisabledVerifier.into(),
        }
    }

    #[cfg(test)]
    mod tests {
        use core::marker::PhantomData;

        use crate::{
            mock::FakeVerifier,
            tests::registered_vk::{REGISTERED_VK, REGISTERED_VK_HASH, VALID_HASH_REGISTERED_VK},
        };

        use super::*;
        use hp_verifiers::{Verifier, NO_VERSION_HASH};
        use rstest::rstest;
        use sp_core::U256;

        struct OtherVerifier;
        impl Verifier for OtherVerifier {
            type Proof = u64;
            type Pubs = u64;
            type Vk = u64;
            fn hash_context_data() -> &'static [u8] {
                let context = b"other";
                assert_ne!(FakeVerifier::hash_context_data(), context);
                context
            }
            fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
                FakeVerifier::validate_vk(vk)
            }
            fn verify_proof(
                vk: &Self::Vk,
                proof: &Self::Proof,
                pubs: &Self::Pubs,
            ) -> Result<Option<Weight>, VerifyError> {
                FakeVerifier::verify_proof(vk, proof, pubs)
            }
            fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
                FakeVerifier::pubs_bytes(pubs)
            }
        }

        #[rstest]
        #[case::vk_and_pubs_used_in_test(
            PhantomData::<FakeVerifier>,
            0,
            42,
            VkOrHash::from_vk(REGISTERED_VK),
            VALID_HASH_REGISTERED_VK
        )]
        #[case::same_from_vk_hash(
            PhantomData::<FakeVerifier>,
            0,
            42,
            VkOrHash::from_hash(REGISTERED_VK_HASH),
            VALID_HASH_REGISTERED_VK
        )]
        #[case::hash_as_documented(
            PhantomData::<FakeVerifier>,
            24,
            42,
            VkOrHash::from_vk(REGISTERED_VK),
            {
                let mut data_to_hash = keccak_256(b"fake").to_vec();
                data_to_hash.extend_from_slice(REGISTERED_VK_HASH.as_bytes());
                data_to_hash.extend_from_slice(H256::from_low_u64_be(24).as_bytes());
                data_to_hash.extend_from_slice(&keccak_256(42_u64.to_be_bytes().as_ref()));
                H256(keccak_256(data_to_hash.as_slice()))
            }
        )]
        #[case::hash_as_documented(
            PhantomData::<FakeVerifier>,
            0,
            42,
            VkOrHash::from_vk(REGISTERED_VK),
            {
                let mut data_to_hash = keccak_256(b"fake").to_vec();
                data_to_hash.extend_from_slice(REGISTERED_VK_HASH.as_bytes());
                data_to_hash.extend_from_slice(NO_VERSION_HASH.as_bytes());
                data_to_hash.extend_from_slice(&keccak_256(42_u64.to_be_bytes().as_ref()));
                H256(keccak_256(data_to_hash.as_slice()))
            }
        )]
        #[should_panic]
        #[case::should_take_care_of_pubs(
            PhantomData::<FakeVerifier>,
            0,
            24,
            VkOrHash::from_vk(REGISTERED_VK),
            VALID_HASH_REGISTERED_VK
        )]
        #[should_panic]
        #[case::should_take_care_of_context_data(
            PhantomData::<OtherVerifier>,
            0,
            42,
            VkOrHash::from_vk(REGISTERED_VK),
            VALID_HASH_REGISTERED_VK
        )]
        #[should_panic]
        #[case::should_take_care_of_vk(
            PhantomData::<FakeVerifier>,
            0,
            42,
            VkOrHash::from_vk(24),
            VALID_HASH_REGISTERED_VK
        )]
        #[should_panic]
        #[case::should_take_care_of_verification_version(
            PhantomData::<FakeVerifier>,
            100,
            42,
            VkOrHash::from_vk(REGISTERED_VK),
            VALID_HASH_REGISTERED_VK
        )]
        fn hash_statement_as_expected<V: Verifier>(
            #[case] _verifier: PhantomData<V>,
            #[case] proof: V::Proof,
            #[case] pubs: V::Pubs,
            #[case] vk_or_hash: VkOrHash<V::Vk>,
            #[case] expected: H256,
        ) {
            let hash = compute_statement_hash::<V>(&vk_or_hash, &proof, &pubs);

            assert_eq!(hash, expected);
        }

        struct Other2Verifier;
        impl Verifier for Other2Verifier {
            type Proof = ();
            type Pubs = ();
            type Vk = U256;
            fn hash_context_data() -> &'static [u8] {
                b"more"
            }

            fn verify_proof(
                _vk: &Self::Vk,
                _proof: &Self::Proof,
                _pubs: &Self::Pubs,
            ) -> Result<Option<Weight>, VerifyError> {
                Ok(None)
            }

            fn pubs_bytes(_pubs: &Self::Pubs) -> Cow<'_, [u8]> {
                Cow::Borrowed(&[])
            }
        }

        struct VerifierWithoutHash;
        impl Verifier for VerifierWithoutHash {
            type Proof = ();
            type Pubs = ();
            type Vk = H256;

            fn vk_hash(vk: &Self::Vk) -> Self::Vk {
                *vk
            }

            fn hash_context_data() -> &'static [u8] {
                b""
            }

            fn verify_proof(
                _vk: &Self::Vk,
                _proof: &Self::Proof,
                _pubs: &Self::Pubs,
            ) -> Result<Option<Weight>, VerifyError> {
                Ok(None)
            }

            fn pubs_bytes(_pubs: &Self::Pubs) -> Cow<'_, [u8]> {
                Cow::Borrowed(&[])
            }
        }

        #[rstest]
        #[case::vk_used_in_test(PhantomData::<FakeVerifier>, REGISTERED_VK, REGISTERED_VK_HASH)]
        #[should_panic]
        #[case::u256_vk_changed(PhantomData::<Other2Verifier>, U256::from(REGISTERED_VK), REGISTERED_VK_HASH
        )]
        #[case::forward_vk(PhantomData::<VerifierWithoutHash>, REGISTERED_VK_HASH, REGISTERED_VK_HASH
        )]
        fn hash_vk_as_expected<V: Verifier>(
            #[case] _verifier: PhantomData<V>,
            #[case] vk: V::Vk,
            #[case] expected: H256,
        ) {
            let hash = V::vk_hash(&vk);

            assert_eq!(hash, expected);
        }
    }
}
