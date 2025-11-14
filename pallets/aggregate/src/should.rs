// Copyright 20USER_2, Horizen Labs, Inc.
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

#![cfg(test)]

use frame_support::{
    assert_err, assert_noop, assert_ok,
    dispatch::{DispatchInfo, GetDispatchInfo, Pays},
    traits::{fungible::InspectHold, Hooks},
    weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BadOrigin, Keccak256},
    SaturatedConversion,
};

use super::*;
use data::{
    AggregateSecurityRules, Delivery, DeliveryParams, DomainState, ProofSecurityRules, Reserve,
};
use mock::*;

use hp_dispatch::{Destination, DispatchAggregation};
use hp_on_proof_verified::OnProofVerified;

use rstest::rstest;

use utility::*;

mod utility;

#[test]
fn add_a_proof() {
    test().execute_with(|| {
        let statement = H256::from_low_u64_be(123);

        Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

        assert_proof_evt(DOMAIN_ID, 1, statement);
        let att = &Domains::<Test>::get(DOMAIN_ID).unwrap().next;
        assert_eq!(1, att.id);
        assert_eq!(
            vec![statement_entry(None, USER_1, statement)],
            *att.statements
        );
    })
}

#[test]
fn emit_domain_full_event_when_publish_queue_is_full() {
    test().execute_with(|| {
        let statements = DOMAIN_QUEUE_SIZE * DOMAIN_SIZE as u32;
        let event = Event::DomainFull {
            domain_id: DOMAIN_ID,
        };

        for _ in 0..statements - 1 {
            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, Default::default());
        }

        assert_not_evt(event.clone(), "Domain full");
        Aggregate::on_proof_verified(Some(USER_1), DOMAIN, Default::default());

        assert_evt(event, "Domain full");
    })
}

mod not_add_the_statement_to {
    use super::*;

    mod any_domain_if {
        use super::*;

        #[test]
        fn no_domain_provided() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(Some(USER_1), None, statement);

                assert_no_cannot_aggregate_evt();

                assert_eq!(0, count_all_statements());
            })
        }

        #[test]
        fn no_account_provided() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(None, DOMAIN, statement);

                assert_cannot_aggregate_evt(statement, CannotAggregateCause::NoAccount);

                assert_eq!(0, count_all_statements());
            })
        }
    }

    mod provided_domain_if {
        use super::*;

        #[test]
        fn is_not_registered() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(Some(USER_1), NOT_REGISTERED_DOMAIN, statement);

                assert_cannot_aggregate_evt(
                    statement,
                    CannotAggregateCause::DomainNotRegistered {
                        domain_id: NOT_REGISTERED_DOMAIN_ID,
                    },
                );

                assert_eq!(0, count_all_statements());
            })
        }

        #[rstest]
        fn is_on_hold_or_removable_state(
            #[values(DomainState::Hold, DomainState::Removable, DomainState::Removed)]
            state: DomainState,
        ) {
            test().execute_with(|| {
                Domains::<Test>::mutate_extant(DOMAIN_ID, |d| {
                    d.state = state;
                });

                let statement = H256::from_low_u64_be(123);
                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

                assert_cannot_aggregate_evt(
                    statement,
                    CannotAggregateCause::InvalidDomainState {
                        domain_id: DOMAIN_ID,
                        state,
                    },
                );
                assert_eq!(0, count_all_statements());
            })
        }
    }
}

mod add_statement_accordingly_to_proof_submitting_rule {
    use super::*;

    #[rstest]
    fn untrusted_accept_any_user(
        #[values(USER_1, USER_2, USER_ALLOWLISTED_1, USER_DOMAIN_SUBMIT_RULE)] user: AccountId,
    ) {
        test().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(user), DOMAIN, statement);
            assert_proof_evt(DOMAIN_ID, 1, statement);
        })
    }

    #[rstest]
    fn only_owner_reject_no_owner(#[values(USER_1, USER_2, USER_ALLOWLISTED_1)] user: AccountId) {
        test().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(user), DOMAIN_ONLY_OWNER, statement);

            assert_cannot_aggregate_evt(statement, CannotAggregateCause::UnauthorizedUser);

            assert_eq!(0, count_all_statements());
        })
    }

    #[test]
    fn only_owner_accept_owner() {
        test().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(
                Some(USER_DOMAIN_SUBMIT_RULE),
                DOMAIN_ONLY_OWNER,
                statement,
            );
            assert_proof_evt(DOMAIN_ID_ONLY_OWNER, 1, statement);
        })
    }

    #[rstest]
    fn whitlisted_reject_no_allowlisted(
        #[values(USER_1, USER_2, USER_DOMAIN_SUBMIT_RULE)] user: AccountId,
    ) {
        test().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(user), DOMAIN_ALLOWLISTED, statement);

            assert_cannot_aggregate_evt(statement, CannotAggregateCause::UnauthorizedUser);

            assert_eq!(0, count_all_statements());
        })
    }

    #[rstest]
    fn whitlisted_accept_whitlisted(
        #[values(USER_ALLOWLISTED_1, USER_ALLOWLISTED_2)] user: AccountId,
    ) {
        test().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(user), DOMAIN_ALLOWLISTED, statement);
            assert_proof_evt(DOMAIN_ID_ALLOWLISTED, 1, statement);
        })
    }
}
mod check_if_no_room_for_new_statements_in_should_published_set_and {
    use super::*;

    const LAST_ID: u64 = 999;

    /// Fill the domain with MaxPendingPublishQueueSize::get() aggregations in should published set,
    /// and fill the next one with  AggregationSize::get()-1 statements.
    fn test() -> sp_io::TestExternalities {
        let mut ext = super::test();
        let size = <Test as crate::Config>::AggregationSize::get();

        ext.execute_with(|| {
            Domains::<Test>::mutate_extant(DOMAIN_ID, |d| {
                for i in 1..=DOMAIN_QUEUE_SIZE as u64 {
                    d.should_publish
                        .try_insert(i, Aggregation::<Test>::create(i, size))
                        .unwrap();
                }
                d.next = Aggregation::<Test>::create(LAST_ID, size);
                for i in 0..(size - 1) {
                    d.next.add_statement(
                        USER_1,
                        Reserve::<Balance>::new(35_u32.into(), 0_u32.into()),
                        H256::from_low_u64_be(i.into()),
                    );
                }
            });
        });
        ext
    }

    mod on_proof_verified {
        use super::*;

        #[test]
        fn not_add_any_statement() {
            test().execute_with(|| {
                let statements = count_all_statements();

                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, H256::from_low_u64_be(123));

                assert_eq!(statements, count_all_statements());
            })
        }

        #[test]
        fn not_emit_aggregation_event() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

                assert_not_proof_evt(DOMAIN_ID, LAST_ID, statement);
            })
        }

        #[test]
        fn not_emit_queue_aggregation() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

                assert_not_complete_evt(DOMAIN_ID, LAST_ID);
            })
        }

        #[test]
        fn not_hold_currency() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

                assert_eq!(
                    Balances::reserved_balance(USER_1),
                    0,
                    "Should not hold any balance"
                );
            })
        }

        #[test]
        fn emit_cannot_aggregate_event() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

                assert_not_complete_evt(DOMAIN_ID, LAST_ID);
                assert_cannot_aggregate_evt(
                    statement,
                    CannotAggregateCause::DomainStorageFull {
                        domain_id: DOMAIN_ID,
                    },
                );
            })
        }
    }

    #[test]
    fn free_room_for_new_aggregations_when_old_aggregated() {
        test().execute_with(|| {
            Aggregate::aggregate(Origin::Signed(33).into(), DOMAIN_ID, 1).unwrap();
            mock::System::reset_events();

            let statement = H256::from_low_u64_be(123);
            let account = USER_1;
            Aggregate::on_proof_verified(Some(account), DOMAIN, statement);

            assert_proof_evt(DOMAIN_ID, LAST_ID, statement);
            assert_complete_evt(DOMAIN_ID, LAST_ID);
            assert_evt(
                Event::DomainFull {
                    domain_id: DOMAIN_ID,
                },
                "Domain full",
            );
        })
    }

    #[test]
    fn free_room_for_aggregation_when_olds_aggregated_more_than_once() {
        test().execute_with(|| {
            Aggregate::aggregate(Origin::Signed(33).into(), DOMAIN_ID, 1).unwrap();
            Aggregate::aggregate(Origin::Signed(33).into(), DOMAIN_ID, 3).unwrap();
            Aggregate::aggregate(Origin::Signed(33).into(), DOMAIN_ID, 5).unwrap();
            System::events().clear();

            let statement = H256::from_low_u64_be(123);
            let event = Event::DomainFull {
                domain_id: DOMAIN_ID,
            };

            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

            assert_proof_evt(DOMAIN_ID, LAST_ID, statement);
            assert_complete_evt(DOMAIN_ID, LAST_ID);
            // To be sure we are not full
            assert_not_evt(event.clone(), "Domain full");

            let statements = 2 * <Test as Config>::AggregationSize::get() as u64;
            for p in 0..(statements - 1) {
                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, H256::from_low_u64_be(123 + p));
            }
            // One statement is missed to full the domain
            assert_not_evt(event.clone(), "Domain full");

            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, H256::from_low_u64_be(123));
            // Now is full again
            assert_evt(event, "Domain full");
        })
    }
}

