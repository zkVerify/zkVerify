use crate::mock::RuntimeEvent as TestEvent;
use crate::mock::*;
use crate::*;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::{EventRecord, Phase};
use sp_core::TypedGet;
use sp_runtime::{traits::BadOrigin, RuntimeAppPublic, TokenError};

pub fn assert_evt(event: Event<Test>, context: &str) {
    assert_evt_gen(true, event, context);
}

pub fn assert_not_evt(event: Event<Test>, context: &str) {
    assert_evt_gen(false, event, context);
}

fn assert_evt_gen(contains: bool, event: Event<Test>, context: &str) {
    let message = match contains {
        true => format!("{context} - CANNOT FIND {event:?}"),
        false => format!("{context} - FOUND {event:?}"),
    };
    assert_eq!(
        contains,
        mock::System::events().contains(&EventRecord {
            phase: Phase::Initialization,
            event: TestEvent::Claim(event),
            topics: vec![],
        }),
        "{message}"
    )
}

#[test]
fn genesis_default_build() {
    test().execute_with(|| {
        assert!(Beneficiaries::<Test>::iter().next().is_none());
        assert_eq!(TotalClaimable::<Test>::get(), BalanceOf::<Test>::zero());
        assert!(!ClaimActive::<Test>::get());
        assert!(ClaimId::<Test>::get().is_none());
        assert_eq!(
            Balances::free_balance(Claim::account_id()),
            EXISTENTIAL_DEPOSIT
        );
        assert_eq!(Claim::pot(), 0);
    })
}

#[test]
#[should_panic(expected = "FundsUnavailable")]
fn genesis_build_insufficient_balance() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Insufficient,
    )
    .execute_with(|| {});
}

#[test]
fn genesis_build_sufficient_balance() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_eq!(
            Beneficiaries::<Test>::iter().collect::<BTreeMap<_, _>>(),
            GENESIS_BENEFICIARIES_MAP.clone()
        );
        assert_eq!(TotalClaimable::<Test>::get(), SUFFICIENT_GENESIS_BALANCE);
        assert!(ClaimActive::<Test>::get());
        assert_eq!(
            ClaimId::<Test>::get(),
            Some((0, INIT_CLAIM_MESSAGE.clone()))
        );
        assert_eq!(
            Balances::free_balance(Claim::account_id()),
            SUFFICIENT_GENESIS_BALANCE + EXISTENTIAL_DEPOSIT
        );
        assert_eq!(Claim::pot(), SUFFICIENT_GENESIS_BALANCE);
    });
}

#[test]
fn genesis_build_exceed_op_beneficiaries_is_ok() {
    test_genesis_with_beneficiaries(MaxBeneficiaries::get()).execute_with(|| {});
}

#[test]
#[should_panic(expected = "MaxNumBeneficiariesReached")]
fn genesis_build_exceed_max_beneficiaries_fails() {
    test_genesis_with_beneficiaries(MaxBeneficiaries::get() + 1).execute_with(|| {});
}

#[test]
#[should_panic(expected = "InvalidClaimMessage")]
fn genesis_build_with_beneficiaries_empty_claim_message() {
    test_genesis_empty_claim_message(1).execute_with(|| {});
}

#[test]
fn account_id_as_expected() {
    test().execute_with(|| {
        assert_eq!(Claim::account_id(), ClaimAccountId::<Test>::get());
    });
}

