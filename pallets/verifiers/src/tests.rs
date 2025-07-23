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

use super::*;
use crate::mock::*;
use codec::Encode;
use frame_support::dispatch::{GetDispatchInfo, Pays};
use frame_support::{assert_err, assert_err_ignore_postinfo};
use frame_support::{assert_noop, assert_ok};
use hp_verifiers::{Verifier, WeightInfo};
use rstest::{fixture, rstest};
use sp_core::H256;
use sp_runtime::{BuildStorage, DispatchError};

type Vk = <FakeVerifier as Verifier>::Vk;
type RError = Error<Test, FakeVerifier>;
type VkOrHash = super::VkOrHash<Vk>;
type DisableStorage = Disabled<Test, FakeVerifier>;

pub const USER_1: AccountId = 42;
pub const USER_2: AccountId = 24;
pub static USERS: [(AccountId, Balance); 2] = [(USER_1, 42_000_000_000), (USER_2, 24_000_000_000)];

#[fixture]
pub fn test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: USERS.to_vec(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::from(t);

    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

pub mod registered_vk {
    use hex_literal::hex;

    use super::*;

    pub const REGISTERED_VK: Vk = 4325;
    pub const REGISTERED_VK_HASH: H256 = H256(hex!(
        "7aeb79b96627dd87eac158bec5612ddb7f350513a179d9ab0daf4ab5788c3262"
    ));
    pub const VALID_HASH_REGISTERED_VK: H256 = H256(hex!(
        "eebb2de92f24f7eff7613db9a0e98ba9f1d71bc1ed861e78452c33c22debc6a9"
    ));

    /// Provide an environment with a registered vk
    #[fixture]
    pub fn def_vk(mut test_ext: sp_io::TestExternalities) -> sp_io::TestExternalities {
        test_ext.execute_with(|| {
            FakeVerifierPallet::register_vk(RuntimeOrigin::signed(USER_1), Box::new(REGISTERED_VK))
                .unwrap();
            System::reset_events();
        });
        test_ext
    }
}

fn reserved_balance(vk: &Vk) -> Balance {
    BaseDeposit::get() + PerByteDeposit::get() * vk.encoded_size() as Balance
}

mod register_should {
    use hex_literal::hex;
    use registered_vk::*;

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
                RuntimeOrigin::signed(USER_1),
                Box::new(vk)
            ));

            System::assert_last_event(
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
        assert_eq!(info.call_weight, MockWeightInfo::register_vk(&43));
    }

    #[rstest]
    fn hold_a_deposit(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            let initial_reserved_balance = Balances::reserved_balance(USER_1);
            let vk = 42;
            assert_ok!(FakeVerifierPallet::register_vk(
                RuntimeOrigin::signed(USER_1),
                Box::new(vk)
            ));
            assert_eq!(
                Balances::reserved_balance(USER_1),
                initial_reserved_balance + reserved_balance(&vk)
            );
        })
    }

    #[rstest]
    fn fail_if_insufficient_free_balance(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            assert_noop!(
                FakeVerifierPallet::register_vk(RuntimeOrigin::signed(1), Box::new(42)),
                DispatchError::Token(sp_runtime::TokenError::FundsUnavailable)
            );
        })
    }

    #[rstest]
    fn not_be_allowed_for_root(mut test_ext: sp_io::TestExternalities) {
        test_ext.execute_with(|| {
            assert_noop!(
                FakeVerifierPallet::register_vk(RuntimeOrigin::root(), Box::new(42)),
                DispatchError::BadOrigin
            );
        })
    }

    #[rstest]
    fn handle_double_registration_by_different_users(mut def_vk: sp_io::TestExternalities) {
        def_vk.execute_with(|| {
            let initial_reserved_balance = Balances::reserved_balance(USER_2);
            assert_ok!(FakeVerifierPallet::register_vk(
                RuntimeOrigin::signed(USER_2),
                Box::new(REGISTERED_VK)
            ));
            System::assert_last_event(
                Event::VkRegistered {
                    hash: REGISTERED_VK_HASH,
                }
                .into(),
            );
            assert_eq!(
                Balances::reserved_balance(USER_2),
                initial_reserved_balance + reserved_balance(&REGISTERED_VK)
            );
        })
    }

    #[rstest]
    fn fail_for_double_registration_by_same_user(mut def_vk: sp_io::TestExternalities) {
        def_vk.execute_with(|| {
            assert_noop!(
                FakeVerifierPallet::register_vk(
                    RuntimeOrigin::signed(USER_1),
                    Box::new(REGISTERED_VK)
                ),
                RError::VerificationKeyAlreadyRegistered
            );
        })
    }
}