#[test]
fn queue_a_new_aggregation_when_is_complete() {
    test().execute_with(|| {
        let elements = (0..DOMAIN_SIZE)
            .map(|i| statement_entry(None, USER_1, H256::from_low_u64_be(i.into())))
            .collect::<Vec<_>>();
        for s in elements.clone().into_iter() {
            Aggregate::on_proof_verified(Some(s.account), DOMAIN, s.statement);
        }

        assert_complete_evt(DOMAIN_ID, 1);

        let att = Domains::<Test>::take(DOMAIN_ID)
            .and_then(|mut d| d.should_publish.remove(&1))
            .unwrap();
        assert_eq!(1, att.id);
        assert_eq!(elements, *att.statements);
    })
}
#[test]
fn reserve_at_least_the_publish_proof_price_fraction_when_on_proof_verified() {
    test().execute_with(|| {
        let statement = H256::from_low_u64_be(123);
        let account = USER_1;

        Aggregate::on_proof_verified(Some(account), DOMAIN, statement);

        assert_eq!(Balances::reserved_balance(account), DOMAIN_FEE);
    })
}

const DELIVERY_FEE: u128 = 64 * 10000;
const OWNER_TIP: u128 = 100;

const EXPECTED_DELIVERY_HOLD_FUNDS: u128 = (DELIVERY_FEE + OWNER_TIP) / DOMAIN_SIZE as u128;

fn set_total_delivery_fee(domain_id: u32, fee: Balance, owner_tip: Balance) {
    Domains::<Test>::mutate_extant(domain_id, |d| {
        d.delivery.set_fee(fee);
        d.delivery.set_owner_tip(owner_tip);
    });
}

#[test]
fn reserve_the_delivery_price_fraction_when_on_proof_verified() {
    test().execute_with(|| {
        set_total_delivery_fee(DOMAIN_ID, DELIVERY_FEE, OWNER_TIP);

        let statement = H256::from_low_u64_be(123);
        let account = USER_1;

        Aggregate::on_proof_verified(Some(account), DOMAIN, statement);

        assert_eq!(
            Balances::reserved_balance(account),
            DOMAIN_FEE + EXPECTED_DELIVERY_HOLD_FUNDS
        );
    })
}

#[test]
fn call_estimate_fee_with_the_correct_post_info_when_on_proof_verified() {
    test().execute_with(|| {
        let statement = H256::from_low_u64_be(123);
        let account = USER_1;

        Aggregate::on_proof_verified(Some(account), DOMAIN, statement);

        assert_eq!(
            MockEstimateCallFee::pop().unwrap().post_info.actual_weight,
            Some(<Test as Config>::WeightInfo::aggregate(DOMAIN_SIZE as u32))
        );
    })
}

#[test]
fn not_fail_but_raise_just_an_event_if_a_user_doesn_t_have_enough_found_to_reserve_for_aggregate_on_proof_verified(
) {
    test().execute_with(|| {
        let statement = H256::from_low_u64_be(123);
        set_total_delivery_fee(DOMAIN_ID, DELIVERY_FEE, OWNER_TIP);

        Aggregate::on_proof_verified(Some(NO_DELIVERY_FUND_USER), DOMAIN, statement);

        assert_eq!(
            Balances::reserved_balance(NO_DELIVERY_FUND_USER),
            0,
            "Should not reserve any balance"
        );
        assert_cannot_aggregate_evt(statement, CannotAggregateCause::InsufficientFunds);
        assert_eq!(1, System::events().len())
    })
}

#[test]
fn not_fail_but_raise_just_an_event_if_a_user_doesn_t_have_enough_found_to_reserve_for_delivering_on_proof_verified(
) {
    test().execute_with(|| {
        let statement = H256::from_low_u64_be(123);

        Aggregate::on_proof_verified(Some(NO_DOMAIN_FEE_FUND_USER), DOMAIN, statement);

        assert_eq!(
            Balances::reserved_balance(NO_DOMAIN_FEE_FUND_USER),
            0,
            "Should not reserve any balance"
        );
        assert_cannot_aggregate_evt(statement, CannotAggregateCause::InsufficientFunds);
        assert_eq!(1, System::events().len())
    })
}

mod clean_the_published_storage_on_initialize {
    use super::*;

    #[test]
    fn in_base_case() {
        test().execute_with(|| {
            assert!(Published::<Test>::get().is_empty());
        })
    }

    #[test]
    fn when_some_aggregations_are_present() {
        test().execute_with(|| {
            Published::<Test>::mutate(|published: &mut _| {
                published.push((1, Aggregation::<Test>::create(12, 3)));
                published.push((2, Aggregation::<Test>::create(13, 3)));
            });

            Aggregate::on_initialize(36);
            assert!(Published::<Test>::get().is_empty());
        })
    }

    #[test]
    fn and_return_the_correct_weight() {
        test().execute_with(|| {
            Published::<Test>::mutate(|published: &mut _| {
                published.push((2, Aggregation::<Test>::create(12, 3)));
                published.push((2, Aggregation::<Test>::create(13, 3)));
            });

            let w = Aggregate::on_initialize(36);
            assert_eq!(w, db_weights().writes(1));
            // Sanity check: w is not void
            assert_ne!(w, 0.into());
        })
    }
}

mod aggregate {
    use super::*;

    fn dispatch_info() -> DispatchInfo {
        Call::<Test>::aggregate {
            domain_id: 2,
            aggregation_id: 42,
        }
        .get_dispatch_info()
    }

    fn add_aggregations(user: Option<AccountId>, domain: Option<u32>, size: u32) {
        for i in 0..size {
            Aggregate::on_proof_verified(user, domain, H256::from_low_u64_be(i.into()));
        }
    }

    #[test]
    fn emit_a_new_receipt() {
        test().execute_with(|| {
            add_aggregations(Some(USER_2), DOMAIN, DOMAIN_SIZE);

            assert_ok!(Aggregate::aggregate(
                Origin::Signed(USER_1).into(),
                DOMAIN_ID,
                1
            ));

            assert_new_receipt(DOMAIN_ID, 1, None);
        })
    }

    #[test]
    fn dispatch_aggregation() {
        test().execute_with(|| {
            add_aggregations(Some(USER_2), DOMAIN, DOMAIN_SIZE);

            // Record initial balance of delivery owner
            let initial_balance = Balances::free_balance(USER_DELIVERY_OWNER);

            Aggregate::aggregate(Origin::Signed(USER_1).into(), DOMAIN_ID, 1).unwrap();

            let MockDispatchAggregation {
                domain_id,
                aggregation_id,
                aggregation,
                destination,
                delivery_owner,
                ..
            } = MockDispatchAggregation::pop().expect("No call received");

            assert_new_receipt(domain_id, aggregation_id, Some(aggregation));
            assert_eq!(hyperbridge_destination(), destination);
            assert_eq!(USER_DELIVERY_OWNER, delivery_owner);

            // Get the domain to access delivery fee and tip
            let domain = Domains::<Test>::get(domain_id).unwrap();
            let owner_tip = *domain.delivery.owner_tip();

            // Verify delivery owner's final balance
            let final_balance = Balances::free_balance(USER_DELIVERY_OWNER);
            assert_eq!(
                final_balance,
                initial_balance + owner_tip,
                "Delivery owner should only receive their tip"
            );
        })
    }

    #[test]
    fn route_aggregation_to_correct_destination() {
        test().execute_with(|| {
            add_aggregations(Some(USER_2), DOMAIN_NONE, DOMAIN_SIZE);

            Aggregate::aggregate(Origin::Signed(USER_1).into(), DOMAIN_ID_NONE, 1).unwrap();

            let MockDispatchAggregation {
                domain_id: _,
                aggregation_id: _,
                aggregation: _,
                destination,
                delivery_owner,
                ..
            } = MockDispatchAggregation::pop().expect("No call received");

            assert_eq!(none_delivering().destination, destination);
            assert_eq!(USER_DELIVERY_OWNER, delivery_owner);
        })
    }

    #[test]
    fn accept_also_composing_aggregation() {
        test().execute_with(|| {
            add_aggregations(Some(USER_2), DOMAIN, DOMAIN_SIZE / 2);

            assert_ok!(Aggregate::aggregate(
                Origin::Signed(USER_1).into(),
                DOMAIN_ID,
                1
            ));
            assert_new_receipt(DOMAIN_ID, 1, None);
        })
    }