#[test]
fn new_claim() {
    test().execute_with(|| {
        assert_ok!(Claim::begin_claim(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone(),
            INIT_CLAIM_MESSAGE.clone(),
        ));
        assert_evt(
            Event::ClaimStarted {
                claim_id: 0,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "New claim",
        );
        assert!(Beneficiaries::<Test>::iter().next().is_none());
        assert_eq!(TotalClaimable::<Test>::get(), BalanceOf::<Test>::zero());
        assert!(ClaimActive::<Test>::get());
        assert_eq!(
            ClaimId::<Test>::get().unwrap(),
            (0, INIT_CLAIM_MESSAGE.clone())
        );
        assert_eq!(
            Balances::free_balance(Claim::account_id()),
            EXISTENTIAL_DEPOSIT
        );
        assert_eq!(Claim::pot(), 0);
    })
}

#[test]
fn new_claim_wrong_origin() {
    test().execute_with(|| {
        assert_err!(
            Claim::begin_claim(
                Origin::Signed(USER_1).into(),
                EMPTY_BENEFICIARIES_MAP.clone(),
                INIT_CLAIM_MESSAGE.clone()
            ),
            BadOrigin
        );
        assert_not_evt(
            Event::ClaimStarted {
                claim_id: 0,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "No new claim",
        );
    })
}

#[test]
fn new_claim_sufficient_funds() {
    test_with_configs(
        WithGenesisBeneficiaries::No,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::begin_claim(
            Origin::Signed(MANAGER_USER).into(),
            GENESIS_BENEFICIARIES_MAP.clone(),
            INIT_CLAIM_MESSAGE.clone()
        ));
        assert_evt(
            Event::ClaimStarted {
                claim_id: 0,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "New claim",
        );
        assert_eq!(
            Beneficiaries::<Test>::iter().collect::<BTreeMap<_, _>>(),
            GENESIS_BENEFICIARIES_MAP.clone()
        );
        assert_eq!(TotalClaimable::<Test>::get(), SUFFICIENT_GENESIS_BALANCE);
        assert!(ClaimActive::<Test>::get());
        assert_eq!(
            ClaimId::<Test>::get(),
            Some((0, INIT_CLAIM_MESSAGE.clone()))
        );
    })
}

#[test]
fn new_claim_insufficient_funds() {
    test_with_configs(
        WithGenesisBeneficiaries::No,
        GenesisClaimBalance::Insufficient,
    )
    .execute_with(|| {
        assert_noop!(
            Claim::begin_claim(
                Origin::Signed(MANAGER_USER).into(),
                GENESIS_BENEFICIARIES_MAP.clone(),
                INIT_CLAIM_MESSAGE.clone()
            ),
            TokenError::FundsUnavailable
        );
    })
}

#[test]
fn new_claim_adding_too_many_op_beneficiaries() {
    test_with_configs(
        WithGenesisBeneficiaries::No,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_noop!(
            Claim::begin_claim(
                Origin::Signed(MANAGER_USER).into(),
                utils::get_beneficiaries_map::<Test>(MaxOpBeneficiaries::get() + 1).0,
                INIT_CLAIM_MESSAGE.clone()
            ),
            Error::<Test>::TooManyBeneficiaries
        );
    })
}

#[test]
fn new_claim_empty_message() {
    test_with_configs(
        WithGenesisBeneficiaries::No,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_noop!(
            Claim::begin_claim(
                Origin::Signed(MANAGER_USER).into(),
                utils::get_beneficiaries_map::<Test>(MaxOpBeneficiaries::get() + 1).0,
                EMPTY_CLAIM_MESSAGE.clone()
            ),
            Error::<Test>::InvalidClaimMessage
        );
    })
}

#[test]
fn cannot_start_new_claim_if_one_already_in_progress() {
    test().execute_with(|| {
        assert_ok!(Claim::begin_claim(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone(),
            INIT_CLAIM_MESSAGE.clone()
        ));
        assert_evt(
            Event::ClaimStarted {
                claim_id: 0,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "New claim",
        );
        assert_err!(
            Claim::begin_claim(
                Origin::Signed(MANAGER_USER).into(),
                GENESIS_BENEFICIARIES_MAP.clone(),
                INIT_CLAIM_MESSAGE.clone()
            ),
            Error::<Test>::AlreadyStarted
        );
        assert_not_evt(
            Event::ClaimStarted {
                claim_id: 1,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "No new claim",
        );
    })
}

#[test]
fn claim() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, user_signature, _, _) = USER_1_SIGN.clone();
        assert_ok!(Claim::claim(
            Origin::None.into(),
            user_signer,
            user_signature
        ));
        assert_evt(
            Event::Claimed {
                beneficiary: USER_1,
                amount: USER_1_AMOUNT,
            },
            "Successfull claim",
        );
        assert_eq!(Claim::pot(), SUFFICIENT_GENESIS_BALANCE - USER_1_AMOUNT);
        assert_eq!(
            TotalClaimable::<Test>::get(),
            SUFFICIENT_GENESIS_BALANCE - USER_1_AMOUNT
        );
        assert_eq!(Balances::free_balance(USER_1), USER_1_AMOUNT);
        assert!(Beneficiaries::<Test>::get(USER_1).is_none());
    });
}

