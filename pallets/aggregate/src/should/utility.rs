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

use crate::data::{AggregateSecurityRules, Delivery, Reserve};
use crate::mock::RuntimeEvent as TestEvent;
use crate::mock::{self, *};
use crate::*;
use data::{DomainState, StatementEntry};
use frame_support::weights::RuntimeDbWeight;
use frame_system::{EventRecord, Phase};
use sp_core::{Get, H256};

pub fn assert_evt(event: Event<Test>, context: &str) {
    assert_evt_gen(true, event, context);
}

pub fn assert_not_evt(event: Event<Test>, context: &str) {
    assert_evt_gen(false, event, context);
}

pub fn assert_proof_evt(domain_id: u32, id: u64, value: H256) {
    assert_proof_evt_gen(true, domain_id, id, value)
}

pub fn assert_not_proof_evt(domain_id: u32, id: u64, value: H256) {
    assert_proof_evt_gen(false, domain_id, id, value)
}

pub fn assert_complete_evt(domain_id: u32, id: u64) {
    assert_complete_evt_gen(true, domain_id, id);
}

pub fn assert_not_complete_evt(domain_id: u32, id: u64) {
    assert_complete_evt_gen(false, domain_id, id);
}

pub fn assert_cannot_aggregate_evt(statement: H256, cause: CannotAggregateCause) {
    assert_cannot_aggregate_evt_gen(true, statement, cause);
}

pub fn assert_no_cannot_aggregate_evt() {
    assert!(cannot_aggregate_events().is_empty());
}

pub fn assert_state_changed_evt(domain_id: u32, state: DomainState) {
    assert_state_event_evt_gen(true, domain_id, state);
}

pub fn assert_no_state_changed_evt() {
    assert!(
        state_events().is_empty(),
        "Should be empty: {:?}",
        state_events()
    );
}

pub fn assert_new_receipt(domain: u32, id: u64, expected_receipt: Option<H256>) {
    let matched = mock::System::events()
    .iter()
    .any(|record| {
        matches!(record.event, TestEvent::Aggregate(Event::<Test>::NewAggregationReceipt {
                domain_id,
                aggregation_id,
                receipt,
            }
        ) if domain_id == domain && aggregation_id == id && expected_receipt.map(|h| h == receipt).unwrap_or(true))
    });
    assert!(
        matched,
        "Cannot find aggregation receipt [{domain}-{id}]-{expected_receipt:?}"
    );
}

pub fn statement_entry(
    domain: Option<&Domain<Test>>,
    account: u64,
    statement: H256,
) -> StatementEntry<AccountId, Balance> {
    let size = |d: &Domain<Test>| d.max_aggregation_size as BalanceOf<Test>;
    let aggregation_size: BalanceOf<Test> = domain
        .map(size)
        .or_else(|| Domains::<Test>::get(DOMAIN_ID).map(|d| size(&d)))
        .unwrap();
    StatementEntry::new(
        account,
        Reserve::<Balance>::new(ESTIMATED_FEE_CORRECTED as u128 / aggregation_size, 0),
        statement,
    )
}

pub fn count_all_statements() -> usize {
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

impl Aggregation<Test> {
    pub(crate) fn add_statement(
        &mut self,
        account: AccountOf<Test>,
        reserve: Reserve<BalanceOf<Test>>,
        statement: H256,
    ) {
        self.statements
            .try_push(StatementEntry::new(account, reserve, statement))
            .unwrap();
    }
}

pub fn db_weights() -> RuntimeDbWeight {
    <<Test as frame_system::Config>::DbWeight as Get<RuntimeDbWeight>>::get()
}

pub fn registered_ids() -> Vec<u32> {
    mock::System::events()
        .iter()
        .filter_map(|record| match record.event {
            TestEvent::Aggregate(Event::<Test>::NewDomain { id, .. }) => Some(id),
            _ => None,
        })
        .collect()
}

pub fn cannot_aggregate_events() -> Vec<Event<Test>> {
    mock::System::events()
        .into_iter()
        .filter_map(|record| match record.event {
            TestEvent::Aggregate(ev @ Event::<Test>::CannotAggregate { .. }) => Some(ev),
            _ => None,
        })
        .collect()
}

pub fn state_events() -> Vec<Event<Test>> {
    mock::System::events()
        .into_iter()
        .filter_map(|record| match record.event {
            TestEvent::Aggregate(ev @ Event::<Test>::DomainStateChanged { .. }) => Some(ev),
            _ => None,
        })
        .collect()
}

pub fn register_domain(
    user: AccountId,
    size: AggregationSize,
    queue: Option<u32>,
    aggregate_rules: AggregateSecurityRules,
    delivery: Delivery<crate::mock::Balance>,
    delivery_owner: Option<crate::mock::AccountId>,
) -> u32 {
    frame_support::assert_ok!(Aggregate::register_domain(
        Origin::Signed(user).into(),
        size,
        queue,
        aggregate_rules,
        delivery,
        delivery_owner
    ));
    registered_ids()[0]
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
            event: TestEvent::Aggregate(event),
            topics: vec![],
        }),
        "{message}"
    )
}

fn assert_proof_evt_gen(contains: bool, domain_id: u32, id: u64, value: H256) {
    assert_evt_gen(
        contains,
        Event::NewProof {
            domain_id,
            aggregation_id: id,
            statement: value,
        },
        "Search new proof",
    );
}

fn assert_complete_evt_gen(contains: bool, domain_id: u32, id: u64) {
    assert_evt_gen(
        contains,
        Event::AggregationComplete {
            domain_id,
            aggregation_id: id,
        },
        "Completed aggregation",
    );
}

fn assert_cannot_aggregate_evt_gen(contains: bool, statement: H256, cause: CannotAggregateCause) {
    assert_evt_gen(
        contains,
        Event::CannotAggregate { statement, cause },
        "Cannot aggregate error",
    );
}

fn assert_state_event_evt_gen(contains: bool, domain_id: u32, state: DomainState) {
    assert_evt_gen(
        contains,
        Event::DomainStateChanged {
            id: domain_id,
            state,
        },
        "Domain state change",
    );
}