    #[test]
    fn refund_the_publisher_from_the_reserved_funds() {
        test().execute_with(|| {
            let accounts = [USER_1, USER_2];
            let elements = (0..DOMAIN_SIZE as u64)
                .map(|i| {
                    (
                        accounts[(i % accounts.len().saturated_into::<u64>()) as usize],
                        H256::from_low_u64_be(i.into()),
                    )
                })
                .collect::<Vec<(u64, _)>>();
            for (account, statement) in elements.clone().into_iter() {
                Aggregate::on_proof_verified(Some(account), DOMAIN, statement);
            }
            let expected_balance =
                Balances::free_balance(PUBLISHER_USER) + ESTIMATED_FEE_CORRECTED as u128;

            assert_ok!(Aggregate::aggregate(
                Origin::Signed(PUBLISHER_USER).into(),
                DOMAIN_ID,
                1
            ));

            assert_eq!(Balances::free_balance(PUBLISHER_USER), expected_balance);
        })
    }

    #[test]
    fn un_hold_the_submitters_aggregation_funds_if_called_by_the_manager() {
        test().execute_with(|| {
            let accounts = [USER_1, USER_2];
            let elements = (0..DOMAIN_SIZE as u64)
                .map(|i| {
                    (
                        accounts[(i % accounts.len().saturated_into::<u64>()) as usize],
                        H256::from_low_u64_be(i.into()),
                    )
                })
                .collect::<Vec<(u64, _)>>();
            let expected_balances = [
                Balances::free_balance(USER_1),
                Balances::free_balance(USER_2),
            ];

            for (account, statement) in elements.clone().into_iter() {
                Aggregate::on_proof_verified(Some(account), DOMAIN, statement);
            }
            let expected_root_balance = Balances::free_balance(ROOT_USER);

            assert_ok!(Aggregate::aggregate(
                Origin::Signed(ROOT_USER).into(),
                DOMAIN_ID,
                1
            ));

            assert_eq!(
                Balances::free_balance(ROOT_USER),
                expected_root_balance,
                "Manager should not receive any funds"
            );
            assert_eq!(
                Balances::free_balance(USER_1),
                expected_balances[0],
                "Users should be re-founded"
            );
            assert_eq!(
                Balances::free_balance(USER_2),
                expected_balances[1],
                "Users should be re-founded"
            );
        })
    }

    #[rstest]
    fn pay_the_delivery_owner_for_delivering_from_the_reserved_funds(
        #[values(PUBLISHER_USER, ROOT_USER)] executor: AccountId,
    ) {
        test().execute_with(|| {
            set_total_delivery_fee(DOMAIN_ID, DELIVERY_FEE, OWNER_TIP);
            let accounts = [USER_1, USER_2];
            let elements = (0..DOMAIN_SIZE as u64)
                .map(|i| {
                    (
                        accounts[(i % accounts.len().saturated_into::<u64>()) as usize],
                        H256::from_low_u64_be(i.into()),
                    )
                })
                .collect::<Vec<(u64, _)>>();
            for (account, statement) in elements.clone().into_iter() {
                Aggregate::on_proof_verified(Some(account), DOMAIN, statement);
            }

            let delivery_per_statement = (DELIVERY_FEE + OWNER_TIP) / DOMAIN_SIZE as u128;
            let total_collected = delivery_per_statement * DOMAIN_SIZE as u128;
            let expected_balance = Balances::free_balance(USER_DELIVERY_OWNER) + total_collected;

            assert_ok!(Aggregate::aggregate(
                Origin::Signed(executor).into(),
                DOMAIN_ID,
                1
            ));

            assert_eq!(
                Balances::free_balance(USER_DELIVERY_OWNER),
                expected_balance
            );
        })
    }

    #[rstest]
    fn after_aggregate_submitter_user_should_not_have_any_funds_on_hold(
        #[values(PUBLISHER_USER, ROOT_USER)] executor: AccountId,
    ) {
        test().execute_with(|| {
            set_total_delivery_fee(DOMAIN_ID, 2 * DELIVERY_FEE, OWNER_TIP);
            let accounts = [USER_1, USER_2];
            let elements = (0..DOMAIN_SIZE as u64)
                .map(|i| {
                    (
                        accounts[(i % accounts.len().saturated_into::<u64>()) as usize],
                        H256::from_low_u64_be(i.into()),
                    )
                })
                .collect::<Vec<(u64, _)>>();
            for (account, statement) in elements.clone().into_iter() {
                Aggregate::on_proof_verified(Some(account), DOMAIN, statement);
            }
            assert_ok!(Aggregate::aggregate(
                Origin::Signed(executor).into(),
                DOMAIN_ID,
                1
            ));

            assert_eq!(Balances::total_balance_on_hold(&USER_1), 0);
            assert_eq!(Balances::total_balance_on_hold(&USER_2), 0);
        })
    }

    mod policies {
        use super::*;

        fn init_domain(
            owner: AccountId,
            rules: AggregateSecurityRules,
            size: u32,
            completed: bool,
            owner_delivery: Option<AccountId>,
        ) -> (sp_io::TestExternalities, u32, u64) {
            let mut ex = test();
            let (domain_id, aggregation_id) = ex.execute_with(|| {
                assert_ok!(Aggregate::register_domain(
                    Origin::Signed(owner).into(),
                    size,
                    None,
                    rules,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    owner_delivery
                ));

                let domain_id = registered_ids()[0];
                let aggregation_id = Domains::<Test>::mutate_extant(domain_id, |d| {
                    let size = if completed { size } else { size - 1 };
                    let aggregation_id = d.next.id;
                    for i in 0..size {
                        d.next.add_statement(
                            USER_1,
                            Default::default(),
                            H256::from_low_u64_be(i.into()),
                        );
                    }
                    aggregation_id
                });
                (domain_id, aggregation_id)
            });
            (ex, domain_id, aggregation_id)
        }

        #[rstest]
        fn untrusted_should_accept_any_case(
            #[values(USER_1, USER_2, ROOT_USER)] aggregator: AccountId,
            #[values(true, false)] completed: bool,
        ) {
            let (mut test, domain_id, aggregation_id) = init_domain(
                USER_1,
                AggregateSecurityRules::Untrusted,
                16,
                completed,
                None,
            );
            test.execute_with(|| {
                assert_ok!(Aggregate::aggregate(
                    Origin::Signed(aggregator).into(),
                    domain_id,
                    aggregation_id
                ));
            })
        }

