// Copyright 2024, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
#![cfg(test)]

use frame_support::{
    derive_impl, parameter_types,
    traits::{fungible::HoldConsideration, LinearStoragePrice},
    weights::{constants::ParityDbWeight, Weight},
};
use frame_system::RawOrigin;
use hp_verifiers::{Verifier, VerifyError, WeightInfo};
use sp_core::{ConstU128, ConstU32};
use sp_runtime::traits::IdentityLookup;

pub use fake_pallet::FakeVerifier;

pub type Balance = u128;
pub type AccountId = u64;
pub type Origin = RawOrigin<AccountId>;

/// A on_proof_verifier fake pallet
pub mod on_proof_verified {
    pub use pallet::*;

    #[frame_support::pallet]
    #[allow(unused_imports)]
    mod pallet {
        use frame_support::pallet_prelude::*;
        use sp_core::H256;

        use hp_on_proof_verified::OnProofVerified;

        #[pallet::pallet]
        pub struct Pallet<T>(_);

        #[pallet::config]
        pub trait Config: frame_system::Config {
            type RuntimeEvent: From<Event<Self>>
                + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        }

        type AccountOf<T> = <T as frame_system::Config>::AccountId;

        #[pallet::event]
        #[pallet::generate_deposit(pub(super) fn deposit_event)]
        pub enum Event<T: Config> {
            NewProof {
                account: Option<AccountOf<T>>,
                domain_id: Option<u32>,
                value: H256,
            },
        }

        impl<A, T: Config<AccountId = A>> OnProofVerified<A> for Pallet<T> {
            fn on_proof_verified(account: Option<A>, domain_id: Option<u32>, value: H256) {
                Self::deposit_event(Event::NewProof {
                    account,
                    domain_id,
                    value,
                });
            }
        }

        pub fn new_proof_event<A, T: Config<AccountId = A>>(
            account: Option<A>,
            domain_id: Option<u32>,
            h: H256,
        ) -> Event<T> {
            Event::NewProof {
                account,
                domain_id,
                value: h,
            }
        }
    }
}

pub mod fake_pallet {
    use super::*;

    pub const PROOF_WITH_FAKE_VERSION_LOWER_BOUND: u64 = 1000;

    /// - Accept Proof iff proof == pubs and vk != 0.
    /// - If vk == 0 the vk is invalid and raise InvalidVerificationKey
    /// - If proof == 0 the proof is invalid and raise InvalidProofData
    /// - If pubs == 0 pubs are invalid raise InvalidInput
    /// - Otherwise
    ///     - proof != pubs the proof raise a VerifyError
    ///
    #[crate::verifier]
    pub struct FakeVerifier;

    impl FakeVerifier {
        pub fn malformed_vk() -> Box<<Self as Verifier>::Vk> {
            Box::new(0)
        }

        pub fn malformed_proof() -> Box<<Self as Verifier>::Proof> {
            Box::new(0)
        }

        pub fn malformed_pubs() -> Box<<Self as Verifier>::Pubs> {
            Box::new(0)
        }
    }

    impl Verifier for FakeVerifier {
        type Proof = u64;

        type Pubs = u64;

        type Vk = u64;

        fn hash_context_data() -> &'static [u8] {
            b"fake"
        }

        fn verify_proof(
            vk: &Self::Vk,
            proof: &Self::Proof,
            pubs: &Self::Pubs,
        ) -> Result<(), VerifyError> {
            match (*vk, *proof, *pubs) {
                (0, _, _) => Err(VerifyError::InvalidVerificationKey),
                (_, 0, _) => Err(VerifyError::InvalidProofData),
                (_, _, 0) => Err(VerifyError::InvalidInput),
                (_vk, proof, pubs) if proof == pubs => Ok(()),
                _ => Err(VerifyError::VerifyError),
            }
        }

        fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
            if *vk == 0 {
                Err(VerifyError::InvalidVerificationKey)
            } else {
                Ok(())
            }
        }

        fn pubs_bytes(pubs: &Self::Pubs) -> sp_std::borrow::Cow<[u8]> {
            sp_std::borrow::Cow::Owned(pubs.to_be_bytes().into())
        }

        fn verifier_version_hash(proof: &Self::Proof) -> Option<sp_core::H256> {
            match *proof {
                n if n >= PROOF_WITH_FAKE_VERSION_LOWER_BOUND => Some(sp_core::H256::from_low_u64_be(n)),
                _ => None,
            }
        }
    }
}

pub struct MockWeightInfo;
impl WeightInfo<FakeVerifier> for MockWeightInfo {
    fn submit_proof(_proof: &u64, _pubs: &u64) -> Weight {
        Weight::from_parts(1, 2)
    }

    fn submit_proof_with_vk_hash(_proof: &u64, _pubs: &u64) -> Weight {
        Weight::from_parts(3, 4)
    }

    fn register_vk(_vk: &u64) -> Weight {
        Weight::from_parts(5, 6)
    }

    fn unregister_vk() -> Weight {
        Weight::from_parts(7, 8)
    }
}

pub struct MockCommonWeightInfo;
impl crate::common::WeightInfo for MockCommonWeightInfo {
    fn disable_verifier() -> Weight {
        Weight::from_parts(1001, 1002)
    }

    fn on_verify_disabled_verifier() -> Weight {
        Weight::from_parts(1003, 1004)
    }
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        CommonVerifiersPallet: crate::common,
        FakeVerifierPallet: fake_pallet,
        OnProofVerifiedMock: on_proof_verified,
    }
);

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = frame_system::mocking::MockBlockU32<Test>;
    type AccountData = pallet_balances::AccountData<Balance>;
    type DbWeight = ParityDbWeight;
}

parameter_types! {
    pub const BaseDeposit: Balance = 1;
    pub const PerByteDeposit: Balance = 2;
    pub const HoldReasonVkRegistration: RuntimeHoldReason = RuntimeHoldReason::CommonVerifiersPallet(crate::common::HoldReason::VkRegistration);
}

impl crate::Config<FakeVerifier> for Test {
    type RuntimeEvent = RuntimeEvent;
    type OnProofVerified = OnProofVerifiedMock;
    type Ticket = HoldConsideration<
        AccountId,
        Balances,
        HoldReasonVkRegistration,
        LinearStoragePrice<BaseDeposit, PerByteDeposit, Balance>,
    >;
    type WeightInfo = MockWeightInfo;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = ();
}

impl crate::common::Config for Test {
    type CommonWeightInfo = MockCommonWeightInfo;
}

impl on_proof_verified::Config for Test {
    type RuntimeEvent = RuntimeEvent;
}
