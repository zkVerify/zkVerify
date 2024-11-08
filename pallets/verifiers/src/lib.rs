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

//! This crate abstract the implementation of a new verifier pallet.
//! ```ignore
//! use pallet_verifiers::verifier;
//! use hp_verifiers::{Verifier, VerifyError};
//! /// Follow attribute generate a new verifier pallet in this crate.
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
//!     fn pubs_bytes(pubs: &Self::Pubs) -> sp_std::borrow::Cow<[u8]> {
//!         sp_std::borrow::Cow::Owned(pubs.to_be_bytes().into())
//!     }
//! }
//! ```
//! Your crate should also implement a struct that implement `hp_verifiers::WeightInfo<YourVerifierStruct>`
//! trait. This struct is used to define the weight of the verifier pallet and should map the generic
//! request in you weight implementation computed with your benchmark.

// Workaround for a bug in `frame_support::pallet` procedural macro that generate some docs only code wrongly:
// they forget to add the where clause to the calls (and maybe in some other places).
#[cfg(not(doc))]
pub use pallet::*;

pub use pallet_verifiers_macros::*;

pub mod common;
#[allow(missing_docs)]
pub mod mock;
mod tests;

pub use hp_verifiers::WeightInfo;
#[frame_support::pallet]
pub mod pallet {

    // Workaround for a bug in `frame_support::pallet` procedural macro that generate some docs only code wrongly:
    // they forget to add the where clause to the calls (and maybe in some other places).
    #![cfg(not(doc))]

