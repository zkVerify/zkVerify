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
    weights::{RuntimeDbWeight, Weight},
};
use frame_system::RawOrigin;
use hp_verifiers::{Verifier, VerifyError, WeightInfo};
use sp_core::{ConstU128, ConstU32};
use sp_runtime::traits::IdentityLookup;

pub use fake_pallet::FakeVerifier;

pub type Balance = u128;
pub type AccountId = u64;
pub type Origin = RawOrigin<AccountId>;

pub const MAGIC_VK_VERIFY_PROOF_WEIGHT: u64 = 1_234_567;

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
        pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {}

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

            fn weight(domain_id: &Option<u32>) -> Weight {
                match domain_id {
                    Some(_) => Weight::from_parts(42, 24),
                    None => Default::default(),
                }
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
    use alloc::borrow::Cow;

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

    impl FakeVerifier {
        pub fn compute_dyn_verify_weight(vk: u64, proof: u64, pubs: u64) -> Option<Weight> {
            if vk == MAGIC_VK_VERIFY_PROOF_WEIGHT {
                Some(Weight::from_parts(
                    1000 * proof + 10_000 * pubs,
                    1_000_000 * pubs,
                ))
            } else {
                None
            }
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
        ) -> Result<Option<Weight>, VerifyError> {
            match (*vk, *proof, *pubs) {
                (0, _, _) => Err(VerifyError::InvalidVerificationKey),
                (_, 0, _) => Err(VerifyError::InvalidProofData),
                (_, _, 0) => Err(VerifyError::InvalidInput),
                (vk, proof, pubs) if proof == pubs => {
                    Ok(Self::compute_dyn_verify_weight(vk, proof, pubs))
                }
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

        fn pubs_bytes(pubs: &Self::Pubs) -> Cow<'_, [u8]> {
            Cow::Owned(pubs.to_be_bytes().into())
        }

        fn verifier_version_hash(proof: &Self::Proof) -> sp_core::H256 {
            match *proof {
                n if [24, 100].contains(&n) || n >= PROOF_WITH_FAKE_VERSION_LOWER_BOUND => {
                    sp_core::H256::from_low_u64_be(n)
                }
                _ => hp_verifiers::NO_VERSION_HASH,
            }
        }
    }
}

pub struct MockWeightInfo;
impl WeightInfo<FakeVerifier> for MockWeightInfo {
    fn register_vk(_vk: &u64) -> Weight {
        Weight::from_parts(5, 6)
    }

    fn unregister_vk() -> Weight {
        Weight::from_parts(7, 8)
    }

    fn verify_proof(
        proof: &<FakeVerifier as Verifier>::Proof,
        pubs: &<FakeVerifier as Verifier>::Pubs,
    ) -> Weight {
        Weight::from_parts(10_000_000_000 * proof + 1_000_000_000_000 * pubs, 0)
    }

    fn get_vk() -> Weight {
        Weight::from_parts(100, 10)
    }

    fn validate_vk(vk: &<FakeVerifier as Verifier>::Vk) -> Weight {
        Weight::from_parts(1_000_000 * vk, 0)
    }

    fn compute_statement_hash(
        proof: &<FakeVerifier as Verifier>::Proof,
        pubs: &<FakeVerifier as Verifier>::Pubs,
    ) -> Weight {
        Weight::from_parts(
            100_000_000_000_000 * proof + 1_000_000_000_000_000 * pubs,
            0,
        )
    }
}

pub struct MockCommonWeightInfo;
impl crate::common::WeightInfo for MockCommonWeightInfo {
    fn disable_verifier() -> Weight {
        Weight::from_parts(101, 102)
    }

    fn on_verify_disabled_verifier() -> Weight {
        Weight::from_parts(103, 104)
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

parameter_types! {
    /// ParityDB can be enabled with a feature flag, but is still experimental. These weights
    /// are available for brave runtime engineers who may want to try this out as default.
    pub const MockDbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 1_000,
        write: 100_000,
    };
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = frame_system::mocking::MockBlockU32<Test>;
    type AccountData = pallet_balances::AccountData<Balance>;
    type DbWeight = MockDbWeight;
}

parameter_types! {
    pub const BaseDeposit: Balance = 1;
    pub const PerByteDeposit: Balance = 2;
    pub const HoldReasonVkRegistration: RuntimeHoldReason = RuntimeHoldReason::CommonVerifiersPallet(crate::common::HoldReason::VkRegistration);
}

impl crate::Config<FakeVerifier> for Test {
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
    type DoneSlashHandler = ();
}

impl crate::common::Config for Test {
    type CommonWeightInfo = MockCommonWeightInfo;
}

impl on_proof_verified::Config for Test {}
