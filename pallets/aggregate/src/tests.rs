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
use crate::mock::RuntimeEvent as TestEvent;
use crate::mock::*;
use frame_support::assert_ok;
use frame_support::traits::Hooks;
use frame_system::{EventRecord, Phase, RawOrigin};
use hp_poe::OnProofVerified;
use sp_core::H256;
use sp_runtime::SaturatedConversion;

fn _assert_evt_gen(contains: bool, event: Event<Test>, context: &str) {
    let message = match contains {
        true => format!("{context} - CANNOT FIND {:?}", event),
        false => format!("{context} - FOUND {:?}", event),
    };
    assert_eq!(
        contains,
        mock::System::events().contains(&EventRecord {
            phase: Phase::Initialization,
            event: TestEvent::Aggregate(event),
            topics: vec![],
        }),
        "{message}"
    )
}

fn assert_evt(event: Event<Test>, context: &str) {
    _assert_evt_gen(true, event, context);
}

fn assert_not_evt(event: Event<Test>, context: &str) {
    _assert_evt_gen(false, event, context);
}

fn _assert_proof_evt_gen(contains: bool, domain_id: u32, id: u64, value: H256) {
    _assert_evt_gen(
        contains,
        Event::ProofVerified {
            domain_id,
            aggregation_id: id,
            statement: value,
        },
        "Search new proof",
    );
}

fn assert_proof_evt(domain_id: u32, id: u64, value: H256) {
    _assert_proof_evt_gen(true, domain_id, id, value)
}

fn assert_not_proof_evt(domain_id: u32, id: u64, value: H256) {
    _assert_proof_evt_gen(false, domain_id, id, value)
}

fn _assert_ready_evt_gen(contains: bool, domain_id: u32, id: u64) {
    _assert_evt_gen(
        contains,
        Event::ReadyToAggregate {
            domain_id,
            aggregation_id: id,
        },
        "Available aggregation",
    );
}

fn assert_ready_evt(domain_id: u32, id: u64) {
    _assert_ready_evt_gen(true, domain_id, id);
}

fn assert_not_ready_evt(domain_id: u32, id: u64) {
    _assert_ready_evt_gen(false, domain_id, id);
}

fn _assert_cannot_aggregate_evt_gen(contains: bool, statement: H256, cause: CannotAggregateCause) {
    _assert_evt_gen(
        contains,
        Event::CannotAggregate { statement, cause },
        "Cannot aggregate error",
    );
}

fn assert_cannot_aggregate_evt(statement: H256, cause: CannotAggregateCause) {
    _assert_cannot_aggregate_evt_gen(true, statement, cause);
}

fn assert_new_receipt(domain: u32, id: u64, expected_receipt: Option<H256>) {
    let matched = mock::System::events()
        .iter()
        .find(|record| {
            matches!(record.event, TestEvent::Aggregate(Event::<Test>::NewAggregationReceipt {
                    domain_id,
                    aggregation_id,
                    receipt,
                }
            ) if domain_id == domain && aggregation_id == id && expected_receipt.map(|h| h == receipt).unwrap_or(true))
        })
        .is_some();
    assert!(
        matched,
        "Cannot find aggregation receipt [{domain}-{id}]-{expected_receipt:?}"
    );
}

fn statement_entry(account: u64, statement: H256) -> StatementEntry<AccountId, Balance> {
    StatementEntry::new(account, FEE_PER_STATEMENT_CORRECTED as u128, statement)
}

fn count_all_statements() -> usize {
    Domains::<Test>::iter_values()
        .map(|d| {
            d.next.statements.iter().count()
                + d.should_publish
                    .values()
                    .map(|a| a.statements.len())
                    .sum::<usize>()
        })
        .sum()
}

#[test]
fn should_add_a_proof() {
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
fn should_emit_domain_full_event_when_publish_queue_is_full() {
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

mod should_not_add_the_statement_to_any_domain_if {
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

mod when_there_is_no_room_for_new_statements_in_should_published_set {
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

    #[test]
    fn should_not_add_any_statement() {
        test().execute_with(|| {
            let statements = count_all_statements();

            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, H256::from_low_u64_be(123));

            assert_eq!(statements, count_all_statements());
        })
    }

    #[test]
    fn should_not_emit_aggregation_event() {
        test().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

            assert_not_proof_evt(DOMAIN_ID, LAST_ID, statement);
        })
    }

    #[test]
    fn should_not_emit_queue_aggregation() {
        test().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(Some(USER_1), DOMAIN, statement);

            assert_not_ready_evt(DOMAIN_ID, LAST_ID);
        })
    }

    #[test]
    fn should_not_reserve_currency() {
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
    fn should_emit_cannot_aggregate_event() {
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

    #[test]
    fn should_free_room_for_new_aggregations_when_aggregated() {
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
    fn should_free_room_for_aggregation_when_aggregated_more_than_once() {
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
fn should_queue_a_new_aggregation_when_is_complete() {
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
fn add_a_proof_should_reserve_at_least_the_publish_proof_price_fraction() {
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
fn if_a_user_doesn_t_have_enough_found_to_reserve_the_proof_should_not_fail_but_raise_just_an_event(
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

mod should_clean_the_published_storage_on_initialize {
    use super::*;

    #[test]
    fn is_empty() {
        test().execute_with(|| {
            assert_eq!(Published::<Test>::get().is_empty(), true);
        })
    }

    #[test]
    fn should_be_emptied_on_initialize() {
        test().execute_with(|| {
            Published::<Test>::mutate(|published: &mut _| {
                published
                    .try_push(Aggregation::<Test>::create(12, 3))
                    .unwrap();
                published
                    .try_push(Aggregation::<Test>::create(13, 3))
                    .unwrap();
            });

            Aggregate::on_initialize(36);
            assert_eq!(Published::<Test>::get().is_empty(), true);
        })
    }

    #[test]
    fn and_return_the_write_db_weight() {
        test().execute_with(|| {
            Published::<Test>::mutate(|published: &mut _| {
                published
                    .try_push(Aggregation::<Test>::create(12, 3))
                    .unwrap();
                published
                    .try_push(Aggregation::<Test>::create(13, 3))
                    .unwrap();
            });

            let w = Aggregate::on_initialize(36);
            assert_eq!(w, db_weights().writes(1));
            // Sanity check: w is not void
            assert_ne!(w, 0.into());
        })
    }
}

mod should_aggregate {
    use frame_support::assert_err;

    use super::*;

    #[test]
    fn in_happy_path() {
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

    #[test]
    fn no_refound_publisher_on_error() {
        test().execute_with(|| {
            // Generate the aggregations. We use USER_1 for all operations till the last aggregate
            // call where we'll use USER_2: in this case if an error occurred we CANNOT refound
            // USER_2 because the submitters should not pay for a service that they not received.
            let max = <Test as crate::Config>::MaxPublishedPerBlock::get() as u32;
            let statements = (max + 1) * <Test as crate::Config>::AggregationSize::get();
            for _ in 0..statements {
                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, Default::default());
            }

            for id in 1..=max {
                Aggregate::aggregate(RawOrigin::Signed(USER_1).into(), DOMAIN_ID, id as u64)
                    .unwrap();
                assert_new_receipt(DOMAIN_ID, id as u64, None);
            }

            let user_balance = Balances::free_balance(USER_2);
            assert_err!(
                Aggregate::aggregate(
                    RawOrigin::Signed(USER_2).into(),
                    DOMAIN_ID,
                    (max + 1) as u64
                ),
                Error::<Test>::TooMuchAggregations
            );

            assert_eq!(user_balance, Balances::free_balance(USER_2));
        })
    }
}