        #[rstest]
        fn only_owner_only_owner_uncompleted_and_should_accept_aggregate_call(
            #[values(
                AggregateSecurityRules::OnlyOwner,
                AggregateSecurityRules::OnlyOwnerUncompleted
            )]
            rules: AggregateSecurityRules,
            #[values(USER_1, ROOT_USER, USER_DELIVERY_OWNER)] aggregator: AccountId,
            #[values(true, false)] completed: bool,
        ) {
            let (mut test, domain_id, aggregation_id) =
                init_domain(USER_1, rules, 16, completed, Some(USER_DELIVERY_OWNER));
            test.execute_with(|| {
                assert_ok!(Aggregate::aggregate(
                    Origin::Signed(aggregator).into(),
                    domain_id,
                    aggregation_id
                ));
            })
        }

        #[test]
        fn only_owner_uncompleted_should_accept_call_from_untrusted_user_on_completed_aggregation()
        {
            let (mut test, domain_id, aggregation_id) = init_domain(
                USER_1,
                AggregateSecurityRules::OnlyOwnerUncompleted,
                16,
                true,
                None,
            );
            test.execute_with(|| {
                assert_ok!(Aggregate::aggregate(
                    Origin::Signed(USER_2).into(),
                    domain_id,
                    aggregation_id
                ));
            })
        }

        #[rstest]
        #[case::on_only_owner_from_untrusted_user_on_uncompleted_aggregation(
            AggregateSecurityRules::OnlyOwner,
            USER_2,
            false
        )]
        #[case::on_only_owner_from_untrusted_user_on_completed_aggregation(
            AggregateSecurityRules::OnlyOwner,
            USER_2,
            true
        )]
        #[case::on_only_owner_uncompleted_from_untrusted_user_on_uncompleted_aggregation(
            AggregateSecurityRules::OnlyOwner,
            USER_2,
            false
        )]
        fn should_reject_aggregate(
            #[case] rules: AggregateSecurityRules,
            #[case] aggregator: AccountId,
            #[case] completed: bool,
        ) {
            let (mut test, domain_id, aggregation_id) =
                init_domain(USER_1, rules, 16, completed, Some(USER_DELIVERY_OWNER));
            test.execute_with(|| {
                assert_noop!(
                    Aggregate::aggregate(
                        Origin::Signed(aggregator).into(),
                        domain_id,
                        aggregation_id
                    ),
                    BadOrigin
                );
            })
        }
    }

    #[test]
    fn raise_error_if_invalid_domain_is_used() {
        test().execute_with(|| {
            let err =
                Aggregate::aggregate(Origin::Signed(USER_1).into(), NOT_REGISTERED_DOMAIN_ID, 1)
                    .unwrap_err()
                    .error;
            assert_eq!(err, Error::<Test>::UnknownDomainId.into());
        })
    }

    #[test]
    fn raise_error_if_aggregation_fails() {
        test().execute_with(|| {
            let sentinel = sp_runtime::DispatchError::Other("SENTINEL");
            MockDispatchAggregation::set_return(Err(sentinel));

            add_aggregations(Some(USER_2), DOMAIN, DOMAIN_SIZE);

            assert_noop!(
                Aggregate::aggregate(Origin::Signed(USER_1).into(), DOMAIN_ID, 1),
                sentinel
            );
        })
    }

    #[test]
    fn dont_pay_for_a_full_proof_if_invalid_domain_is_used() {
        test().execute_with(|| {
            let post_info =
                Aggregate::aggregate(Origin::Signed(USER_1).into(), NOT_REGISTERED_DOMAIN_ID, 1)
                    .unwrap_err()
                    .post_info;
            assert_eq!(
                post_info,
                Some(<Test as Config>::WeightInfo::aggregate_on_invalid_domain()).into()
            );
        })
    }

    #[test]
    fn raise_error_if_invalid_id_is_used() {
        test().execute_with(|| {
            for i in 0..<Test as crate::Config>::AggregationSize::get() {
                Aggregate::on_proof_verified(Some(USER_2), DOMAIN, H256::from_low_u64_be(i.into()));
            }

            let err = Aggregate::aggregate(Origin::Signed(USER_1).into(), DOMAIN_ID, 1000)
                .unwrap_err()
                .error;
            assert_eq!(err, Error::<Test>::InvalidAggregationId.into());
        })
    }

    #[test]
    fn dont_pay_for_a_full_proof_if_invalid_id_is_used() {
        test().execute_with(|| {
            for i in 0..<Test as crate::Config>::AggregationSize::get() {
                Aggregate::on_proof_verified(Some(USER_2), DOMAIN, H256::from_low_u64_be(i.into()));
            }

            let post_info = Aggregate::aggregate(Origin::Signed(USER_1).into(), DOMAIN_ID, 1000)
                .unwrap_err()
                .post_info;
            assert_eq!(
                post_info,
                Some(<Test as Config>::WeightInfo::aggregate_on_invalid_id()).into()
            );
        })
    }

    #[rstest]
    #[case::normal_user_pays(USER_1, Pays::Yes)]
    #[case::manager_not_pay(ROOT_USER, Pays::No)]
    fn should_pay(#[case] executor: AccountId, #[case] pays_fee: Pays) {
        test().execute_with(|| {
            add_aggregations(Some(USER_2), DOMAIN, DOMAIN_SIZE);

            let pays = Aggregate::aggregate(Origin::Signed(executor).into(), DOMAIN_ID, 1)
                .unwrap()
                .pays_fee;

            assert_eq!(pays_fee, pays);
        });
    }

    #[test]
    fn use_correct_weight() {
        let info = dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(
            info.call_weight,
            MockWeightInfo::aggregate(MaxAggregationSize::get() as u32)
                + MockDispatchAggregation::max_weight()
        );
    }

    #[rstest]
    #[case::full(DOMAIN_SIZE)]
    #[case::half(DOMAIN_SIZE/2)]
    #[case::just_one_proof(1)]
    fn should_pay_just_for_the_real_used_weight(
        #[case] proofs: u32,
        #[values(
            (DOMAIN_ID, hyperbridge_destination()),
            (DOMAIN_ID_NONE, none_destination())
        )]
        (domain_id, destination): (u32, Destination),
    ) {
        test().execute_with(|| {
            for _ in 0..proofs {
                Aggregate::on_proof_verified(Some(USER_1), Some(domain_id), Default::default());
            }

            let expected_weight = <Test as Config>::WeightInfo::aggregate(proofs)
                + <<Test as Config>::DispatchAggregation as DispatchAggregation<
                    Balance,
                    AccountId,
                >>::dispatch_weight(&destination);

            assert_eq!(
                expected_weight,
                Aggregate::aggregate(Origin::Signed(PUBLISHER_USER).into(), domain_id, 1)
                    .unwrap()
                    .calc_actual_weight(&dispatch_info())
            )
        });
    }
}

mod register_domain {
    use super::*;