#[test]
fn claim_prefixed() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, _, user_signature, _) = USER_1_SIGN.clone();
        assert_ok!(Claim::claim(
            Origin::None.into(),
            user_signer,
            user_signature
        ));
    });
}

#[test]
fn claim_eth_prefixed() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, _, _, user_signature) = USER_1_SIGN.clone();
        assert_ok!(Claim::claim(
            Origin::None.into(),
            user_signer,
            user_signature
        ));
    });
}

#[test]
fn double_claim_is_err() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, user_signature, _, _) = USER_1_SIGN.clone();
        assert_ok!(Claim::claim(
            Origin::None.into(),
            user_signer.clone(),
            user_signature.clone()
        ));
        assert_noop!(
            Claim::claim(Origin::None.into(), user_signer, user_signature),
            Error::<Test>::NotEligible
        );
    });
}

#[test]
fn claim_non_existing_beneficiary() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, user_signature, _, _) = NON_BENEFICIARY_SIGN.clone();
        assert_noop!(
            Claim::claim(Origin::None.into(), user_signer, user_signature),
            Error::<Test>::NotEligible
        );
    });
}

#[test]
fn claim_invalid_signature() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, mut user_signature, _, _) = USER_1_SIGN.clone();
        user_signature.1[0] += 1u8; // Alter signature

        assert_noop!(
            Claim::claim(Origin::None.into(), user_signer, user_signature),
            Error::<Test>::BadSignature
        );
    });
}

#[test]
fn reject_double_prefixed_message() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let user_signer = sp_runtime::testing::UintAuthorityId::from(USER_1);
        let claim_message = INIT_CLAIM_MESSAGE.clone();
        let double_wrapped_message = [
            crate::MSG_PREFIX,
            crate::MSG_PREFIX,
            claim_message.as_slice(),
            crate::MSG_SUFFIX,
            crate::MSG_SUFFIX,
        ]
        .concat();
        let user_signature = user_signer
            .sign(&double_wrapped_message.as_slice())
            .unwrap();

        assert_noop!(
            Claim::claim(Origin::None.into(), user_signer, user_signature),
            Error::<Test>::BadSignature
        );
    });
}

#[test]
fn reject_double_eth_prefixed_message() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let user_signer = sp_runtime::testing::UintAuthorityId::from(USER_1);
        let claim_message = INIT_CLAIM_MESSAGE.clone();
        let double_wrapped_message_eth = [
            crate::ETH_PREFIX,
            crate::ETH_PREFIX,
            claim_message.as_slice(),
        ]
        .concat();
        let user_signature = user_signer
            .sign(&double_wrapped_message_eth.as_slice())
            .unwrap();

        assert_noop!(
            Claim::claim(Origin::None.into(), user_signer, user_signature),
            Error::<Test>::BadSignature
        );
    });
}

#[test]
fn claim_invalid_beneficiary() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (_, user_1_signature, _, _) = USER_1_SIGN.clone();
        let user_2_signer = sp_runtime::testing::UintAuthorityId::from(USER_2);

        assert_noop!(
            Claim::claim(Origin::None.into(), user_2_signer, user_1_signature),
            Error::<Test>::BadSignature
        );
    });
}

#[test]
fn claim_insufficient_balance() {
    // Should never happen
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, user_signature, _, _) = USER_1_SIGN.clone();
        Beneficiaries::<Test>::insert(USER_1, Balances::total_issuance()); // Increase astronomically
        assert_err!(
            Claim::claim(Origin::None.into(), user_signer, user_signature),
            TokenError::FundsUnavailable
        );
        assert_not_evt(
            Event::Claimed {
                beneficiary: USER_1,
                amount: Balances::total_issuance(),
            },
            "Cannot claim if money not available",
        );
    })
}