    use codec::Encode;
    use frame_support::{
        dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, PostDispatchInfo},
        pallet_prelude::*,
        Identity,
    };
    use frame_system::pallet_prelude::*;
    use hp_on_proof_verified::OnProofVerified;
    use sp_core::{hexdisplay::AsBytesRef, H256};
    use sp_io::hashing::keccak_256;
    use sp_std::boxed::Box;

    use hp_verifiers::{Verifier, VerifyError, WeightInfo};

    #[pallet::pallet]
    /// The pallet component.
    pub struct Pallet<T, I = ()>(_);

    /// A complete Verification Key or its hash.
    #[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum VkOrHash<K>
    where
        K: sp_std::fmt::Debug + Clone + PartialEq + Encode + Decode + TypeInfo + MaxEncodedLen,
    {
        /// The Vk hash
        Hash(H256),
        /// The Vk
        Vk(Box<K>),
    }

    impl<K> VkOrHash<K>
    where
        K: sp_std::fmt::Debug + Clone + PartialEq + Encode + Decode + TypeInfo + MaxEncodedLen,
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
        /// Weights
        type WeightInfo: hp_verifiers::WeightInfo<I>;
    }

    fn statement_hash(ctx: &[u8], vk_hash: &H256, pubs: &[u8]) -> H256 {
        let mut data_to_hash = keccak_256(ctx).to_vec();
        data_to_hash.extend_from_slice(vk_hash.as_bytes());
        data_to_hash.extend_from_slice(keccak_256(pubs).as_bytes_ref());
        H256(keccak_256(data_to_hash.as_slice()))
    }

    fn compute_hash<I: Verifier>(pubs: &I::Pubs, vk_or_hash: &VkOrHash<I::Vk>) -> H256 {
        let hash = match vk_or_hash {
            VkOrHash::Hash(h) => sp_std::borrow::Cow::Borrowed(h),
            VkOrHash::Vk(vk) => sp_std::borrow::Cow::Owned(I::vk_hash(vk)),
        };
        statement_hash(
            I::hash_context_data(),
            hash.as_ref(),
            I::pubs_bytes(pubs).as_ref(),
        )
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
    }

    impl<T, I> From<VerifyError> for Error<T, I> {
        fn from(e: VerifyError) -> Self {
            match e {
                VerifyError::InvalidInput => Error::<T, I>::InvalidInput,
                VerifyError::InvalidProofData => Error::<T, I>::InvalidProofData,
                VerifyError::VerifyError => Error::<T, I>::VerifyError,
                VerifyError::InvalidVerificationKey => Error::<T, I>::InvalidVerificationKey,
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
    = StorageMap<Hasher = Identity, Key = H256, Value = I::Vk>;

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I>
    where
        I: Verifier,
    {
        /// Submit a proof and accept it if and only if is valid.
        /// On success emit a `poe::NewElement` event.
        /// Accept either a Vk or its hash. If you use the Vk hash the Vk should be already registered
        /// with `register_vk` extrinsic.
        #[pallet::call_index(0)]
        #[pallet::weight(match &vk_or_hash {
                VkOrHash::Vk(_) => T::WeightInfo::submit_proof(proof, pubs),
                VkOrHash::Hash(_) => T::WeightInfo::submit_proof_with_vk_hash(proof, pubs),
            })]
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
                VkOrHash::Hash(h) => {
                    Vks::<T, I>::get(h).ok_or(Error::<T, I>::VerificationKeyNotFound)?
                }
                VkOrHash::Vk(vk) => {
                    I::validate_vk(vk).map_err(Error::<T, I>::from)?;
                    vk.as_ref().clone()
                }
            };
            let account = ensure_signed(origin).ok();
            let statement = compute_hash::<I>(&pubs, &vk_or_hash);
            I::verify_proof(&vk, &proof, &pubs)
                .inspect(|_| Self::deposit_event(Event::ProofVerified { statement }))
                .map(|_x| T::OnProofVerified::on_proof_verified(account, domain_id, statement))
                .map_err(Error::<T, I>::from)?;
            Ok(().into())
        }

        /// Register a new verification key.
        /// On success emit a `VkRegistered` event that contain the hash to use on `submit_proof`.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::register_vk(vk))]
        pub fn register_vk(_origin: OriginFor<T>, vk: Box<I::Vk>) -> DispatchResultWithPostInfo {
            log::trace!("Register vk");
            ensure!(
                !Self::disabled().unwrap_or_default(),
                on_disable_error::<T, I>()
            );
            I::validate_vk(&vk).map_err(Error::<T, I>::from)?;
            let hash = I::vk_hash(&vk);
            Vks::<T, I>::insert(hash, vk);
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
    }

    pub(crate) fn on_disable_error<T: Config<I>, I: Verifier>() -> DispatchErrorWithPostInfo {
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
            tests::submit_proof_should::{
                REGISTERED_VK, REGISTERED_VK_HASH, VALID_HASH_REGISTERED_VK,
            },
        };

        use super::*;
        use hp_verifiers::Verifier;
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
            ) -> Result<(), VerifyError> {
                FakeVerifier::verify_proof(vk, proof, pubs)
            }
            fn pubs_bytes(pubs: &Self::Pubs) -> sp_std::borrow::Cow<[u8]> {
                FakeVerifier::pubs_bytes(pubs)
            }
        }

        #[rstest]
        #[case::vk_and_pubs_used_in_test(
            PhantomData::<FakeVerifier>,
            42,
            VkOrHash::from_vk(REGISTERED_VK),
            VALID_HASH_REGISTERED_VK
        )]
        #[case::same_from_vk_hash(
            PhantomData::<FakeVerifier>,
            42,
            VkOrHash::from_hash(REGISTERED_VK_HASH),
            VALID_HASH_REGISTERED_VK
        )]
        #[case::hash_as_documented(
            PhantomData::<FakeVerifier>,
            42,
            VkOrHash::from_vk(REGISTERED_VK),
            {
                let mut data_to_hash = keccak_256(b"fake").to_vec();
                data_to_hash.extend_from_slice(REGISTERED_VK_HASH.as_bytes());
                data_to_hash.extend_from_slice(&keccak_256(42_u64.to_be_bytes().as_ref()));
                H256(keccak_256(data_to_hash.as_slice()))
            }
        )]
        #[should_panic]
        #[case::should_take_care_of_pubs(
            PhantomData::<FakeVerifier>,
            24,
            VkOrHash::from_vk(REGISTERED_VK),
            VALID_HASH_REGISTERED_VK
        )]
        #[should_panic]
        #[case::should_take_care_of_context_data(
            PhantomData::<OtherVerifier>,
            42,
            VkOrHash::from_vk(REGISTERED_VK),
            VALID_HASH_REGISTERED_VK
        )]
        #[should_panic]
        #[case::should_take_care_of_vk(
            PhantomData::<FakeVerifier>,
            42,
            VkOrHash::from_vk(24),
            VALID_HASH_REGISTERED_VK
        )]
        fn hash_statement_as_expected<V: Verifier>(
            #[case] _verifier: PhantomData<V>,
            #[case] pubs: V::Pubs,
            #[case] vk_or_hash: VkOrHash<V::Vk>,
            #[case] expected: H256,
        ) {
            let hash = compute_hash::<V>(&pubs, &vk_or_hash);

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
            ) -> Result<(), VerifyError> {
                Ok(())
            }

            fn pubs_bytes(_pubs: &Self::Pubs) -> hp_verifiers::Cow<[u8]> {
                hp_verifiers::Cow::Borrowed(&[])
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
            ) -> Result<(), VerifyError> {
                Ok(())
            }

            fn pubs_bytes(_pubs: &Self::Pubs) -> hp_verifiers::Cow<[u8]> {
                hp_verifiers::Cow::Borrowed(&[])
            }
        }

        #[rstest]
        #[case::vk_used_in_test(PhantomData::<FakeVerifier>, REGISTERED_VK, REGISTERED_VK_HASH)]
        #[should_panic]
        #[case::u256_vk_changed(PhantomData::<Other2Verifier>, U256::from(REGISTERED_VK), REGISTERED_VK_HASH)]
        #[case::forward_vk(PhantomData::<VerifierWithoutHash>, REGISTERED_VK_HASH, REGISTERED_VK_HASH)]
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