    #[test]
    fn add_a_domain_with_the_given_values() {
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                16,
                Some(8),
                AggregateSecurityRules::OnlyOwnerUncompleted,
                ProofSecurityRules::OnlyAllowlisted,
                priced_none_delivering(1234, 12),
                Some(USER_DOMAIN_2)
            ));
            let registered_id = registered_ids()[0];

            let domain = Domains::<Test>::get(registered_id).unwrap();

            assert_eq!(registered_id, domain.id);
            assert_eq!(16, domain.max_aggregation_size);
            assert_eq!(8, domain.publish_queue_size);
            assert_eq!(
                AggregateSecurityRules::OnlyOwnerUncompleted,
                domain.aggregate_rules
            );
            assert_eq!(ProofSecurityRules::OnlyAllowlisted, domain.proof_rules);
            assert_eq!(
                DeliveryParams::<AccountId, Balance>::new(
                    USER_DOMAIN_2,
                    Delivery::new(none_destination(), 1234, 12)
                ),
                domain.delivery
            );

            assert_eq!(domain.next, Aggregation::<Test>::create(1, 16));
            assert!(domain.should_publish.is_empty());
        })
    }

    #[test]
    fn normal_user_that_not_provide_delivery_owner_become_the_owner() {
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                16,
                Some(8),
                AggregateSecurityRules::OnlyOwnerUncompleted,
                ProofSecurityRules::Untrusted,
                priced_none_delivering(1234, 12),
                None
            ));
            let registered_id = registered_ids()[0];

            let domain = Domains::<Test>::get(registered_id).unwrap();

            assert_eq!(USER_DOMAIN_1, domain.delivery.owner);
        })
    }

    #[test]
    fn manager_can_add_a_domain_with_a_bridge_domain() {
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(ROOT_USER).into(),
                16,
                Some(8),
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::Untrusted,
                hyperbridge_destination().into(),
                Some(USER_DOMAIN_2)
            ));
            let registered_id = registered_ids()[0];

            let domain = Domains::<Test>::get(registered_id).unwrap();

            assert_eq!(&hyperbridge_destination(), domain.delivery.destination());
            assert_eq!(USER_DOMAIN_2, domain.delivery.owner);
        })
    }

    #[test]
    fn normal_users_cannot_add_a_domain_with_a_bridge_domain() {
        test().execute_with(|| {
            assert_noop!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    16,
                    Some(8),
                    AggregateSecurityRules::OnlyOwner,
                    ProofSecurityRules::Untrusted,
                    hyperbridge_destination().into(),
                    None,
                ),
                BadOrigin
            );
        })
    }

    #[test]
    fn add_more_domains() {
        test().execute_with(|| {
            let values = [(8, Some(4)), (16, None), (32, Some(8))];
            let delivery_users = [USER_DOMAIN_1, USER_2, USER_DOMAIN_2];
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                values[0].0,
                values[0].1,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::Untrusted,
                priced_none_delivering(4321, 43),
                None
            ));
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                values[1].0,
                values[1].1,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::Untrusted,
                priced_none_delivering(4331, 43),
                Some(delivery_users[1])
            ));
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                values[2].0,
                values[2].1,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::Untrusted,
                priced_none_delivering(4341, 43),
                Some(delivery_users[2])
            ));

            let registered_ids = registered_ids();

            // Sequentially ids
            for (prev, next) in registered_ids.iter().zip(registered_ids.iter().skip(1)) {
                assert_eq!(prev + 1, *next)
            }

            for (pos, id) in registered_ids.into_iter().enumerate() {
                let domain = Domains::<Test>::get(id).unwrap();
                let aggregation_size = values[pos].0;
                let queue_size = values[pos]
                    .1
                    .unwrap_or_else(<Test as Config>::MaxPendingPublishQueueSize::get);
                assert_eq!(id, domain.id);
                assert_eq!(aggregation_size, domain.max_aggregation_size);
                assert_eq!(queue_size, domain.publish_queue_size);
                assert_eq!(&none_destination(), domain.delivery.destination());
                assert_eq!(4321 + (pos as u128) * 10, *domain.delivery.fee());
                assert_eq!(delivery_users[pos], domain.delivery.owner);

                assert_eq!(
                    domain.next,
                    Aggregation::<Test>::create(1, aggregation_size)
                );
                assert!(domain.should_publish.is_empty());
            }
        })
    }

    #[test]
    fn fail_if_wrong_configuration_params() {
        test().execute_with(|| {
            // Sanity check
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                MaxAggregationSize::get(),
                Some(MaxPendingPublishQueueSize::get()),
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::Untrusted,
                none_delivering(),
                None
            ));

            assert_err!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    0,
                    Some(MaxPendingPublishQueueSize::get()),
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    None
                ),
                Error::<Test>::InvalidDomainParams
            );
            assert_err!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    MaxAggregationSize::get() + 1,
                    Some(MaxPendingPublishQueueSize::get()),
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    None
                ),
                Error::<Test>::InvalidDomainParams
            );
            assert_err!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    MaxAggregationSize::get(),
                    Some(MaxPendingPublishQueueSize::get() + 1),
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    None
                ),
                Error::<Test>::InvalidDomainParams
            );
        })
    }

    #[test]
    fn fail_if_manager_try_to_register_domain_without_providing_any_delivery_owner() {
        test().execute_with(|| {
            assert_noop!(
                Aggregate::register_domain(
                    Origin::Signed(ROOT_USER).into(),
                    0,
                    Some(MaxPendingPublishQueueSize::get()),
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    None
                ),
                Error::<Test>::MissedDeliveryOwnership
            );
        })
    }

    #[test]
    fn save_consideration_tickets_if_user_register_a_domain() {
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                16,
                None,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::OnlyAllowlisted,
                none_delivering(),
                None
            ));

            let domain = Domains::<Test>::get(registered_ids()[0]).unwrap();

            assert_eq!(
                domain.ticket_domain,
                Some(
                    MockConsideration {
                        who: USER_DOMAIN_1,
                        count: 1,
                        size: Domain::<Test>::compute_encoded_size(
                            16,
                            MaxPendingPublishQueueSize::get(),
                            &none_destination(),
                        ) as u64,
                    }
                    .into()
                ),
            );

            assert_eq!(
                domain.ticket_allowlist,
                Some(countable_mock_consideration(USER_DOMAIN_1, 0, 0)),
            );
        });
    }

    #[test]
    fn donst_store_consideration_tickets_if_manager_register_domain() {
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(ROOT_USER).into(),
                16,
                None,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::OnlyAllowlisted,
                none_delivering(),
                Some(USER_DOMAIN_1)
            ));

            let domain = Domains::<Test>::get(registered_ids()[0]).unwrap();

            assert_eq!(None, domain.ticket_domain);
            assert_eq!(None, domain.ticket_allowlist);
        });
    }

    #[rstest]
    fn donst_store_allowlist_consideration_ticket_if_doesnt_set_allowlist_for_domain(
        #[values(ProofSecurityRules::Untrusted, ProofSecurityRules::OnlyOwner)]
        proof_rules: ProofSecurityRules,
    ) {
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                16,
                None,
                AggregateSecurityRules::Untrusted,
                proof_rules,
                none_delivering(),
                Some(USER_DOMAIN_1)
            ));

            let domain = Domains::<Test>::get(registered_ids()[0]).unwrap();

            assert_eq!(None, domain.ticket_allowlist);
        });
    }

    #[rstest]
    #[case(hyperbridge_destination(), (78855, 1743, 9405, 21079))]
    #[case(Destination::None, (78822, 1710, 9372, 21046))]
    fn not_change_domain_encoded_size(
        #[case] destination: Destination,
        #[case] variables: (usize, usize, usize, usize),
    ) {
        let (bigger, min_agg_max_queue, max_agg_min_queue, middle) = variables;
        // This test is here to check that you don't change the domain struct without change `compute_encoded_size`
        // accordantly
        use codec::MaxEncodedLen;
        // Check base: always TRUE
        assert_eq!(
            Domain::<Test>::max_encoded_len(),
            Domain::<Test>::compute_encoded_size(
                MaxAggregationSize::get(),
                MaxPendingPublishQueueSize::get(),
                &hyperbridge_destination()
            )
        );

        // Fixture max
        assert_eq!(Domain::<Test>::max_encoded_len(), 78855);

        // Max configurations
        assert_eq!(
            bigger,
            Domain::<Test>::compute_encoded_size(
                MaxAggregationSize::get(),
                MaxPendingPublishQueueSize::get(),
                &destination
            )
        );

        // Fixtures
        assert_eq!(
            min_agg_max_queue,
            Domain::<Test>::compute_encoded_size(
                1,
                MaxPendingPublishQueueSize::get(),
                &destination
            )
        );
        assert_eq!(
            max_agg_min_queue,
            Domain::<Test>::compute_encoded_size(MaxAggregationSize::get(), 1, &destination)
        );
        assert_eq!(
            middle,
            Domain::<Test>::compute_encoded_size(
                MaxAggregationSize::get() / 2,
                MaxPendingPublishQueueSize::get() / 2,
                &destination
            )
        );
    }

    #[test]
    fn rise_error_on_if_new_consideration_fails() {
        test().execute_with(|| {
            assert_err!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_ERROR_NEW).into(),
                    16,
                    None,
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    None
                ),
                sp_runtime::DispatchError::from("User Domain Error New")
            );
        })
    }

    #[test]
    fn apply_fee() {
        test().execute_with(|| {
            assert_eq!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    16,
                    None,
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    None
                )
                .unwrap()
                .pays_fee,
                Pays::Yes
            );
        })
    }

    #[test]
    fn don_t_apply_fee_to_manager() {
        test().execute_with(|| {
            assert_eq!(
                Aggregate::register_domain(
                    Origin::Signed(ROOT_USER).into(),
                    16,
                    None,
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    hyperbridge_destination().into(),
                    Some(USER_DOMAIN_2)
                )
                .unwrap()
                .pays_fee,
                Pays::No
            );
        })
    }

    #[test]
    fn use_correct_weight() {
        let info = Call::<Test>::register_domain {
            aggregation_size: 16,
            queue_size: Some(8),
            aggregate_rules: AggregateSecurityRules::Untrusted,
            proof_rules: ProofSecurityRules::Untrusted,
            delivery: hyperbridge_destination().into(),
            delivery_owner: None,
        }
        .get_dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(info.call_weight, MockWeightInfo::register_domain());
    }

    #[test]
    fn no_more_domain_ids() {
        test().execute_with(|| {
            NextDomainId::<Test>::put(u32::MAX);
            assert_noop!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    16,
                    None,
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    None
                ),
                Error::<Test>::InvalidDomainParams
            );
        })
    }
}

mod hold_domain {
    use super::*;

    mod put_the_domain_in_right_state {
        use super::*;

        mod hold {
            use super::*;

            #[test]
            fn if_there_are_some_statements_in_next_aggregation() {
                test().execute_with(|| {
                    Aggregate::on_proof_verified(Some(USER_1), DOMAIN, Default::default());

                    assert_ok!(Aggregate::hold_domain(
                        Origin::Signed(USER_DOMAIN_1).into(),
                        DOMAIN_ID
                    ));

                    let domain = Domains::<Test>::get(DOMAIN_ID).unwrap();

                    assert_eq!(DomainState::Hold, domain.state);
                    assert_state_changed_evt(DOMAIN_ID, DomainState::Hold);
                })
            }

            #[test]
            fn if_there_are_submitter_in_the_allowlist() {
                test().execute_with(|| {
                    assert_ok!(Aggregate::hold_domain(
                        Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                        DOMAIN_ID_ALLOWLISTED
                    ));

                    let domain = Domains::<Test>::get(DOMAIN_ID_ALLOWLISTED).unwrap();
                    // Sanity check
                    assert!(domain.next.statements.is_empty());

                    assert_eq!(DomainState::Hold, domain.state);
                    assert_state_changed_evt(DOMAIN_ID_ALLOWLISTED, DomainState::Hold);
                })
            }

            #[test]
            fn if_is_configured_for_allowlisted_only_but_there_aren_t_any() {
                test().execute_with(|| {
                    let _ = SubmittersAllowlist::<Test>::clear_prefix(
                        DOMAIN_ID_ALLOWLISTED,
                        1_000,
                        None,
                    );

                    assert_ok!(Aggregate::hold_domain(
                        Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                        DOMAIN_ID_ALLOWLISTED
                    ));

                    let domain = Domains::<Test>::get(DOMAIN_ID_ALLOWLISTED).unwrap();

                    assert_eq!(DomainState::Removable, domain.state);
                    assert_state_changed_evt(DOMAIN_ID_ALLOWLISTED, DomainState::Removable);
                })
            }