mod unregister_should {
    use super::*;
    use registered_vk::*;

    #[rstest]
    fn unregister_a_previously_registered_vk(mut def_vk: sp_io::TestExternalities) {
        def_vk.execute_with(|| {
            assert!(FakeVerifierPallet::vks(REGISTERED_VK_HASH).is_some());
            FakeVerifierPallet::unregister_vk(RuntimeOrigin::signed(USER_1), REGISTERED_VK_HASH)
                .unwrap();
            assert!(FakeVerifierPallet::vks(REGISTERED_VK_HASH).is_none());
        })
    }

    #[rstest]
    fn keep_previously_registered_vk_around_if_another_user_is_referencing_it(
        mut def_vk: sp_io::TestExternalities,
    ) {
        def_vk.execute_with(|| {
            FakeVerifierPallet::register_vk(RuntimeOrigin::signed(USER_2), Box::new(REGISTERED_VK))
                .unwrap();
            FakeVerifierPallet::unregister_vk(RuntimeOrigin::signed(USER_1), REGISTERED_VK_HASH)
                .unwrap();
            assert!(FakeVerifierPallet::vks(REGISTERED_VK_HASH).is_some());
        })
    }

    #[rstest]
    fn release_deposit(mut def_vk: sp_io::TestExternalities) {
        def_vk.execute_with(|| {
            let initial_reserved_balance = Balances::reserved_balance(USER_1);
            FakeVerifierPallet::unregister_vk(RuntimeOrigin::signed(USER_1), REGISTERED_VK_HASH)
                .unwrap();
            assert_eq!(
                Balances::reserved_balance(USER_1),
                initial_reserved_balance - reserved_balance(&REGISTERED_VK)
            )
        })
    }

    #[rstest]
    fn emit_vk_unregistered_event_if_vk_is_dropped(mut def_vk: sp_io::TestExternalities) {
        def_vk.execute_with(|| {
            FakeVerifierPallet::unregister_vk(RuntimeOrigin::signed(USER_1), REGISTERED_VK_HASH)
                .unwrap();
            System::assert_last_event(
                Event::VkUnregistered {
                    hash: REGISTERED_VK_HASH,
                }
                .into(),
            );
        })
    }

    #[rstest]
    fn emit_no_vk_unregistered_event_if_vk_is_not_dropped(mut def_vk: sp_io::TestExternalities) {
        def_vk.execute_with(|| {
            FakeVerifierPallet::register_vk(RuntimeOrigin::signed(USER_2), Box::new(REGISTERED_VK))
                .unwrap();
            FakeVerifierPallet::unregister_vk(RuntimeOrigin::signed(USER_1), REGISTERED_VK_HASH)
                .unwrap();
            assert!(System::events().into_iter().all(|e| {
                !matches!(e.event.clone().try_into(), Ok(Event::VkUnregistered { .. }))
            }));
        })
    }

    mod fail {
        use super::*;
        use frame_support::assert_noop;

