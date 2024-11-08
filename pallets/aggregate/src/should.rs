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
use crate::mock::{self, *};
use frame_support::{
    assert_err, assert_ok,
    dispatch::{GetDispatchInfo, Pays},
    traits::Hooks,
};
use hp_on_proof_verified::OnProofVerified;
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

mod not_add_the_statement_to_any_domain_if {
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
    fn test() -> sp_io::TestExternalities {
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
            Aggregate::aggregate(Origin::Signed(33).into(), DOMAIN_ID, 1).unwrap();
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
            Aggregate::aggregate(Origin::Signed(33).into(), DOMAIN_ID, 1).unwrap();
            Aggregate::aggregate(Origin::Signed(33).into(), DOMAIN_ID, 3).unwrap();
            Aggregate::aggregate(Origin::Signed(33).into(), DOMAIN_ID, 5).unwrap();
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
        let elements = (0..DOMAIN_SIZE)
            .map(|i| statement_entry(None, USER_1, H256::from_low_u64_be(i.into())))
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

        assert_eq!(Balances::reserved_balance(account), DOMAIN_FEE);
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
                published.push((1, Aggregation::<Test>::create(12, 3)));
                published.push((2, Aggregation::<Test>::create(13, 3)));
            });

            Aggregate::on_initialize(36);
            assert_eq!(Published::<Test>::get().is_empty(), true);
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
    use frame_support::dispatch::DispatchInfo;

    use super::*;

    fn dispatch_info() -> DispatchInfo {
        Call::<Test>::aggregate {
            domain_id: 2,
            id: 42,
        }
        .get_dispatch_info()
    }

    #[test]
    fn emit_a_new_receipt() {
        test().execute_with(|| {
            for i in 0..<Test as crate::Config>::AggregationSize::get() {
                Aggregate::on_proof_verified(Some(USER_2), DOMAIN, H256::from_low_u64_be(i.into()));
            }

            assert_ok!(Aggregate::aggregate(
                Origin::Signed(USER_1).into(),
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

    #[test]
    fn use_correct_weight() {
        let info = dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(
            info.weight,
            MockWeightInfo::aggregate(mock::MaxAggregationSize::get() as u32)
        );
    }

    #[test]
    fn should_pay_just_for_the_real_used_weight() {
        test().execute_with(|| {
            for _ in 0..DOMAIN_SIZE {
                Aggregate::on_proof_verified(Some(USER_1), DOMAIN, Default::default());
            }

            let expected_weight = <Test as Config>::WeightInfo::aggregate(DOMAIN_SIZE as u32);

            assert_eq!(
                expected_weight,
                Aggregate::aggregate(Origin::Signed(PUBLISHER_USER).into(), DOMAIN_ID, 1)
                    .unwrap()
                    .calc_actual_weight(&dispatch_info())
            )
        })
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
                Some(8)
            ));
            let registered_id = registered_ids()[0];

            let domain = Domains::<Test>::get(registered_id).unwrap();

            assert_eq!(registered_id, domain.id);
            assert_eq!(16, domain.max_aggregation_size);
            assert_eq!(8, domain.publish_queue_size);
            assert_eq!(domain.next, Aggregation::<Test>::create(1, 16));
            assert!(domain.should_publish.is_empty());
        })
    }

    #[test]
    fn add_more_domains() {
        test().execute_with(|| {
            let values = [(8, Some(4)), (16, None), (32, Some(8))];
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                values[0].0,
                values[0].1
            ));
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                values[1].0,
                values[1].1
            ));
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                values[2].0,
                values[2].1
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
                    .unwrap_or_else(|| <Test as Config>::MaxPendingPublishQueueSize::get());
                assert_eq!(id, domain.id);
                assert_eq!(aggregation_size, domain.max_aggregation_size);
                assert_eq!(queue_size, domain.publish_queue_size);
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
                Some(MaxPendingPublishQueueSize::get())
            ));

            assert_err!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    0,
                    Some(MaxPendingPublishQueueSize::get())
                ),
                Error::<Test>::InvalidDomainParams
            );
            assert_err!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    MaxAggregationSize::get() + 1,
                    Some(MaxPendingPublishQueueSize::get())
                ),
                Error::<Test>::InvalidDomainParams
            );
            assert_err!(
                Aggregate::register_domain(
                    Origin::Signed(USER_DOMAIN_1).into(),
                    MaxAggregationSize::get(),
                    Some(MaxPendingPublishQueueSize::get() + 1)
                ),
                Error::<Test>::InvalidDomainParams
            );
        })
    }

    #[test]
    fn save_consideration_ticket_if_user_register_a_domain() {
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                16,
                None
            ));

            let domain = Domains::<Test>::get(registered_ids()[0]).unwrap();

            assert_eq!(
                Some(MockConsideration {
                    who: USER_DOMAIN_1,
                    count: 1,
                    size: Domain::<Test>::encoded_size(16, MaxPendingPublishQueueSize::get())
                        as u64,
                }),
                domain.ticket,
            );
        });
    }

    #[test]
    fn donst_store_consideration_ticket_if_manager_register_domain() {
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(
                Origin::Signed(ROOT_USER).into(),
                16,
                None
            ));

            let domain = Domains::<Test>::get(registered_ids()[0]).unwrap();

            assert_eq!(None, domain.ticket);
        });
    }

    #[test]
    fn not_change_domain_encoded_size() {
        // This test is here to check the you don't changed the domain struct without change `encoded_size`
        // accordantly
        use codec::MaxEncodedLen;
        // Check base: always TRUE
        assert_eq!(
            Domain::<Test>::max_encoded_len(),
            Domain::<Test>::encoded_size(
                MaxAggregationSize::get(),
                MaxPendingPublishQueueSize::get()
            )
        );

        // Fixture max
        assert_eq!(Domain::<Test>::max_encoded_len(), 61341);

        // Fixtures
        assert_eq!(
            1365,
            Domain::<Test>::encoded_size(1, MaxPendingPublishQueueSize::get())
        );
        assert_eq!(
            7251,
            Domain::<Test>::encoded_size(MaxAggregationSize::get(), 1)
        );
        assert_eq!(
            16365,
            Domain::<Test>::encoded_size(
                MaxAggregationSize::get() / 2,
                MaxPendingPublishQueueSize::get() / 2
            )
        );
    }

    #[test]
    fn rise_error_on_if_new_consideration_fails() {
        test().execute_with(|| {
            assert_err!(
                Aggregate::register_domain(Origin::Signed(USER_DOMAIN_ERROR_NEW).into(), 16, None),
                sp_runtime::DispatchError::from("User Domain Error New")
            );
        })
    }

    #[test]
    fn apply_fee() {
        test().execute_with(|| {
            assert_eq!(
                Aggregate::register_domain(Origin::Signed(USER_DOMAIN_1).into(), 16, None)
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
                Aggregate::register_domain(Origin::Signed(ROOT_USER).into(), 16, None)
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
        }
        .get_dispatch_info();

        assert_eq!(info.pays_fee, Pays::Yes);
        assert_eq!(info.weight, MockWeightInfo::register_domain());
    }
}

mod unregister_domain {
    use rstest::rstest;
    use sp_runtime::traits::BadOrigin;

    use super::*;

    const IDS: [u32; 4] = [123, 321, 412, 1000];

    // Register some Domains in the wild and set the last id
    fn test() -> sp_io::TestExternalities {
        let mut ext = super::test();
        ext.execute_with(|| {
            Domains::<Test>::insert(
                IDS[0],
                Domain::<Test>::create(IDS[0], USER_DOMAIN_1.into(), 1, 10, 3, None),
            );
            Domains::<Test>::insert(
                IDS[1],
                Domain::<Test>::create(IDS[1], USER_DOMAIN_1.into(), 1, 5, 4, None),
            );
            Domains::<Test>::insert(
                IDS[2],
                Domain::<Test>::create(IDS[2], USER_DOMAIN_2.into(), 1, 6, 5, None),
            );
            Domains::<Test>::insert(
                IDS[3],
                Domain::<Test>::create(IDS[3], USER_DOMAIN_2.into(), 1, 32, 6, None),
            );
            NextDomainId::<Test>::put(10_000);

            for id in IDS {
                assert!(Domains::<Test>::get(id).is_some());
            }

            // On IDS[0] we fill all possible aggregations:
            let len = 10 * 4 - 1;

            for (p, _) in (0..len).enumerate() {
                let u = USERS[p % USERS.len()].0;
                Aggregate::on_proof_verified(Some(u), Some(IDS[0]), Default::default());
            }
        });
        ext
    }

    #[test]
    fn remove_the_domain() {
        test().execute_with(|| {
            assert_ok!(Aggregate::unregister_domain(
                Origin::Signed(USER_DOMAIN_1).into(),
                IDS[1]
            ));

            assert!(Domains::<Test>::get(IDS[1]).is_none());
        })
    }

    #[test]
    fn raise_error_if_invalid_domain() {
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
    fn raise_error_if_the_issuer_is_not_the_owner() {
        test().execute_with(|| {
            assert_err!(
                Aggregate::unregister_domain(Origin::Signed(USER_DOMAIN_2).into(), IDS[1]),
                BadOrigin
            );
            assert_err!(
                Aggregate::unregister_domain(Origin::Signed(USER_DOMAIN_1).into(), IDS[2]),
                BadOrigin
            );
        })
    }

    #[test]
    fn manager_can_remove_domain() {
        test().execute_with(|| {
            for id in IDS {
                assert_ok!(Aggregate::unregister_domain(
                    Origin::Signed(ROOT_USER).into(),
                    id
                ));
                assert!(Domains::<Test>::get(id).is_none());
            }
        })
    }

    #[rstest]
    #[case::user(Origin::Signed(USER_DOMAIN_1))]
    #[case::manager(Origin::Signed(ROOT_USER))]
    fn unbound_all_founds_reserved_for_aggregation(#[case] origin: Origin) {
        test().execute_with(|| {
            let expected_1 = USERS[0].1;
            let expected_2 = USERS[1].1;
            //  Sanity check:
            assert!(expected_1 > Balances::free_balance(USERS[0].0));
            assert!(expected_2 > Balances::free_balance(USERS[1].0));

            assert_ok!(Aggregate::unregister_domain(origin.into(), IDS[0]));

            assert_eq!(expected_1, Balances::free_balance(USERS[0].0));
            assert_eq!(expected_2, Balances::free_balance(USERS[1].0));
        })
    }

    #[rstest]
    #[case::user(Origin::Signed(USER_DOMAIN_1))]
    #[case::manager(Origin::Signed(ROOT_USER))]
    fn emit_event_for_each_aggregation_that_announce_it_s_removed(#[case] origin: Origin) {
        test().execute_with(|| {
            assert_ok!(Aggregate::unregister_domain(origin.into(), IDS[0]));

            for id in 1..=4 {
                assert_evt(
                    Event::<Test>::AggregationRemoved {
                        domain_id: IDS[0],
                        aggregation_id: id,
                    },
                    "Removed aggregation",
                );
            }
        })
    }

    #[test]
    fn unregister_domain_drop_consideration_ticket() {
        let origin = Origin::Signed(USER_DOMAIN_1);
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(origin.clone().into(), 16, None));

            let id = registered_ids()[0];

            assert_ok!(Aggregate::unregister_domain(origin.into(), id));

            let (id, dropped_consideration) = MockConsideration::pop().unwrap();

            assert_eq!(USER_DOMAIN_1, id);
            assert_eq!(USER_DOMAIN_1, dropped_consideration.who);
        })
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic(expected = "Drop"))]
    fn ignore_error_on_drop_ticket_but_defensive_proof_on_test() {
        let origin = Origin::Signed(USER_DOMAIN_ERROR_DROP);
        test().execute_with(|| {
            assert_ok!(Aggregate::register_domain(origin.clone().into(), 16, None));

            let id = registered_ids()[0];

            Aggregate::unregister_domain(origin.into(), id).unwrap();
        })
    }

    #[test]
    fn apply_fee() {
        test().execute_with(|| {
            assert_eq!(
                Aggregate::unregister_domain(Origin::Signed(USER_DOMAIN_1).into(), IDS[0])
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
                Aggregate::unregister_domain(Origin::Signed(ROOT_USER).into(), IDS[0])
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
        assert_eq!(info.weight, MockWeightInfo::unregister_domain());
    }
}

mod get_statement_path {
    use super::*;

    use sp_runtime::traits::Keccak256;

    fn test() -> sp_io::TestExternalities {
        let mut ext = super::test();

        let mut a = Aggregation::<Test>::create(123, 16);
        (0..16_u64).for_each(|i| a.add_statement(USER_1, 0, H256::from_low_u64_be(i as u64)));

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