            #[test]
            fn if_there_are_some_aggregation_in_publish_queue_but_no_statements_in_the_next_aggregation(
            ) {
                test().execute_with(|| {
                    for _ in 0..DOMAIN_SIZE {
                        Aggregate::on_proof_verified(Some(USER_1), DOMAIN, Default::default());
                    }
                    assert_ok!(Aggregate::hold_domain(
                        Origin::Signed(USER_DOMAIN_1).into(),
                        DOMAIN_ID
                    ));

                    let domain = Domains::<Test>::get(DOMAIN_ID).unwrap();
                    // Sanity check
                    assert!(domain.next.statements.is_empty());

                    assert_eq!(DomainState::Hold, domain.state);
                    assert_state_changed_evt(DOMAIN_ID, DomainState::Hold);
                })
            }
        }

        mod removable {
            use super::*;

            #[test]
            fn if_there_aren_t_any_statement_in_next_aggregation_and_any_aggregation_in_should_publishing_queue(
            ) {
                test().execute_with(|| {
                    assert_ok!(Aggregate::hold_domain(
                        Origin::Signed(USER_DOMAIN_1).into(),
                        DOMAIN_ID
                    ));

                    let domain = Domains::<Test>::get(DOMAIN_ID).unwrap();
                    // Sanity check
                    assert!(domain.next.statements.is_empty());
                    assert!(domain.should_publish.is_empty());

                    assert_eq!(DomainState::Removable, domain.state);
                    assert_state_changed_evt(DOMAIN_ID, DomainState::Removable);
                })
            }
        }
    }

    mod raise_error_if {
        use super::*;

        #[test]
        fn invalid_domain() {
            test().execute_with(|| {
                assert_err!(
                    Aggregate::hold_domain(
                        Origin::Signed(USER_DOMAIN_1).into(),
                        NOT_REGISTERED_DOMAIN_ID
                    ),
                    Error::<Test>::UnknownDomainId
                );
            })
        }

        #[test]
        fn if_the_issuer_is_not_the_owner() {
            test().execute_with(|| {
                assert_err!(
                    Aggregate::hold_domain(Origin::Signed(USER_DOMAIN_2).into(), DOMAIN_ID),
                    BadOrigin
                );

                let id = register_domain(
                    USER_DOMAIN_2,
                    16,
                    None,
                    AggregateSecurityRules::Untrusted,
                    ProofSecurityRules::Untrusted,
                    none_delivering(),
                    None,
                );

                assert_err!(
                    Aggregate::hold_domain(Origin::Signed(USER_DOMAIN_1).into(), id),
                    BadOrigin
                );
            })
        }

        #[rstest]
        fn the_domain_is_not_in_valid_state(
            #[values(DomainState::Hold, DomainState::Removable, DomainState::Removed)]
            state: DomainState,
        ) {
            test().execute_with(|| {
                Domains::<Test>::mutate_extant(DOMAIN_ID, |d| {
                    d.state = state;
                });

                assert_err!(
                    Aggregate::hold_domain(Origin::Signed(USER_DOMAIN_1).into(), DOMAIN_ID),
                    Error::<Test>::InvalidDomainState
                );
                assert!(Domains::<Test>::get(DOMAIN_ID).is_some());
            })
        }
    }
}

mod handle_the_hold_state_transactions {

    use super::*;

    #[test]
    fn when_aggregate_all_aggregation_in_should_publish_queue_move_to_removable_state() {
        test().execute_with(|| {
            let aggregates = DOMAIN_QUEUE_SIZE / 2;
            for _ in 0..(DOMAIN_SIZE * aggregates) {
                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, Default::default());
            }

            assert_ok!(Aggregate::hold_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                DOMAIN_ID
            ));
            // Sanity Check
            assert_state_changed_evt(DOMAIN_ID, DomainState::Hold);
            System::reset_events();

            for id in 0..(aggregates - 1) {
                Aggregate::aggregate(Origin::Signed(USER_1).into(), DOMAIN_ID, id as u64 + 1)
                    .unwrap();
                assert_no_state_changed_evt();
            }

            Aggregate::aggregate(Origin::Signed(USER_1).into(), DOMAIN_ID, aggregates as u64)
                .unwrap();

            let domain = Domains::<Test>::get(DOMAIN_ID).unwrap();

            assert_eq!(DomainState::Removable, domain.state);
            assert_state_changed_evt(DOMAIN_ID, DomainState::Removable);
        })
    }

    #[test]
    fn when_remove_all_elements_in_submitter_list_with_remove_proof_submitters() {
        test().execute_with(|| {
            let submitters = SubmittersAllowlist::<Test>::iter_key_prefix(DOMAIN_ID_ALLOWLISTED)
                .collect::<Vec<_>>();

            assert_ok!(Aggregate::hold_domain(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED
            ));
            let domain = Domains::<Test>::get(DOMAIN_ID_ALLOWLISTED).unwrap();
            // Sanity check
            assert_eq!(DomainState::Hold, domain.state);

            assert_ok!(Aggregate::remove_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                submitters,
            ));

            assert_state_changed_evt(DOMAIN_ID_ALLOWLISTED, DomainState::Removable);
        })
    }
}

mod unregister_domain {
    use super::*;
    use sp_core::Get;

    fn test() -> sp_io::TestExternalities {
        let mut ext = super::test();
        ext.execute_with(|| {
            Domains::<Test>::mutate_extant(DOMAIN_ID, |d| {
                d.state = DomainState::Removable;
            });
        });
        ext
    }

    fn register_removable_domain(user: AccountId, proof_rules: Option<ProofSecurityRules>) -> u32 {
        let id = register_domain(
            user,
            16,
            None,
            AggregateSecurityRules::Untrusted,
            proof_rules.unwrap_or(ProofSecurityRules::Untrusted),
            none_delivering(),
            None,
        );
        Domains::<Test>::mutate_extant(id, |d| {
            d.state = DomainState::Removable;
        });
        id
    }

    #[rstest]
    #[case::owner(USER_DOMAIN_1)]
    #[case::manager(ROOT_USER)]
    fn remove_the_domain_if_valid_use(#[case] user: AccountId) {
        test().execute_with(|| {
            assert_ok!(Aggregate::unregister_domain(
                Origin::Signed(user).into(),
                DOMAIN_ID
            ));

            assert!(Domains::<Test>::get(DOMAIN_ID).is_none());
        })
    }

    mod raise_error_if {
        use super::*;

        #[test]
        fn invalid_domain() {
            test().execute_with(|| {
                assert_err!(
                    Aggregate::unregister_domain(
                        Origin::Signed(USER_DOMAIN_1).into(),
                        NOT_REGISTERED_DOMAIN_ID
                    ),
                    Error::<Test>::UnknownDomainId
                );
            })
        }

        #[test]
        fn if_the_issuer_is_not_the_owner() {
            test().execute_with(|| {
                assert_err!(
                    Aggregate::unregister_domain(Origin::Signed(USER_DOMAIN_2).into(), DOMAIN_ID),
                    BadOrigin
                );

                let id = register_removable_domain(USER_DOMAIN_2, None);

                assert_err!(
                    Aggregate::unregister_domain(Origin::Signed(USER_DOMAIN_1).into(), id),
                    BadOrigin
                );
            })
        }

        #[rstest]
        fn the_domain_is_not_in_valid_state(
            #[values(DomainState::Ready, DomainState::Hold, DomainState::Removed)]
            state: DomainState,
        ) {
            test().execute_with(|| {
                Domains::<Test>::mutate_extant(DOMAIN_ID, |d| {
                    d.state = state;
                });

                assert_err!(
                    Aggregate::unregister_domain(Origin::Signed(USER_DOMAIN_1).into(), DOMAIN_ID),
                    Error::<Test>::InvalidDomainState
                );
                assert!(Domains::<Test>::get(DOMAIN_ID).is_some());
            })
        }
    }

    #[test]
    fn unregister_domain_drop_consideration_tickets() {
        let origin = Origin::Signed(USER_DOMAIN_1);
        test().execute_with(|| {
            let id =
                register_removable_domain(USER_DOMAIN_1, Some(ProofSecurityRules::OnlyAllowlisted));

            assert_ok!(Aggregate::unregister_domain(origin.into(), id));

            let (id, dropped_consideration) =
                MockConsideration::pop(MockHoldDomain::get()).unwrap();

            assert_eq!(USER_DOMAIN_1, id);
            assert_eq!(USER_DOMAIN_1, dropped_consideration.who);

            let (id, dropped_consideration) =
                MockConsideration::pop(MockHoldAllowlist::get()).unwrap();

            assert_eq!(USER_DOMAIN_1, id);
            assert_eq!(USER_DOMAIN_1, dropped_consideration.who);
        })
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic(expected = "Drop"))]
    fn ignore_error_on_drop_ticket_but_defensive_proof_on_test() {
        let origin = Origin::Signed(USER_DOMAIN_ERROR_DROP);
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                origin.clone().into(),
                16,
                None,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::Untrusted,
                none_delivering(),
                None
            ));

            let id = registered_ids()[0];

            Domains::<Test>::mutate_extant(id, |d| {
                d.state = DomainState::Removable;
            });

            Aggregate::unregister_domain(origin.into(), id).unwrap();
        })
    }

    #[test]
    fn apply_fee() {
        test().execute_with(|| {
            assert_eq!(
                Aggregate::unregister_domain(Origin::Signed(USER_DOMAIN_1).into(), DOMAIN_ID)
                    .unwrap()
                    .pays_fee,
                Pays::Yes
            );
        })
    }

    #[test]
    fn don_t_apply_fee_to_manager() {
        test().execute_with(|| {
            assert_eq!(
                Aggregate::unregister_domain(Origin::Signed(ROOT_USER).into(), DOMAIN_ID)
                    .unwrap()
                    .pays_fee,
                Pays::No
            );
        })
    }

    #[test]
    fn use_correct_weight() {
        let info = Call::<Test>::unregister_domain { domain_id: 22 }.get_dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(info.call_weight, MockWeightInfo::unregister_domain());
    }
}