        #[rstest]
        fn on_root_origin(mut test_ext: sp_io::TestExternalities) {
            test_ext.execute_with(|| {
                assert_noop!(
                    FakeVerifierPallet::unregister_vk(RuntimeOrigin::root(), REGISTERED_VK_HASH),
                    DispatchError::BadOrigin
                );
            })
        }

        #[rstest]
        fn if_vk_exists_but_caller_did_not_register_it(mut def_vk: sp_io::TestExternalities) {
            def_vk.execute_with(|| {
                assert_noop!(
                    FakeVerifierPallet::unregister_vk(
                        RuntimeOrigin::signed(USER_2),
                        REGISTERED_VK_HASH
                    ),
                    DispatchError::BadOrigin
                );
            })
        }

        #[rstest]
        fn on_nonexistent_vk(mut def_vk: sp_io::TestExternalities) {
            def_vk.execute_with(|| {
                assert_noop!(
                    FakeVerifierPallet::unregister_vk(
                        RuntimeOrigin::signed(USER_1),
                        H256::from_low_u64_be(42)
                    ),
                    RError::VerificationKeyNotFound
                );
            })
        }
    }
}

mod submit_proof_should {
    use super::*;
    use frame_support::weights::Weight;
    use hex_literal::hex;
    use registered_vk::*;

