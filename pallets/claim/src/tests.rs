use crate::mock;
use crate::mock::RuntimeEvent as TestEvent;
use crate::mock::*;
use crate::*;
use frame_support::{assert_err, assert_noop, assert_ok, dispatch::Pays};
use frame_system::{EventRecord, Phase};
use sp_core::TypedGet;
use sp_runtime::{traits::BadOrigin, TokenError};

pub fn assert_evt(event: Event<Test>, context: &str) {
    assert_evt_gen(true, event, context);
}

pub fn assert_not_evt(event: Event<Test>, context: &str) {
    assert_evt_gen(false, event, context);
}

fn assert_evt_gen(contains: bool, event: Event<Test>, context: &str) {
    let message = match contains {
        true => format!("{context} - CANNOT FIND {:?}", event),
        false => format!("{context} - FOUND {:?}", event),
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
        assert!(!AirdropActive::<Test>::get());
        assert!(AirdropId::<Test>::get().is_none());
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
        assert!(AirdropActive::<Test>::get());
        assert_eq!(AirdropId::<Test>::get(), Some(0));
        assert_eq!(
            Balances::free_balance(Claim::account_id()),
            SUFFICIENT_GENESIS_BALANCE + EXISTENTIAL_DEPOSIT
        );
        assert_eq!(Claim::pot(), SUFFICIENT_GENESIS_BALANCE);
    });
}

#[test]
fn account_id_as_expected() {
    test().execute_with(|| {
        assert_eq!(Claim::account_id(), ClaimAccountId::<Test>::get());
    });
}

#[test]
fn new_airdrop() {
    test().execute_with(|| {
        assert_ok!(Claim::begin_airdrop(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone()
        ));
        assert_evt(Event::AirdropStarted { airdrop_id: 0 }, "New airdrop");
        assert!(Beneficiaries::<Test>::iter().next().is_none());
        assert_eq!(TotalClaimable::<Test>::get(), BalanceOf::<Test>::zero());
        assert!(AirdropActive::<Test>::get());
        assert_eq!(AirdropId::<Test>::get().unwrap(), 0);
        assert_eq!(
            Balances::free_balance(Claim::account_id()),
            EXISTENTIAL_DEPOSIT
        );
        assert_eq!(Claim::pot(), 0);
    })
}

#[test]
fn new_airdrop_pays_no_fee() {
    test().execute_with(|| {
        assert_eq!(
            Claim::begin_airdrop(
                Origin::Signed(MANAGER_USER).into(),
                EMPTY_BENEFICIARIES_MAP.clone()
            )
            .unwrap()
            .pays_fee,
            Pays::No
        );
    })
}

#[test]
fn new_airdrop_wrong_origin() {
    test().execute_with(|| {
        assert_err!(
            Claim::begin_airdrop(
                Origin::Signed(USER_1).into(),
                EMPTY_BENEFICIARIES_MAP.clone()
            ),
            BadOrigin
        );
        assert_not_evt(Event::AirdropStarted { airdrop_id: 0 }, "No new airdrop");
    })
}

#[test]
fn new_airdrop_sufficient_funds() {
    test_with_configs(
        WithGenesisBeneficiaries::No,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::begin_airdrop(
            Origin::Signed(MANAGER_USER).into(),
            GENESIS_BENEFICIARIES_MAP.clone()
        ));
        assert_evt(Event::AirdropStarted { airdrop_id: 0 }, "New airdrop");
        assert_eq!(
            Beneficiaries::<Test>::iter().collect::<BTreeMap<_, _>>(),
            GENESIS_BENEFICIARIES_MAP.clone()
        );
        assert_eq!(TotalClaimable::<Test>::get(), SUFFICIENT_GENESIS_BALANCE);
        assert!(AirdropActive::<Test>::get());
        assert_eq!(AirdropId::<Test>::get(), Some(0));
    })
}

#[test]
fn new_airdrop_insufficient_funds() {
    test_with_configs(
        WithGenesisBeneficiaries::No,
        GenesisClaimBalance::Insufficient,
    )
    .execute_with(|| {
        assert_noop!(
            Claim::begin_airdrop(
                Origin::Signed(MANAGER_USER).into(),
                GENESIS_BENEFICIARIES_MAP.clone()
            ),
            TokenError::FundsUnavailable
        );
    })
}

#[test]
fn cannot_start_new_airdrop_if_one_already_in_progress() {
    test().execute_with(|| {
        assert_ok!(Claim::begin_airdrop(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone()
        ));
        assert_evt(Event::AirdropStarted { airdrop_id: 0 }, "New airdrop");
        assert_err!(
            Claim::begin_airdrop(
                Origin::Signed(MANAGER_USER).into(),
                GENESIS_BENEFICIARIES_MAP.clone()
            ),
            Error::<Test>::AlreadyStarted
        );
        assert_not_evt(Event::AirdropStarted { airdrop_id: 1 }, "No new airdrop");
    })
}

#[test]
fn claim() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::claim(Origin::Signed(USER_1).into(), None));
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
fn claim_pays_no_fee() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_eq!(
            Claim::claim(Origin::Signed(USER_1).into(), None)
                .unwrap()
                .pays_fee,
            Pays::No
        );
    });
}

#[test]
fn double_claim_is_err() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::claim(Origin::Signed(USER_1).into(), None));
        assert_err!(
            Claim::claim(Origin::Signed(USER_1).into(), None),
            Error::<Test>::NotEligible
        )
    });
}

