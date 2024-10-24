// Copyright 2024, Horizen Labs, Inc.
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
use frame_system::{EventRecord, Phase, RawOrigin};
use hp_poe::OnProofVerified;
use sp_core::H256;
use sp_runtime::SaturatedConversion;

fn assert_element_evt(domain_id: u32, id: u64, value: H256) {
    assert!(mock::System::events().contains(&EventRecord {
        phase: Phase::Initialization,
        event: TestEvent::Aggregate(crate::Event::NewElement {
            domain_id,
            attestation_id: id,
            value,
        }),
        topics: vec![],
    }))
}

fn assert_available_evt(domain_id: u32, id: u64) {
    assert!(mock::System::events().contains(&EventRecord {
        phase: Phase::Initialization,
        event: TestEvent::Aggregate(crate::Event::AvailableAttestation { domain_id, id }),
        topics: vec![],
    }))
}

fn assert_cannot_aggregate_statement_evt(statement: H256, cause: CannotAggregateCause) {
    assert!(mock::System::events().contains(&EventRecord {
        phase: Phase::Initialization,
        event: TestEvent::Aggregate(crate::Event::CannotAggregate { statement, cause }),
        topics: vec![],
    }))
}

fn statement_entry(account: u64, statement: H256) -> StatementEntry<AccountId, Balance> {
    StatementEntry::new(account, FEE_PER_STATEMENT_CORRECTED as u128, statement)
}

fn count_all_statements() -> usize {
    let all_attestations: usize = Domains::<Test>::iter_values()
        .map(|d| d.next_attestation.statements.iter().count())
        .sum();
    all_attestations
}

#[test]
fn should_add_a_proof_only() {
    new_test_ext().execute_with(|| {
        let statement = H256::from_low_u64_be(123);
        let account = 42;

        Aggregate::on_proof_verified(Some(account), DOMAIN, statement);

        assert_element_evt(DOMAIN_ID, 1, statement);
        let att = &Domains::<Test>::get(DOMAIN_ID).unwrap().next_attestation;
        assert_eq!(1, att.id);
        assert_eq!(vec![statement_entry(account, statement)], *att.statements);
    })
}

mod should_not_add_the_proof_to_any_domain_if {
    use super::*;

    #[test]
    fn no_domain_provided() {
        new_test_ext().execute_with(|| {
            let statement = H256::from_low_u64_be(123);
            let account = 42;

            Aggregate::on_proof_verified(Some(account), None, statement);

            assert_cannot_aggregate_statement_evt(statement, CannotAggregateCause::NoDomain);

            assert_eq!(0, count_all_statements());
        })
    }

    #[test]
    fn is_not_registered() {
        new_test_ext().execute_with(|| {
            let statement = H256::from_low_u64_be(123);
            let account = 42;

            Aggregate::on_proof_verified(Some(account), NOT_REGISTERED_DOMAIN, statement);

            assert_cannot_aggregate_statement_evt(
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
        new_test_ext().execute_with(|| {
            let statement = H256::from_low_u64_be(123);

            Aggregate::on_proof_verified(None, DOMAIN, statement);

            assert_cannot_aggregate_statement_evt(statement, CannotAggregateCause::NoAccount);

            assert_eq!(0, count_all_statements());
        })
    }
}

#[test]
fn should_not_add_the_proof_to_any_domain_is_not_registered() {
    new_test_ext().execute_with(|| {
        let statement = H256::from_low_u64_be(123);
        let account = 42;

        Aggregate::on_proof_verified(Some(account), NOT_REGISTERED_DOMAIN, statement);

        assert_cannot_aggregate_statement_evt(
            statement,
            CannotAggregateCause::DomainNotRegistered {
                domain_id: NOT_REGISTERED_DOMAIN_ID,
            },
        );

        assert_eq!(0, count_all_statements());
    })
}

#[test]
fn should_not_add_the_proof_to_any_domain_is_no_account() {
    new_test_ext().execute_with(|| {
        let statement = H256::from_low_u64_be(123);
        Aggregate::on_proof_verified(None, DOMAIN, statement);

        assert_cannot_aggregate_statement_evt(statement, CannotAggregateCause::NoAccount);

        assert_eq!(0, count_all_statements());
    })
}

#[test]
fn should_queue_a_new_attestation_when_is_complete() {
    new_test_ext().execute_with(|| {
        let account = 42;
        let elements = (0..<Test as crate::Config>::AttestationSize::get())
            .map(|i| statement_entry(account, H256::from_low_u64_be(i.into())))
            .collect::<Vec<_>>();
        for s in elements.clone().into_iter() {
            Aggregate::on_proof_verified(Some(s.account.clone()), DOMAIN, s.statement);
        }
        assert_available_evt(DOMAIN_ID, 1);
        let att = Domains::<Test>::take(DOMAIN_ID)
            .and_then(|mut d| d.should_publish.remove(&1))
            .unwrap();
        assert_eq!(1, att.id);
        assert_eq!(elements, *att.statements);
    })
}

#[test]
fn should_publish_attestation() {
    new_test_ext().execute_with(|| {
        for i in 0..<Test as crate::Config>::AttestationSize::get() {
            Aggregate::on_proof_verified(Some(24), DOMAIN, H256::from_low_u64_be(i.into()));
        }
        assert_ok!(Aggregate::publish_attestation(
            RawOrigin::Signed(42).into(),
            DOMAIN_ID,
            1
        ));
    })
}

#[test]
fn add_a_proof_should_reserve_at_least_the_publish_proof_price_fraction() {
    new_test_ext().execute_with(|| {
        let statement = H256::from_low_u64_be(123);
        let account = 42;

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
    new_test_ext().execute_with(|| {
        let statement = H256::from_low_u64_be(123);
        let account = NO_FOUND_USER;

        Aggregate::on_proof_verified(Some(account), DOMAIN, statement);
        assert_eq!(
            Balances::reserved_balance(account),
            0,
            "Should not reserve any balance"
        );
        assert_cannot_aggregate_statement_evt(statement, CannotAggregateCause::InsufficientFound);
        assert_eq!(1, mock::System::events().len())
    })
}

#[test]
fn the_publisher_should_receive_the_bounded_founds() {
    new_test_ext().execute_with(|| {
        let accounts = [USERS[0].0, USERS[1].0];
        let elements = (0..(<Test as crate::Config>::AttestationSize::get() as u64))
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
        assert_ok!(Aggregate::publish_attestation(
            RawOrigin::Signed(PUBLISHER_USER).into(),
            DOMAIN_ID,
            1
        ));

        assert_eq!(Balances::free_balance(PUBLISHER_USER), expected_balance);
    })
}