    #[rstest]
    #[case::vk(42, VkOrHash::Vk(Box::new(REGISTERED_VK)), VALID_HASH_REGISTERED_VK)]
    #[case::use_registered_vk(42, VkOrHash::Hash(REGISTERED_VK_HASH), VALID_HASH_REGISTERED_VK)]
    #[case::use_version(fake_pallet::PROOF_WITH_FAKE_VERSION_LOWER_BOUND + 42, VkOrHash::Vk(Box::new(REGISTERED_VK)), H256(hex!(
        "02975e6536d4cb401ba480feffe98e1e579169bfa7e67e65b0f73494b92206d3"
    )))]
    fn validate_proof_and_notify_execution_when(
        mut def_vk: sp_io::TestExternalities,
        #[case] proof_and_pubs: u64,
        #[case] vk_or_hash: VkOrHash,
        #[case] expected_hash: H256,
    ) {
        use on_proof_verified::new_proof_event;

        def_vk.execute_with(|| {
            // Dispatch a signed extrinsic.
            assert_ok!(FakeVerifierPallet::submit_proof(
                RuntimeOrigin::signed(42),
                vk_or_hash,
                Box::new(proof_and_pubs),
                Box::new(proof_and_pubs),
                Some(666),
            ));

            assert!(!System::events().is_empty());

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

            assert!(!System::events().is_empty());

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

            assert!(!System::events().is_empty());

            System::assert_last_event(
                new_proof_event(None, Some(1), VALID_HASH_REGISTERED_VK).into(),
            );
        });
    }

    #[test]
    fn use_submit_proof_weight_to_compute_the_weight() {
        let vk_or_hash = VkOrHash::from_vk(24);
        let expected_weight =
            crate::submit_proof_weight::<Test, FakeVerifier>(&vk_or_hash, &42, &24, &None, None);

        let info = Call::<Test, FakeVerifier>::submit_proof {
            vk_or_hash,
            proof: Box::new(42),
            pubs: Box::new(24),
            domain_id: None,
        }
        .get_dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(info.call_weight, expected_weight);
    }

    #[test]
    fn return_a_corrected_weight_if_verify_proof_return_it() {
        test_ext().execute_with(|| {
            // Dispatch a signed extrinsic.
            let vk = VkOrHash::Vk(Box::new(MAGIC_VK_VERIFY_PROOF_WEIGHT));
            let proof = Box::new(42);
            let pubs = Box::new(42);
            let post_dispatch_info = FakeVerifierPallet::submit_proof(
                RuntimeOrigin::signed(42),
                vk.clone(),
                proof.clone(),
                pubs.clone(),
                None,
            )
            .unwrap();
            assert!(
                post_dispatch_info.actual_weight.is_some(),
                "Should return a weight"
            );
            assert!(
                matches!(post_dispatch_info.pays_fee, Pays::Yes),
                "Should always pay"
            );
            let w = post_dispatch_info.actual_weight.unwrap();
            assert_eq!(
                w,
                submit_proof_weight::<Test, FakeVerifier>(
                    &vk,
                    &proof,
                    &pubs,
                    &None,
                    FakeVerifier::compute_dyn_verify_weight(
                        MAGIC_VK_VERIFY_PROOF_WEIGHT,
                        *proof,
                        *pubs
                    )
                )
            );

            // Sanity checks :
            // 1. compute_dyn_verify_weight return not None in this case
            // 2. If verify_proof return Ok(None) dispatch info should have none weight

            assert!(
                FakeVerifier::compute_dyn_verify_weight(
                    MAGIC_VK_VERIFY_PROOF_WEIGHT,
                    *proof,
                    *pubs
                )
                .is_some(),
                "Fake verifier should return a valid weight for MAGIC_VK_VERIFY_PROOF_WEIGHT"
            );

            assert!(
                FakeVerifierPallet::submit_proof(
                    RuntimeOrigin::signed(42),
                    VkOrHash::Vk(Box::new(12)),
                    proof.clone(),
                    pubs.clone(),
                    None,
                )
                .unwrap()
                .actual_weight
                .is_none(),
                "Unexpected weight"
            );
        })
    }

    #[rstest]
    #[case::no_domain(
        VkOrHash::from_vk(24),
        5,
        6,
        None,
        None,
        Weight::from_parts(6506050024001000, 0)
    )]
    #[case::no_domain(
        VkOrHash::from_hash(Default::default()),
        5,
        6,
        None,
        None,
        Weight::from_parts(6506050000001100, 10)
    )]
    #[case::no_domain(
        VkOrHash::from_vk(24),
        12,
        24,
        None,
        None,
        Weight::from_parts(25224120024001000, 0)
    )]
    #[case::domain(
        VkOrHash::from_vk(24),
        5,
        6,
        Some(12),
        None,
        Weight::from_parts(6506050024001042, 24)
    )]
    #[case::domain(
        VkOrHash::from_hash(Default::default()),
        5,
        6,
        Some(12),
        None,
        Weight::from_parts(6506050000001142, 24)
    )]
    #[case::domain(
        VkOrHash::from_vk(24),
        12,
        24,
        Some(12),
        None,
        Weight::from_parts(25224120024001042, 24)
    )]
    #[case::override_weight(
        VkOrHash::from_vk(24),
        12,
        24,
        Some(12),
        Some(Weight::from_parts(97_000_000_000_000_000, 1_000_000_000_000)),
        Weight::from_parts(122200000024001042, 1_000_000_000_000)
    )]
    fn submit_proof_expected_weights(
        #[case] vk_or_hash: VkOrHash,
        #[case] proof: u64,
        #[case] pubs: u64,
        #[case] domain_id: Option<u32>,
        #[case] override_weight: Option<Weight>,
        #[case] expected: Weight,
    ) {
        let weight = crate::submit_proof_weight::<Test, FakeVerifier>(
            &vk_or_hash,
            &proof,
            &pubs,
            &domain_id,
            override_weight,
        );

        assert_eq!(expected, weight);
    }

    mod reject {
        use super::*;

        #[rstest]
        fn not_signed_user(mut def_vk: sp_io::TestExternalities) {
            def_vk.execute_with(|| {
                // Dispatch a signed extrinsic.
                assert_noop!(
                    FakeVerifierPallet::submit_proof(
                        RuntimeOrigin::none(),
                        VkOrHash::Vk(Box::new(REGISTERED_VK)),
                        Box::new(42),
                        Box::new(42),
                        Some(1),
                    ),
                    DispatchError::BadOrigin
                );
            });
        }

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
                RuntimeOrigin::signed(USER_1),
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
