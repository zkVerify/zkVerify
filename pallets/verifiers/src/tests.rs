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

use frame_support::dispatch::{GetDispatchInfo, Pays};
use frame_support::{assert_noop, assert_ok};

use hp_verifiers::{Verifier, WeightInfo};
use sp_core::H256;

use super::*;
use crate::mock::*;
use frame_support::{assert_err, assert_err_ignore_postinfo};
use rstest::{fixture, rstest};

type Vk = <FakeVerifier as Verifier>::Vk;
type RError = Error<Test, FakeVerifier>;
type VkOrHash = super::VkOrHash<Vk>;

#[fixture]
pub fn test_ext() -> sp_io::TestExternalities {
    crate::mock::test_ext()
}

type DisableStorage = Disabled<Test, FakeVerifier>;

mod register_should {
    use hex_literal::hex;

    use super::*;

    #[rstest]
    #[case(42, H256(hex!("ee55bf17be166383be3ca3ff9d91bc5f3400bb658843fe52e62f5ceb16b5f101")))]
    #[case(24, H256(hex!("0e570c1367b641384abf443b67b3de101c1f6ed3b7d41113772866dfc15f38f9")))]
    fn accept_valid_vk(
        mut test_ext: sp_io::TestExternalities,
        #[case] vk: Vk,
        #[case] expected_hash: H256,
    ) {
        test_ext.execute_with(|| {
            assert_ok!(FakeVerifierPallet::register_vk(
                RuntimeOrigin::signed(1),
                Box::new(vk)
            ));

            mock::System::assert_last_event(
                Event::VkRegistered {
                    hash: expected_hash,
                }
                .into(),
            );
        });
    }

    #[rstest]
    fn reject_invalid_vk(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            // Dispatch a signed extrinsic.
            assert_noop!(
                FakeVerifierPallet::register_vk(
                    RuntimeOrigin::signed(1),
                    FakeVerifier::malformed_vk()
                ),
                RError::InvalidVerificationKey
            );
        });
    }

    #[rstest]
    fn reject_valid_vk_if_disabled(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            DisableStorage::set(Some(true));
            assert!(
                FakeVerifierPallet::register_vk(RuntimeOrigin::signed(1), Box::new(42),).is_err(),
            );
        });
    }

    #[test]
    fn use_the_configured_weights() {
        let info = Call::<Test, FakeVerifier>::register_vk { vk: Box::new(42) }.get_dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(info.weight, MockWeightInfo::register_vk(&43));
    }
}

pub mod submit_proof_should {
    use hex_literal::hex;

    use super::*;

    pub const REGISTERED_VK: Vk = 4325;
    pub const REGISTERED_VK_HASH: H256 = H256(hex!(
        "7aeb79b96627dd87eac158bec5612ddb7f350513a179d9ab0daf4ab5788c3262"
    ));
    pub const VALID_HASH_REGISTERED_VK: H256 = H256(hex!(
        "a65dc57cd8f1e436aaa8a8a473005040a4594f5411e0d9c7c5d7f20630217b79"
    ));

    /// Provide an environment with a registered vk (use used the default vk)
    #[fixture]
    fn def_vk(mut test_ext: sp_io::TestExternalities) -> sp_io::TestExternalities {
        test_ext.execute_with(|| {
            FakeVerifierPallet::register_vk(RuntimeOrigin::signed(1), Box::new(REGISTERED_VK))
                .unwrap();
            System::reset_events();
        });
        test_ext
    }

    #[rstest]
    #[case::vk(VkOrHash::Vk(Box::new(REGISTERED_VK)), VALID_HASH_REGISTERED_VK)]
    #[case::use_registered_vk(VkOrHash::Hash(REGISTERED_VK_HASH), VALID_HASH_REGISTERED_VK)]
    fn validate_proof_and_notify_execution_when(
        mut def_vk: sp_io::TestExternalities,
        #[case] vk_or_hash: VkOrHash,
        #[case] expected_hash: H256,
    ) {
        use on_proof_verified::new_proof_event;

        def_vk.execute_with(|| {
            // Dispatch a signed extrinsic.
            assert_ok!(FakeVerifierPallet::submit_proof(
                RuntimeOrigin::signed(42),
                vk_or_hash,
                Box::new(42),
                Box::new(42),
                Some(666),
            ));

            assert!(System::events().len() >= 1);

            System::assert_last_event(new_proof_event(Some(42), Some(666), expected_hash).into());
        });
    }

