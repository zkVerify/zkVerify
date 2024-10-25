use frame_support::weights::RuntimeDbWeight;
use frame_system::{EventRecord, Phase};
use sp_core::{Get, H256};

use crate::mock::RuntimeEvent as TestEvent;
use crate::mock::{self, *};
use crate::*;

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

pub fn assert_ready_evt(domain_id: u32, id: u64) {
    assert_ready_evt_gen(true, domain_id, id);
}

pub fn assert_not_ready_evt(domain_id: u32, id: u64) {
    assert_ready_evt_gen(false, domain_id, id);
}

pub fn assert_cannot_aggregate_evt(statement: H256, cause: CannotAggregateCause) {
    assert_cannot_aggregate_evt_gen(true, statement, cause);
}

pub fn assert_new_receipt(domain: u32, id: u64, expected_receipt: Option<H256>) {
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

pub fn statement_entry(account: u64, statement: H256) -> StatementEntry<AccountId, Balance> {
    StatementEntry::new(account, FEE_PER_STATEMENT_CORRECTED as u128, statement)
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
        reserve: BalanceOf<Test>,
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

fn assert_evt_gen(contains: bool, event: Event<Test>, context: &str) {
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

fn assert_proof_evt_gen(contains: bool, domain_id: u32, id: u64, value: H256) {
    assert_evt_gen(
        contains,
        Event::ProofVerified {
            domain_id,
            aggregation_id: id,
            statement: value,
        },
        "Search new proof",
    );
}

fn assert_ready_evt_gen(contains: bool, domain_id: u32, id: u64) {
    assert_evt_gen(
        contains,
        Event::ReadyToAggregate {
            domain_id,
            aggregation_id: id,
        },
        "Available aggregation",
    );
}

fn assert_cannot_aggregate_evt_gen(contains: bool, statement: H256, cause: CannotAggregateCause) {
    assert_evt_gen(
        contains,
        Event::CannotAggregate { statement, cause },
        "Cannot aggregate error",
    );
}
