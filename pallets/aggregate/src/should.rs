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

use super::*;
use crate::mock;
use crate::mock::*;
use frame_support::assert_ok;
use frame_support::traits::Hooks;
use frame_system::RawOrigin;
use hp_poe::OnProofVerified;
use sp_core::H256;
use sp_runtime::SaturatedConversion;
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
        assert_eq!(vec![statement_entry(USER_1, statement)], *att.statements);
    })
}

#[test]
fn emit_domain_full_event_when_publish_queue_is_full() {
    test().execute_with(|| {
        let statements = <Test as crate::Config>::MaxPendingPublishQueueSize::get()
            * <Test as crate::Config>::AggregationSize::get();
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

mod not_add_the_statement_to_any_domain_if {
    use super::*;

    #[test]
    fn no_domain_provided() {
        test().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(USER_1), None, statement);

            assert_cannot_aggregate_evt(statement, CannotAggregateCause::NoDomain);

            assert_eq!(0, count_all_statements());
        })
    }

    #[test]
    fn provided_domain_is_not_registered() {
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

mod check_if_no_room_for_new_statements_in_should_published_set_and {
    use super::*;

    const LAST_ID: u64 = 999;

    /// Fill the domain with MaxPendingPublishQueueSize::get() aggregations in should published set,
    /// and fill the next one with  AggregationSize::get()-1 statements.
    pub fn test() -> sp_io::TestExternalities {
        let mut ext = super::test();
        let size = <Test as crate::Config>::AggregationSize::get();

        ext.execute_with(|| {
            Domains::<Test>::mutate_extant(DOMAIN_ID, |d| {
                for i in 1..=<Test as crate::Config>::MaxPendingPublishQueueSize::get() as u64 {
                    d.should_publish
                        .try_insert(i, Aggregation::<Test>::create(i, size))
                        .unwrap();
                }
                d.next = Aggregation::<Test>::create(LAST_ID, size);
                for i in 0..(size - 1) {
                    d.next
                        .add_statement(USER_1, 35_u32.into(), H256::from_low_u64_be(i.into()));
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

                assert_not_ready_evt(DOMAIN_ID, LAST_ID);
            })
        }

        #[test]
        fn not_reserve_currency() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

                assert_eq!(
                    Balances::reserved_balance(USER_1),
                    0,
                    "Should not reserve any balance"
                );
            })
        }

        #[test]
        fn emit_cannot_aggregate_event() {
            test().execute_with(|| {
                let statement = H256::from_low_u64_be(123);

                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

                assert_not_ready_evt(DOMAIN_ID, LAST_ID);
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
            Aggregate::aggregate(RawOrigin::Signed(33).into(), DOMAIN_ID, 1).unwrap();
            mock::System::events().clear();

            let statement = H256::from_low_u64_be(123);
            let account = USER_1;
            Aggregate::on_proof_verified(Some(account), DOMAIN, statement);

            assert_proof_evt(DOMAIN_ID, LAST_ID, statement);
            assert_ready_evt(DOMAIN_ID, LAST_ID);
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
            Aggregate::aggregate(RawOrigin::Signed(33).into(), DOMAIN_ID, 1).unwrap();
            Aggregate::aggregate(RawOrigin::Signed(33).into(), DOMAIN_ID, 3).unwrap();
            Aggregate::aggregate(RawOrigin::Signed(33).into(), DOMAIN_ID, 5).unwrap();
            mock::System::events().clear();

            let statement = H256::from_low_u64_be(123);
            let event = Event::DomainFull {
                domain_id: DOMAIN_ID,
            };

            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

            assert_proof_evt(DOMAIN_ID, LAST_ID, statement);
            assert_ready_evt(DOMAIN_ID, LAST_ID);
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
        let elements = (0..<Test as crate::Config>::AggregationSize::get())
            .map(|i| statement_entry(USER_1, H256::from_low_u64_be(i.into())))
            .collect::<Vec<_>>();
        for s in elements.clone().into_iter() {
            Aggregate::on_proof_verified(Some(s.account.clone()), DOMAIN, s.statement);
        }

        assert_ready_evt(DOMAIN_ID, 1);

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

        assert_eq!(
            Balances::reserved_balance(account),
            FEE_PER_STATEMENT_CORRECTED as u128
        );
    })
}

#[test]
fn not_fail_but_raise_just_an_event_if_a_user_doesn_t_have_enough_found_to_reserve_on_on_proof_verified(
) {
    test().execute_with(|| {
        let statement = H256::from_low_u64_be(123);

        Aggregate::on_proof_verified(Some(NO_FOUND_USER), DOMAIN, statement);

        assert_eq!(
            Balances::reserved_balance(NO_FOUND_USER),
            0,
            "Should not reserve any balance"
        );
        assert_cannot_aggregate_evt(statement, CannotAggregateCause::InsufficientFound);
        assert_eq!(1, mock::System::events().len())
    })
}

mod clean_the_published_storage_on_initialize {
    use super::*;

    #[test]
    fn in_base_case() {
        test().execute_with(|| {
            assert_eq!(Published::<Test>::get().is_empty(), true);
        })
    }

    #[test]
    fn when_some_aggregations_are_present() {
        test().execute_with(|| {
            Published::<Test>::mutate(|published: &mut _| {
                published.push(Aggregation::<Test>::create(12, 3));
                published.push(Aggregation::<Test>::create(13, 3));
            });

            Aggregate::on_initialize(36);
            assert_eq!(Published::<Test>::get().is_empty(), true);
        })
    }

    #[test]
    fn and_return_the_correct_weight() {
        test().execute_with(|| {
            Published::<Test>::mutate(|published: &mut _| {
                published.push(Aggregation::<Test>::create(12, 3));
                published.push(Aggregation::<Test>::create(13, 3));
            });

            let w = Aggregate::on_initialize(36);
            assert_eq!(w, db_weights().writes(1));
            // Sanity check: w is not void
            assert_ne!(w, 0.into());
        })
    }
}

mod aggregate {
    use frame_support::assert_err;

    use super::*;

    #[test]
    fn emit_a_new_receipt() {
        test().execute_with(|| {
            for i in 0..<Test as crate::Config>::AggregationSize::get() {
                Aggregate::on_proof_verified(Some(USER_2), DOMAIN, H256::from_low_u64_be(i.into()));
            }

            assert_ok!(Aggregate::aggregate(
                RawOrigin::Signed(USER_1).into(),
                DOMAIN_ID,
                1
            ));
            assert_new_receipt(DOMAIN_ID, 1, None);
        })
    }

    #[test]
    fn refound_the_publisher_from_the_reserved_founds() {
        test().execute_with(|| {
            let accounts = [USER_1, USER_2];
            let elements = (0..(<Test as crate::Config>::AggregationSize::get() as u64))
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
                RawOrigin::Signed(PUBLISHER_USER).into(),
                DOMAIN_ID,
                1
            ));

            assert_eq!(Balances::free_balance(PUBLISHER_USER), expected_balance);
        })
    }

    #[test]
    fn raise_error_if_invalid_domain_is_used() {
        test().execute_with(|| {
            assert_err!(
                Aggregate::aggregate(
                    RawOrigin::Signed(USER_1).into(),
                    NOT_REGISTERED_DOMAIN_ID,
                    1
                ),
                Error::<Test>::UnknownDomainId
            );
        })
    }
}