#[test]
fn cannot_claim_while_claim_inactive() {
    test().execute_with(|| {
        let (user_signer, user_signature, _, _) = USER_1_SIGN.clone();
        assert_noop!(
            Claim::claim(Origin::None.into(), user_signer, user_signature),
            Error::<Test>::AlreadyEnded
        );
    })
}

#[test]
fn cannot_claim_if_signed_origin() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, user_signature, _, _) = USER_1_SIGN.clone();
        assert_noop!(
            Claim::claim(Origin::Signed(USER_1).into(), user_signer, user_signature),
            BadOrigin
        );
    })
}

#[test]
fn claim_for() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::claim_for(
            Origin::Signed(MANAGER_USER).into(),
            USER_1
        ));
        assert_evt(
            Event::Claimed {
                beneficiary: USER_1,
                amount: USER_1_AMOUNT,
            },
            "Successfull claim for another beneficiary",
        );
        assert_eq!(Claim::pot(), SUFFICIENT_GENESIS_BALANCE - USER_1_AMOUNT);
        assert_eq!(
            TotalClaimable::<Test>::get(),
            SUFFICIENT_GENESIS_BALANCE - USER_1_AMOUNT
        );
        assert_eq!(Balances::free_balance(USER_1), USER_1_AMOUNT);
        assert!(Beneficiaries::<Test>::get(USER_1).is_none());
    });
}

#[test]
fn claim_for_insufficient_balance() {
    // Should never happen
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        Beneficiaries::<Test>::insert(NON_BENEFICIARY, SUFFICIENT_GENESIS_BALANCE + 1);
        assert_err!(
            Claim::claim_for(Origin::Signed(MANAGER_USER).into(), NON_BENEFICIARY),
            TokenError::FundsUnavailable
        );
        assert_not_evt(
            Event::Claimed {
                beneficiary: NON_BENEFICIARY,
                amount: SUFFICIENT_GENESIS_BALANCE + 1,
            },
            "Cannot claim for other if money not available",
        );
    })
}

#[test]
fn claim_for_wrong_beneficiary() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_noop!(
            Claim::claim_for(Origin::Signed(MANAGER_USER).into(), NON_BENEFICIARY),
            Error::<Test>::NotEligible
        );
    });
}

#[test]
fn cannot_claim_for_while_claim_inactive() {
    test().execute_with(|| {
        assert_err!(
            Claim::claim_for(Origin::Signed(MANAGER_USER).into(), USER_1),
            Error::<Test>::AlreadyEnded
        );
    })
}

#[test]
fn add_beneficiaries_wrong_origin() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_err!(
            Claim::add_beneficiaries(Origin::Signed(USER_1).into(), NEW_BENEFICIARIES_MAP.clone()),
            BadOrigin
        );
    })
}

#[test]
fn cannot_add_beneficiaries_while_claim_inactive() {
    test().execute_with(|| {
        assert_err!(
            Claim::add_beneficiaries(
                Origin::Signed(MANAGER_USER).into(),
                NEW_BENEFICIARIES_MAP.clone()
            ),
            Error::<Test>::AlreadyEnded
        );
    })
}

#[test]
fn cannot_add_too_many_op_beneficiaries() {
    test_with_configs(
        WithGenesisBeneficiaries::No,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::begin_claim(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone(),
            INIT_CLAIM_MESSAGE.clone()
        ));
        assert_noop!(
            Claim::add_beneficiaries(
                Origin::Signed(MANAGER_USER).into(),
                utils::get_beneficiaries_map::<Test>(MaxOpBeneficiaries::get() + 1).0
            ),
            Error::<Test>::TooManyBeneficiaries
        );
    })
}

#[test]
fn cannot_add_beneficiaries_over_the_max() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_noop!(
            Claim::add_beneficiaries(
                Origin::Signed(MANAGER_USER).into(),
                utils::get_beneficiaries_map::<Test>(
                    MaxBeneficiaries::get() - GENESIS_BENEFICIARIES.len() as u32 + 1
                )
                .0
            ),
            Error::<Test>::MaxNumBeneficiariesReached
        );
    })
}