mod allowlist_proof_submitters {
    use super::*;

    #[test]
    fn add_a_list_of_users() {
        test().execute_with(|| {
            let allowlisted = SubmittersAllowlist::<Test>::iter().count();
            let to_allowlist = vec![USER_1, USER_2, USER_DELIVERY_OWNER];

            assert_ok!(Aggregate::allowlist_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                to_allowlist.clone()
            ));
            assert!(SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_1
            ));
            assert!(SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_2
            ));
            assert!(SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_DELIVERY_OWNER
            ));
            assert_eq!(
                SubmittersAllowlist::<Test>::iter().count(),
                allowlisted + to_allowlist.len()
            );
        })
    }

    #[test]
    fn not_add_duplicate() {
        test().execute_with(|| {
            let allowlisted = SubmittersAllowlist::<Test>::iter().count();
            let to_allowlist = vec![USER_1, USER_1, USER_1];

            assert_ok!(Aggregate::allowlist_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                to_allowlist.clone()
            ));
            assert!(SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_1
            ));
            assert_eq!(SubmittersAllowlist::<Test>::iter().count(), allowlisted + 1);
        })
    }

    #[test]
    fn fail_if_domain_is_not_configured_for_submitters_allowlist() {
        test().execute_with(|| {
            assert_noop!(
                Aggregate::allowlist_proof_submitters(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    DOMAIN_ID,
                    vec![USER_1]
                ),
                Error::<Test>::InvalidDomainParams
            );

            assert_noop!(
                Aggregate::allowlist_proof_submitters(
                    Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                    DOMAIN_ID_ONLY_OWNER,
                    vec![USER_1]
                ),
                Error::<Test>::InvalidDomainParams
            );
        })
    }

    #[test]
    fn bound_amount_for_each_allowlisted_user() {
        // Here we check that the count provided to the consideration increases accordingly to the
        // number of added users.
        test().execute_with(|| {
            let to_allowlist = vec![USER_1, USER_2, USER_1];
            let base = consideration(DOMAIN_ID_ALLOWLISTED).unwrap();

            assert_ok!(Aggregate::allowlist_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                to_allowlist.clone()
            ));

            let updated = consideration(DOMAIN_ID_ALLOWLISTED).unwrap();

            assert_eq!(base.count + 2, updated.count);
            assert_eq!(base.size, updated.size);
            assert_eq!(base.who, updated.who);
        })
    }

    #[test]
    fn raise_error_if_saturated() {
        test().execute_with(|| {
            let to_allowlist = vec![USER_1];
            Domains::<Test>::mutate(DOMAIN_ID_ALLOWLISTED, |d| match d {
                Some(domain) => {
                    domain.ticket_allowlist =
                        Some(countable_mock_consideration(USER_1, u32::MAX, 1));
                }
                None => {
                    panic!("Domain not found");
                }
            });
            assert_noop!(
                Aggregate::allowlist_proof_submitters(
                    Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                    DOMAIN_ID_ALLOWLISTED,
                    to_allowlist.clone()
                ),
                Error::<Test>::InvalidDomainParams
            );
        })
    }

    #[test]
    fn not_add_submitters_if_the_domain_is_in_hold_or_removable_state() {
        test().execute_with(|| {
            Aggregate::hold_domain(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
            )
            .unwrap();

            // Sanity check
            assert_eq!(DomainState::Hold, state(DOMAIN_ID_ALLOWLISTED));

            // Check the Hold state should not accept new submitters
            assert_noop!(
                Aggregate::allowlist_proof_submitters(
                    Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                    DOMAIN_ID_ALLOWLISTED,
                    vec![USER_1]
                ),
                Error::<Test>::InvalidDomainParams
            );

            Aggregate::remove_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                vec![USER_ALLOWLISTED_1, USER_ALLOWLISTED_2, USER_ALLOWLISTED_3],
            )
            .unwrap();

            // Sanity check
            assert_eq!(DomainState::Removable, state(DOMAIN_ID_ALLOWLISTED));

            // Check the Removable state should not accept new submitters
            assert_noop!(
                Aggregate::allowlist_proof_submitters(
                    Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                    DOMAIN_ID_ALLOWLISTED,
                    vec![USER_ALLOWLISTED_1]
                ),
                Error::<Test>::InvalidDomainParams
            );
        })
    }

    #[rstest]
    fn use_correct_weight(#[values(0, 3, 10)] len: u32) {
        let info = Call::<Test>::allowlist_proof_submitters {
            domain_id: 22,
            submitters: (0..len).map(|i| 1_000_000 + i).map(Into::into).collect(),
        }
        .get_dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(
            info.call_weight,
            MockWeightInfo::allowlist_proof_submitters(len)
        );
    }
}

mod remove_proof_submitters {
    use super::*;

    #[test]
    fn remove_a_list_of_users() {
        test().execute_with(|| {
            let allowlisted = SubmittersAllowlist::<Test>::iter().count();
            let to_remove = vec![USER_ALLOWLISTED_1, USER_ALLOWLISTED_3];

            assert_ok!(Aggregate::remove_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                to_remove.clone()
            ));
            assert!(!SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_ALLOWLISTED_1
            ));
            assert!(!SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_ALLOWLISTED_3
            ));
            // Check: Not removed
            assert!(SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_ALLOWLISTED_2
            ));
            assert_eq!(
                SubmittersAllowlist::<Test>::iter().count(),
                allowlisted - to_remove.len()
            );
        })
    }

    #[test]
    fn not_add_remove_duplicates_or_not_present() {
        test().execute_with(|| {
            let allowlisted = SubmittersAllowlist::<Test>::iter().count();
            let to_remove = vec![
                USER_ALLOWLISTED_1,
                USER_1,
                USER_ALLOWLISTED_1,
                USER_ALLOWLISTED_2,
            ];

            assert_ok!(Aggregate::remove_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                to_remove.clone()
            ));
            assert!(!SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_ALLOWLISTED_1
            ));
            assert!(!SubmittersAllowlist::<Test>::contains_key(
                DOMAIN_ID_ALLOWLISTED,
                USER_ALLOWLISTED_2
            ));

            assert_eq!(SubmittersAllowlist::<Test>::iter().count(), allowlisted - 2);
        })
    }

    #[test]
    fn fail_if_domain_is_not_configured_for_submitters_allowlist() {
        test().execute_with(|| {
            assert_noop!(
                Aggregate::remove_proof_submitters(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    DOMAIN_ID,
                    vec![USER_1]
                ),
                Error::<Test>::InvalidDomainParams
            );

            assert_noop!(
                Aggregate::remove_proof_submitters(
                    Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                    DOMAIN_ID_ONLY_OWNER,
                    vec![USER_1]
                ),
                Error::<Test>::InvalidDomainParams
            );
        })
    }

    #[test]
    fn unbound_amount_for_each_removed_user() {
        // Here we check that the count provided to the consideration decreases accordingly to the
        // number of removed users.
        test().execute_with(|| {
            let to_remove = vec![USER_ALLOWLISTED_1, USER_ALLOWLISTED_2];
            let base = consideration(DOMAIN_ID_ALLOWLISTED).unwrap();

            assert_ok!(Aggregate::remove_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                to_remove.clone()
            ));

            let updated = consideration(DOMAIN_ID_ALLOWLISTED).unwrap();

            assert_eq!(base.count - 2, updated.count);
            assert_eq!(base.size, updated.size);
            assert_eq!(base.who, updated.who);
        })
    }

    #[test]
    fn raise_error_if_removed_more() {
        // That should never happen in real cases: it's just a bug
        test().execute_with(|| {
            let to_remove = vec![USER_ALLOWLISTED_1, USER_ALLOWLISTED_2];
            Domains::<Test>::mutate(DOMAIN_ID_ALLOWLISTED, |d| match d {
                Some(domain) => {
                    domain.ticket_allowlist = Some(countable_mock_consideration(USER_1, 1, 1));
                }
                None => {
                    panic!("Domain not found");
                }
            });
            assert_noop!(
                Aggregate::remove_proof_submitters(
                    Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                    DOMAIN_ID_ALLOWLISTED,
                    to_remove.clone()
                ),
                Error::<Test>::InvalidDomainParams
            );
        })
    }

    #[rstest]
    #[case::one(vec![USER_ALLOWLISTED_1])]
    #[case::all(vec![USER_ALLOWLISTED_1, USER_ALLOWLISTED_2, USER_ALLOWLISTED_3])]
    fn not_change_the_domain_ready_state(#[case] to_remove: Vec<AccountId>) {
        test().execute_with(|| {
            assert_ok!(Aggregate::remove_proof_submitters(
                Origin::Signed(USER_DOMAIN_SUBMIT_RULE).into(),
                DOMAIN_ID_ALLOWLISTED,
                to_remove.clone()
            ));

            assert_no_state_changed_evt();
        })
    }

    #[rstest]
    fn use_correct_weight(#[values(0, 3, 10)] len: u32) {
        let info = Call::<Test>::remove_proof_submitters {
            domain_id: 22,
            submitters: (0..len).map(|i| 1_000_000 + i).map(Into::into).collect(),
        }
        .get_dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(
            info.call_weight,
            MockWeightInfo::remove_proof_submitters(len)
        );
    }
}