    #[rstest]
    fn emit_proof_verified_event(mut def_vk: sp_io::TestExternalities) {
        def_vk.execute_with(|| {
            // Dispatch a signed extrinsic.
            assert_ok!(FakeVerifierPallet::submit_proof(
                RuntimeOrigin::root(),
                VkOrHash::Vk(Box::new(REGISTERED_VK)),
                Box::new(42),
                Box::new(42),
                Some(1),
            ));

            assert!(System::events().len() >= 1);

            System::assert_has_event(
                Event::<Test, FakeVerifier>::ProofVerified {
                    statement: VALID_HASH_REGISTERED_VK,
                }
                .into(),
            );
        });
    }

    #[rstest]
    fn forward_no_account_if_is_root(mut def_vk: sp_io::TestExternalities) {
        use on_proof_verified::new_proof_event;

        def_vk.execute_with(|| {
            // Dispatch a signed extrinsic.
            assert_ok!(FakeVerifierPallet::submit_proof(
                RuntimeOrigin::root(),
                VkOrHash::Vk(Box::new(REGISTERED_VK)),
                Box::new(42),
                Box::new(42),
                Some(1),
            ));

            assert!(System::events().len() >= 1);

            System::assert_last_event(
                new_proof_event(None, Some(1), VALID_HASH_REGISTERED_VK).into(),
            );
        });
    }

    #[rstest]
    #[case::submit_proof(VkOrHash::from_vk(24), MockWeightInfo::submit_proof(&5, &6))]
    #[case::submit_proof_with_vk_hash(
        VkOrHash::from_hash(REGISTERED_VK_HASH),
        MockWeightInfo::submit_proof_with_vk_hash(&12, &75)
    )]
    fn use_the_configured_weights(
        #[case] vk_or_hash: VkOrHash,
        #[case] weight: frame_support::weights::Weight,
    ) {
        let info = Call::<Test, FakeVerifier>::submit_proof {
            vk_or_hash,
            proof: Box::new(42),
            pubs: Box::new(24),
            domain_id: None,
        }
        .get_dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(info.weight, weight);
    }

    mod reject {
        use super::*;

        #[rstest]
        fn valid_proof_if_disabled(mut test_ext: sp_io::TestExternalities) {
            test_ext.execute_with(|| {
                DisableStorage::set(Some(true));
                // Dispatch a signed valid proof.
                assert!(FakeVerifierPallet::submit_proof(
                    RuntimeOrigin::signed(1),
                    VkOrHash::from_vk(32),
                    Box::new(42),
                    Box::new(42),
                    None,
                )
                .is_err());
            });
        }

        #[rstest]
        fn invalid_proof(mut test_ext: sp_io::TestExternalities) {
            test_ext.execute_with(|| {
                // Dispatch a signed extrinsic.
                assert_noop!(
                    FakeVerifierPallet::submit_proof(
                        RuntimeOrigin::signed(1),
                        VkOrHash::from_vk(32),
                        Box::new(42),
                        Box::new(24),
                        None,
                    ),
                    RError::VerifyError
                );
            });
        }

        #[rstest]
        fn proof_if_request_to_use_an_unregisterd_vk(mut test_ext: sp_io::TestExternalities) {
            test_ext.execute_with(|| {
                assert_noop!(
                    FakeVerifierPallet::submit_proof(
                        RuntimeOrigin::signed(1),
                        VkOrHash::Hash(H256(hex!(
                            "ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000"
                        ))),
                        Box::new(42),
                        Box::new(42),
                        None,
                    ),
                    RError::VerificationKeyNotFound
                );
            });
        }

        #[rstest]
        fn malformed_proof(mut test_ext: sp_io::TestExternalities) {
            test_ext.execute_with(|| {
                assert_noop!(
                    FakeVerifierPallet::submit_proof(
                        RuntimeOrigin::signed(1),
                        VkOrHash::from_vk(32),
                        FakeVerifier::malformed_proof(),
                        Box::new(42),
                        None,
                    ),
                    RError::InvalidProofData
                );
            });
        }

        #[rstest]
        fn malformed_vk(mut test_ext: sp_io::TestExternalities) {
            test_ext.execute_with(|| {
                assert_noop!(
                    FakeVerifierPallet::submit_proof(
                        RuntimeOrigin::signed(1),
                        VkOrHash::from_vk(*FakeVerifier::malformed_vk()),
                        Box::new(42),
                        Box::new(42),
                        None,
                    ),
                    RError::InvalidVerificationKey
                );
            });
        }

        #[rstest]
        fn malformed_pubs(mut test_ext: sp_io::TestExternalities) {
            test_ext.execute_with(|| {
                assert_noop!(
                    FakeVerifierPallet::submit_proof(
                        RuntimeOrigin::signed(1),
                        VkOrHash::from_vk(42),
                        Box::new(42),
                        FakeVerifier::malformed_pubs(),
                        None,
                    ),
                    RError::InvalidInput
                );
            });
        }
    }
}