#[test]
fn add_beneficiaries_sufficient_funds() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let _ = Balances::mint_into(&Claim::account_id(), NEW_SUFFICIENT_BALANCE).unwrap();
        assert_ok!(Claim::add_beneficiaries(
            Origin::Signed(MANAGER_USER).into(),
            NEW_BENEFICIARIES_MAP.clone()
        ));
        assert_eq!(
            Claim::pot(),
            SUFFICIENT_GENESIS_BALANCE + NEW_SUFFICIENT_BALANCE
        );
        assert_eq!(
            TotalClaimable::<Test>::get(),
            SUFFICIENT_GENESIS_BALANCE + NEW_SUFFICIENT_BALANCE
        );
        assert_eq!(
            Beneficiaries::<Test>::iter().collect::<BTreeMap<_, _>>(),
            GENESIS_BENEFICIARIES_MAP
                .clone()
                .into_iter()
                .chain(NEW_BENEFICIARIES_MAP.clone().into_iter())
                .collect::<BTreeMap<_, _>>()
        );
    })
}

#[test]
fn add_beneficiaries_insufficient_funds() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        // Just add enough funds to cover for first insertions but not all
        let _ = Balances::mint_into(&Claim::account_id(), USER_4_AMOUNT + USER_5_AMOUNT).unwrap();

        assert_noop!(
            Claim::add_beneficiaries(
                Origin::Signed(MANAGER_USER).into(),
                NEW_BENEFICIARIES_MAP.clone()
            ),
            TokenError::FundsUnavailable
        );
    })
}

#[test]
fn cannot_add_already_existing_beneficiary() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_err!(
            Claim::add_beneficiaries(
                Origin::Signed(MANAGER_USER).into(),
                GENESIS_BENEFICIARIES_MAP.clone()
            ),
            Error::<Test>::AlreadyPresent
        );
    })
}

#[test]
fn remove_beneficiaries_one_shot() {
    let mut e = test();

    // Add MaxOpBeneficiaries + 1
    e.execute_with(|| {
        utils::get_beneficiaries_map::<Test>(MaxOpBeneficiaries::get())
            .0
            .into_iter()
            .for_each(|account| Beneficiaries::<Test>::insert(account.0, account.1));
    });
    e.commit_all().unwrap();

    e.execute_with(|| {
        // First remove call should succeed but be insufficient
        assert_ok!(Claim::remove_beneficiaries(
            Origin::Signed(MANAGER_USER).into()
        ));
        assert_evt(Event::NoMoreBeneficiaries, "No more beneficiaries");
        assert_eq!(Beneficiaries::<Test>::count(), 0);
    });
}

#[test]
fn remove_beneficiaries() {
    let mut e = test();

    // Add MaxOpBeneficiaries + 1
    e.execute_with(|| {
        utils::get_beneficiaries_map::<Test>(MaxOpBeneficiaries::get() + 1)
            .0
            .into_iter()
            .for_each(|account| Beneficiaries::<Test>::insert(account.0, account.1));
    });
    e.commit_all().unwrap();

    e.execute_with(|| {
        // First remove call should succeed but be insufficient
        assert_ok!(Claim::remove_beneficiaries(
            Origin::Signed(MANAGER_USER).into()
        ));
        let remaining = MaxBeneficiaries::get() - MaxOpBeneficiaries::get();
        assert_evt(
            Event::BeneficiariesRemoved { remaining },
            "Beneficiaries removed",
        );
        assert_eq!(Beneficiaries::<Test>::count(), remaining);
    });
}

#[test]
fn cannot_remove_beneficiaries_if_claim_in_progress() {
    test().execute_with(|| {
        Claim::begin_claim(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone(),
            INIT_CLAIM_MESSAGE.clone(),
        )
        .unwrap();
        assert_noop!(
            Claim::remove_beneficiaries(Origin::Signed(MANAGER_USER).into()),
            Error::<Test>::AlreadyStarted
        );
    })
}

#[test]
fn remove_beneficiaries_bad_origin() {
    test().execute_with(|| {
        assert_noop!(
            Claim::remove_beneficiaries(Origin::Signed(USER_1).into()),
            BadOrigin
        );
    })
}

