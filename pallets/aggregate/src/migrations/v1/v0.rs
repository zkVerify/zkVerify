//! The old v0 layout: here we need to maintain the layout of the old storage
//! in order to be able to decode it.

use codec::{Decode, Encode};
use frame_support::Blake2_128Concat;
use frame_support::{pallet_prelude::*, storage_alias};
use sp_core::MaxEncodedLen;

/// V0 type for [`crate::Domains`].
#[storage_alias]
pub type Domains<T: crate::Config> = StorageMap<crate::Pallet<T>, Blake2_128Concat, u32, Domain<T>>;

/// V0 type for [`crate::Domain`].
// I don't need to wrap it in a new type because encoding should not change.
pub type Domain<T> = DomainEntry<
    crate::AccountOf<T>,
    crate::BalanceOf<T>,
    <T as crate::Config>::AggregationSize,
    <T as crate::Config>::MaxPendingPublishQueueSize,
    crate::TicketOf<T>,
>;
pub use crate::data::{AggregationEntry, DomainState, User};

type AggregationSize = u32;

// Old layout
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(S, M))]
pub struct DomainEntry<
    A: core::fmt::Debug + core::cmp::PartialEq,
    B: core::fmt::Debug + core::cmp::PartialEq,
    S: Get<AggregationSize>,
    M: Get<u32>,
    T: Encode + Decode + TypeInfo + MaxEncodedLen,
> {
    pub id: u32,
    pub owner: User<A>,
    pub state: DomainState,
    pub next: AggregationEntry<A, B, S>,
    pub max_aggregation_size: AggregationSize,
    pub should_publish: BoundedBTreeMap<u64, AggregationEntry<A, B, S>, M>,
    pub publish_queue_size: u32,
    pub ticket: Option<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::Reserve,
        mock::{MockConsideration, Test},
    };
    use sp_core::{bytes::to_hex, H256};

    #[test]
    fn v0_domain_entry_encoding_should_never_change() {
        // If this test fails you should get the old layout and redefine it here.
        let v0_domain = Domain::<Test> {
            id: 23,
            owner: User::from(123_u64),
            state: DomainState::Hold,
            next: AggregationEntry {
                id: 42,
                size: 16,
                statements: BoundedVec::try_from(vec![
                    crate::data::StatementEntry::new(
                        456_u64,
                        Reserve::new(1000, 2000),
                        H256::from_low_u64_be(45632134),
                    ),
                    crate::data::StatementEntry::new(
                        12_u64,
                        Reserve::new(2000, 1000),
                        H256::from_low_u64_be(321234500111),
                    ),
                ])
                .unwrap(),
            },
            max_aggregation_size: 10,
            should_publish: BoundedBTreeMap::new(),
            publish_queue_size: 5,
            ticket: Some(MockConsideration {
                who: 321,
                count: 10,
                size: 1000,
            }),
        };

        let encoded = to_hex(&v0_domain.encode(), false);

        let expected_encoded = to_hex(&hex_literal::hex!("
                        17000000007b00000000000000012a000000000000001000000008c801000000000000e8030000
                        000000000000000000000000d00700000000000000000000000000000000000000000000000000
                        000000000000000000000000000000000002b84a860c00000000000000d0070000000000000000
                        000000000000e80300000000000000000000000000000000000000000000000000000000000000
                        000000000000000000004acb117a0f0a00000000050000000141010000000000000a0000000000
                        0000e803000000000000
                        "
        ), false);

        assert_eq!(expected_encoded, encoded, "Please check if some of the structs used in domain changed and report here the old version");
    }
}