mod get_statement_path {
    use super::*;

    fn test() -> sp_io::TestExternalities {
        let mut ext = super::test();

        let mut a = Aggregation::<Test>::create(123, 16);
        (0..16_u64).for_each(|i| {
            a.add_statement(USER_1, Default::default(), H256::from_low_u64_be(i as u64))
        });

        ext.execute_with(|| {
            Published::<Test>::mutate(|p: &mut _| p.push((DOMAIN_ID, a)));
        });
        ext
    }

    #[test]
    fn return_a_valid_merkle_path_for_a_published_statement() {
        test().execute_with(|| {
            for i in 0..16 {
                let proof =
                    Aggregate::get_statement_path(DOMAIN_ID, 123, H256::from_low_u64_be(i as u64))
                        .unwrap();

                assert!(binary_merkle_tree::verify_proof::<Keccak256, _, _>(
                    &proof.root,
                    proof.proof,
                    proof.number_of_leaves,
                    proof.leaf_index,
                    &proof.leaf
                ))
            }
        })
    }

    #[test]
    fn return_a_receipt_not_published_error_if_wrong_domain_id() {
        test().execute_with(|| {
            assert_eq!(
                PathRequestError::ReceiptNotPublished(939, 123),
                Aggregate::get_statement_path(939, 123, H256::from_low_u64_be(5)).unwrap_err()
            );
        })
    }

    #[test]
    fn return_a_receipt_not_published_error_if_wrong_aggregation_id() {
        test().execute_with(|| {
            assert_eq!(
                PathRequestError::ReceiptNotPublished(DOMAIN_ID, 42),
                Aggregate::get_statement_path(DOMAIN_ID, 42, H256::from_low_u64_be(5)).unwrap_err()
            );
        })
    }

    #[test]
    fn return_a_not_found_error_if_wrong_statement_requested() {
        let statement = H256::from_low_u64_be(4323);
        test().execute_with(|| {
            assert_eq!(
                PathRequestError::NotFound(DOMAIN_ID, 123, statement),
                Aggregate::get_statement_path(DOMAIN_ID, 123, statement).unwrap_err()
            );
        })
    }
}

mod set_total_delivery_fee {
    use super::*;

    #[rstest]
    #[case::domain_owner(USER_DOMAIN_1)]
    #[case::manager(ROOT_USER)]
    #[case::delivery_owner(USER_DELIVERY_OWNER)]
    fn should_set_the_correct_total_delivery_fee(#[case] issuer: AccountId) {
        test().execute_with(|| {
            assert_ok!(Aggregate::set_total_delivery_fee(
                Origin::Signed(issuer).into(),
                DOMAIN_ID,
                123456,
                123
            ));

            assert_eq!(
                Domains::<Test>::get(DOMAIN_ID).unwrap().delivery.fee(),
                &123456
            );

            assert_ok!(Aggregate::set_total_delivery_fee(
                Origin::Signed(issuer).into(),
                DOMAIN_ID,
                654321,
                654
            ));

            assert_eq!(
                Domains::<Test>::get(DOMAIN_ID).unwrap().delivery.fee(),
                &654321
            );
        })
    }

    #[rstest]
    #[case::unauthorized_issuer(USER_DOMAIN_2, DOMAIN_ID, sp_runtime::DispatchError::BadOrigin)]
    #[case::invalid_domain_id(ROOT_USER, NOT_REGISTERED_DOMAIN_ID, Error::<Test>::UnknownDomainId)]
    fn should_fail(
        #[case] issuer: AccountId,
        #[case] domain_id: u32,
        #[case] error: impl Into<sp_runtime::DispatchError>,
    ) {
        test().execute_with(|| {
            assert_err!(
                Aggregate::set_total_delivery_fee(
                    Origin::Signed(issuer).into(),
                    domain_id,
                    123456,
                    123
                ),
                error
            );
        })
    }
}

#[test]
fn return_the_correct_weigh_on_proof_verified() {
    assert_eq!(
        <Aggregate as OnProofVerified<u64>>::weight(&None),
        Weight::default()
    );
    assert_eq!(
        <Aggregate as OnProofVerified<u64>>::weight(&Some(42)),
        <Test as crate::Config>::WeightInfo::on_proof_verified()
    );
}

mod aggregation_id_max {

    use frame_support::assert_err_ignore_postinfo;

    use super::*;

    const MAX_AGGREGATE_ID: u64 = u64::MAX;

    #[test]
    fn hold_on_aggregate() {
        test().execute_with(|| {
            let domain = Domain::<Test>::try_create(
                DOMAIN_ID,
                USER_DOMAIN_1.into(),
                MAX_AGGREGATE_ID,
                2,
                1,
                AggregateSecurityRules::OnlyOwnerUncompleted,
                ProofSecurityRules::Untrusted,
                None,
                None,
                DeliveryParams::<AccountId, Balance>::new(
                    USER_DOMAIN_1,
                    Delivery::new(none_destination(), 1234, 12),
                ),
            )
            .unwrap();
            Domains::<Test>::insert(DOMAIN_ID, domain);

            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);
            assert_ok!(Aggregate::aggregate(
                Origin::Signed(USER_DOMAIN_1).into(),
                DOMAIN_ID,
                MAX_AGGREGATE_ID
            ));

            assert_state_changed_evt(DOMAIN_ID, DomainState::Hold);

            // Note: Domain contains only one aggregation. Once published, and in hold, state is automatically switched to Removable.
            assert_state_changed_evt(DOMAIN_ID, DomainState::Removable);

            // Not possible to submit new proofs/call aggregate on this domain
            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);
            assert_cannot_aggregate_evt(
                statement,
                CannotAggregateCause::InvalidDomainState {
                    domain_id: DOMAIN_ID,
                    state: DomainState::Removable,
                },
            );

            assert_err_ignore_postinfo!(
                Aggregate::aggregate(Origin::Signed(USER_DOMAIN_1).into(), DOMAIN_ID, 0),
                Error::<Test>::InvalidAggregationId
            );
        })
    }

    #[test]
    fn hold_on_on_proof_verified() {
        test().execute_with(|| {
            let domain = Domain::<Test>::try_create(
                DOMAIN_ID,
                USER_DOMAIN_1.into(),
                MAX_AGGREGATE_ID,
                1,
                1,
                AggregateSecurityRules::OnlyOwnerUncompleted,
                ProofSecurityRules::Untrusted,
                None,
                None,
                DeliveryParams::<AccountId, Balance>::new(
                    USER_DOMAIN_1,
                    Delivery::new(none_destination(), 1234, 12),
                ),
            )
            .unwrap();
            Domains::<Test>::insert(DOMAIN_ID, domain);

            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(USER_DOMAIN_1), Some(DOMAIN_ID), statement);

            assert_eq!(
                DomainState::Hold,
                Domains::<Test>::get(DOMAIN_ID).unwrap().state
            );
            assert_state_changed_evt(DOMAIN_ID, DomainState::Hold);

            // Not possible to submit new proofs/call aggregate on this domain
            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);
            assert_cannot_aggregate_evt(
                statement,
                CannotAggregateCause::InvalidDomainState {
                    domain_id: DOMAIN_ID,
                    state: DomainState::Hold,
                },
            );

            assert_err_ignore_postinfo!(
                Aggregate::aggregate(Origin::Signed(USER_DOMAIN_1).into(), DOMAIN_ID, 0),
                Error::<Test>::InvalidAggregationId
            );
        })
    }
}