#[test]
fn end_claim() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        // Give other balance. Now Self::pot() == SUFFICIENT_GENESIS_BALANCE * 2
        let _ = Balances::mint_into(&Claim::account_id(), SUFFICIENT_GENESIS_BALANCE).unwrap();
        assert_ok!(Claim::end_claim(Origin::Signed(MANAGER_USER).into()));
        assert_evt(
            Event::ClaimEnded {
                claim_id: 0,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "Claim finished",
        );

        assert!(!ClaimActive::<Test>::get());
        assert!(Beneficiaries::<Test>::iter().next().is_none());
        assert_eq!(Claim::pot(), 0);
        assert_eq!(
            Balances::free_balance(Claim::account_id()),
            EXISTENTIAL_DEPOSIT
        );
        assert_eq!(TotalClaimable::<Test>::get(), 0);
        assert_eq!(
            Balances::reducible_balance(
                &UnclaimedDestinationMockAccount::get(),
                Preservation::Expendable,
                Fortitude::Polite,
            ),
            SUFFICIENT_GENESIS_BALANCE * 2
        );
    });
}

#[test]
fn end_claim_wrong_origin() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_err!(Claim::end_claim(Origin::Signed(USER_1).into()), BadOrigin);
        assert_not_evt(
            Event::ClaimEnded {
                claim_id: 0,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "No end claim",
        );
    })
}

#[test]
fn double_end_claim() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::end_claim(Origin::Signed(MANAGER_USER).into()));
        assert_err!(
            Claim::end_claim(Origin::Signed(MANAGER_USER).into()),
            Error::<Test>::AlreadyEnded
        );
    });
}

#[test]
fn end_claim_new_claim() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::end_claim(Origin::Signed(MANAGER_USER).into()));

        assert_ok!(Claim::begin_claim(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone(),
            INIT_CLAIM_MESSAGE.clone()
        ));
        assert_evt(
            Event::ClaimStarted {
                claim_id: 1,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "New claim",
        );
        assert!(Beneficiaries::<Test>::iter().next().is_none());
        assert_eq!(TotalClaimable::<Test>::get(), BalanceOf::<Test>::zero());
        assert!(ClaimActive::<Test>::get());
        assert_eq!(
            ClaimId::<Test>::get(),
            Some((1, INIT_CLAIM_MESSAGE.clone()))
        );
        assert_eq!(
            Balances::free_balance(Claim::account_id()),
            EXISTENTIAL_DEPOSIT
        );
        assert_eq!(Claim::pot(), 0);
    });
}