#[cfg(test)]
mod disable_should {
    use common::WeightInfo;
    // use frame_support::assert_err_with_weight;

    use super::*;

    #[rstest]
    fn set_the_correct_state(
        mut test_ext: sp_io::TestExternalities,
        #[values(true, false)] value: bool,
    ) {
        test_ext.execute_with(|| {
            assert_eq!(FakeVerifierPallet::disabled(), None);

            FakeVerifierPallet::disable(RuntimeOrigin::root(), value).unwrap();
            assert_eq!(FakeVerifierPallet::disabled(), Some(value));
        });
    }

    #[rstest]
    fn disable_execution(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            assert_eq!(FakeVerifierPallet::disabled(), None);

            FakeVerifierPallet::disable(RuntimeOrigin::root(), true).unwrap();
            // Dispatch a signed valid proof.
            assert_err_ignore_postinfo!(
                FakeVerifierPallet::submit_proof(
                    RuntimeOrigin::signed(1),
                    VkOrHash::from_vk(32),
                    Box::new(42),
                    Box::new(42),
                    None,
                ),
                RError::DisabledVerifier
            );
        });
    }

    #[rstest]
    fn disable_register_vk(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            assert_eq!(FakeVerifierPallet::disabled(), None);

            FakeVerifierPallet::disable(RuntimeOrigin::root(), true).unwrap();

            assert_err_ignore_postinfo!(
                FakeVerifierPallet::register_vk(RuntimeOrigin::signed(1), 42.into(),),
                RError::DisabledVerifier
            );
        });
    }

    #[rstest]
    fn disable_execution_pay_the_correct_weight(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            assert_eq!(FakeVerifierPallet::disabled(), None);

            FakeVerifierPallet::disable(RuntimeOrigin::root(), true).unwrap();

            // I cannot use `assert_err_with_weight` here because it doesn't work with
            // try-runtime feature,
            assert_err!(
                FakeVerifierPallet::submit_proof(
                    RuntimeOrigin::signed(1),
                    VkOrHash::from_vk(32),
                    Box::new(42),
                    Box::new(42),
                    None,
                ),
                on_disable_error::<Test, FakeVerifier>(),
            );
            assert_err!(
                FakeVerifierPallet::register_vk(RuntimeOrigin::signed(1), 42.into(),),
                on_disable_error::<Test, FakeVerifier>(),
            );
            assert_eq!(
                on_disable_error::<Test, FakeVerifier>()
                    .post_info
                    .actual_weight,
                Some(MockCommonWeightInfo::on_verify_disabled_verifier())
            );
        });
    }

    #[rstest]
    fn enable_a_disabled_execution(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            DisableStorage::set(Some(true));

            FakeVerifierPallet::disable(RuntimeOrigin::root(), false).unwrap();
            // Dispatch a signed valid proof.
            assert_ok!(FakeVerifierPallet::submit_proof(
                RuntimeOrigin::signed(1),
                VkOrHash::from_vk(32),
                Box::new(42),
                Box::new(42),
                None,
            ));
            assert_ok!(FakeVerifierPallet::register_vk(
                RuntimeOrigin::signed(1),
                42.into(),
            ));
        });
    }

    #[rstest]
    fn be_rejected_if_no_root(
        mut test_ext: sp_io::TestExternalities,
        #[values(true, false)] value: bool,
    ) {
        test_ext.execute_with(|| {
            assert_noop!(
                FakeVerifierPallet::disable(RuntimeOrigin::signed(1), value),
                sp_runtime::traits::BadOrigin
            );
        });
    }
}