#[test]
fn claim_with_opt_dest() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::claim(Origin::Signed(USER_1).into(), Some(USER_2)));
        assert_evt(
            Event::Claimed {
                beneficiary: USER_2,
                amount: USER_1_AMOUNT,
            },
            "Successfull claim for a different dest",
        );
        assert_eq!(Claim::pot(), SUFFICIENT_GENESIS_BALANCE - USER_1_AMOUNT);
        assert_eq!(
            TotalClaimable::<Test>::get(),
            SUFFICIENT_GENESIS_BALANCE - USER_1_AMOUNT
        );
        assert_eq!(Balances::free_balance(USER_1), 0);
        assert_eq!(Balances::free_balance(USER_2), USER_1_AMOUNT);
        assert!(Beneficiaries::<Test>::get(USER_1).is_none());
    });
}

#[test]
fn claim_wrong_beneficiary() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_noop!(
            Claim::claim(Origin::Signed(NON_BENEFICIARY).into(), None),
            Error::<Test>::NotEligible
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
        Beneficiaries::<Test>::insert(NON_BENEFICIARY, SUFFICIENT_GENESIS_BALANCE + 1);
        assert_err!(
            Claim::claim(Origin::Signed(NON_BENEFICIARY).into(), None),
            TokenError::FundsUnavailable
        );
        assert_not_evt(
            Event::Claimed {
                beneficiary: NON_BENEFICIARY,
                amount: SUFFICIENT_GENESIS_BALANCE + 1,
            },
            "Cannot claim if money not available",
        );
    })
}

#[test]
fn cannot_claim_while_airdrop_inactive() {
    test().execute_with(|| {
        assert_err!(
            Claim::claim(Origin::Signed(USER_1).into(), None),
            Error::<Test>::AlreadyEnded
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
        assert_ok!(Claim::claim_for(Origin::None.into(), USER_1));
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
fn claim_for_pays_no_fee() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_eq!(
            Claim::claim_for(Origin::None.into(), USER_1)
                .unwrap()
                .pays_fee,
            Pays::No
        );
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
            Claim::claim_for(Origin::None.into(), NON_BENEFICIARY),
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
            Claim::claim_for(Origin::None.into(), NON_BENEFICIARY),
            Error::<Test>::NotEligible
        );
    });
}

#[test]
fn cannot_claim_for_while_airdrop_inactive() {
    test().execute_with(|| {
        assert_err!(
            Claim::claim_for(Origin::None.into(), USER_1),
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
fn cannot_add_beneficiaries_while_airdrop_inactive() {
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
fn add_beneficiaries_pays_no_fee() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        let _ = Balances::mint_into(&Claim::account_id(), NEW_SUFFICIENT_BALANCE).unwrap();
        assert_eq!(
            Claim::add_beneficiaries(
                Origin::Signed(MANAGER_USER).into(),
                NEW_BENEFICIARIES_MAP.clone()
            )
            .unwrap()
            .pays_fee,
            Pays::No
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
fn end_airdrop() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        // Give other balance. Now Self::pot() == SUFFICIENT_GENESIS_BALANCE * 2
        let _ = Balances::mint_into(&Claim::account_id(), SUFFICIENT_GENESIS_BALANCE).unwrap();
        assert_ok!(Claim::end_airdrop(Origin::Signed(MANAGER_USER).into()));
        assert!(!AirdropActive::<Test>::get());
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
fn end_airdrop_pays_no_fee() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_eq!(
            Claim::end_airdrop(Origin::Signed(MANAGER_USER).into())
                .unwrap()
                .pays_fee,
            Pays::No
        );
    });
}

#[test]
fn end_airdrop_wrong_origin() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_err!(Claim::end_airdrop(Origin::Signed(USER_1).into()), BadOrigin);
        assert_not_evt(Event::AirdropEnded { airdrop_id: 0 }, "No end airdrop");
    })
}

#[test]
fn double_end_airdrop() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::end_airdrop(Origin::Signed(MANAGER_USER).into()));
        assert_err!(
            Claim::end_airdrop(Origin::Signed(MANAGER_USER).into()),
            Error::<Test>::AlreadyEnded
        );
    });
}

#[test]
fn end_airdrop_new_airdrop() {
    test_with_configs(
        WithGenesisBeneficiaries::Yes,
        GenesisClaimBalance::Sufficient,
    )
    .execute_with(|| {
        assert_ok!(Claim::end_airdrop(Origin::Signed(MANAGER_USER).into()));
        assert_ok!(Claim::begin_airdrop(
            Origin::Signed(MANAGER_USER).into(),
            EMPTY_BENEFICIARIES_MAP.clone()
        ));
        assert_evt(Event::AirdropStarted { airdrop_id: 1 }, "New airdrop");
        assert!(Beneficiaries::<Test>::iter().next().is_none());
        assert_eq!(TotalClaimable::<Test>::get(), BalanceOf::<Test>::zero());
        assert!(AirdropActive::<Test>::get());
        assert_eq!(AirdropId::<Test>::get(), Some(1));
        assert_eq!(
            Balances::free_balance(Claim::account_id()),
            EXISTENTIAL_DEPOSIT
        );
        assert_eq!(Claim::pot(), 0);
    });
}