#[test]
fn end_claim_with_remaining_benificiaries() {
    let mut e = test();

    e.execute_with(|| {
        // Add MaxOpBeneficiaries
        assert_ok!(Claim::begin_claim(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone(),
            INIT_CLAIM_MESSAGE.clone()
        ));

        let _ = Balances::mint_into(&Claim::account_id(), 1000000000).unwrap(); // Just to be safe

        assert_ok!(Claim::add_beneficiaries(
            Origin::Signed(MANAGER_USER).into(),
            utils::get_beneficiaries_map::<Test>(MaxOpBeneficiaries::get()).0
        ));
    });
    e.commit_all().unwrap();

    e.execute_with(|| {
        // End claim. All the beneficiaries should've been removed
        assert_ok!(Claim::end_claim(Origin::Signed(MANAGER_USER).into()));
        assert_evt(
            Event::ClaimEnded {
                claim_id: 0,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "Claim finished",
        );
        assert_evt(Event::NoMoreBeneficiaries, "No more beneficiaries");
        assert_eq!(Beneficiaries::<Test>::count(), 0);
    });
}

#[test]
fn cannot_init_new_claim_if_leftovers_benificiaries() {
    test().execute_with(|| {
        Beneficiaries::<Test>::insert(USER_6, USER_6_AMOUNT);

        assert_noop!(
            Claim::begin_claim(
                Origin::Signed(MANAGER_USER).into(),
                EMPTY_BENEFICIARIES_MAP.clone(),
                INIT_CLAIM_MESSAGE.clone()
            ),
            Error::<Test>::NonEmptyBeneficiaries
        );

        // Remove beneficiary and try again
        Beneficiaries::<Test>::remove(USER_6);

        assert_ok!(Claim::begin_claim(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone(),
            INIT_CLAIM_MESSAGE.clone()
        ));
        assert_evt(
            Event::ClaimStarted {
                claim_id: 0,
                claim_message: INIT_CLAIM_MESSAGE.clone(),
            },
            "New claim",
        );
    })
}

#[test]
fn validate_unsigned_works() {
    use crate::{Call as ClaimCall, ClaimValidityError};
    use codec::Encode;
    use sp_runtime::{
        traits::{IdentifyAccount, ValidateUnsigned},
        transaction_validity::{
            InvalidTransaction, TransactionLongevity, TransactionValidityError, ValidTransaction,
        },
    };

    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let (user_signer, user_signature, user_signature_prefixed, user_signature_eth) =
            USER_1_SIGN.clone();
        let user_address = user_signer.clone().into_account();
        let claim_id = ClaimId::<Test>::get().unwrap();
        let source = sp_runtime::transaction_validity::TransactionSource::External;

        // Claim bad signature
        let mut bad_signature = user_signature.clone();
        bad_signature.1[0] += 1;

        assert_eq!(
            Pallet::<Test>::validate_unsigned(
                source,
                &ClaimCall::claim {
                    beneficiary: user_signer.clone(),
                    signature: bad_signature
                }
            ),
            Err(TransactionValidityError::Invalid(
                InvalidTransaction::Custom(ClaimValidityError::InvalidSignature.into())
            ))
        );

        // Claim bad user
        let user_2_signer = sp_runtime::testing::UintAuthorityId::from(USER_2);

        assert_eq!(
            Pallet::<Test>::validate_unsigned(
                source,
                &ClaimCall::claim {
                    beneficiary: user_2_signer,
                    signature: user_signature.clone()
                }
            ),
            Err(TransactionValidityError::Invalid(
                InvalidTransaction::Custom(ClaimValidityError::InvalidSignature.into())
            ))
        );

        // Claim beneficiary non existing
        let (nb_signer, nb_signature, _, _) = NON_BENEFICIARY_SIGN.clone();
        assert_eq!(
            Pallet::<Test>::validate_unsigned(
                source,
                &ClaimCall::claim {
                    beneficiary: nb_signer,
                    signature: nb_signature
                }
            ),
            Err(TransactionValidityError::Invalid(
                InvalidTransaction::Custom(ClaimValidityError::BeneficiaryNotFound.into())
            ))
        );

        // Claim ok
        assert_eq!(
            Pallet::<Test>::validate_unsigned(
                source,
                &ClaimCall::claim {
                    beneficiary: user_signer.clone(),
                    signature: user_signature.clone()
                }
            ),
            Ok(ValidTransaction {
                priority: 100,
                requires: vec![],
                provides: vec![("claim", claim_id.clone(), user_address).encode()],
                longevity: TransactionLongevity::MAX,
                propagate: true,
            })
        );

        // Claim ok prefixed
        assert_eq!(
            Pallet::<Test>::validate_unsigned(
                source,
                &ClaimCall::claim {
                    beneficiary: user_signer.clone(),
                    signature: user_signature_prefixed.clone()
                }
            ),
            Ok(ValidTransaction {
                priority: 100,
                requires: vec![],
                provides: vec![("claim", claim_id.clone(), user_address).encode()],
                longevity: TransactionLongevity::MAX,
                propagate: true,
            })
        );

        // Claim ok eth prefixed
        assert_eq!(
            Pallet::<Test>::validate_unsigned(
                source,
                &ClaimCall::claim {
                    beneficiary: user_signer.clone(),
                    signature: user_signature_eth.clone()
                }
            ),
            Ok(ValidTransaction {
                priority: 100,
                requires: vec![],
                provides: vec![("claim", claim_id, user_address).encode()],
                longevity: TransactionLongevity::MAX,
                propagate: true,
            })
        );

        // Claim while inactive
        ClaimActive::<Test>::put(false);
        assert_eq!(
            Pallet::<Test>::validate_unsigned(
                source,
                &ClaimCall::claim {
                    beneficiary: user_signer,
                    signature: user_signature
                }
            ),
            Err(TransactionValidityError::Invalid(
                InvalidTransaction::Custom(ClaimValidityError::ClaimInactive.into())
            ))
        );
    });
}
