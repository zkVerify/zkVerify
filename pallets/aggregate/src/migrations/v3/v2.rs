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

//! The old v1 layout: here we need to maintain the layout of the old storage
//! in order to be able to decode it.

use codec::{Decode, Encode};
use frame_support::Blake2_128Concat;
use frame_support::{pallet_prelude::*, storage_alias};
use sp_core::MaxEncodedLen;

type AggregationSize = u32;

/// V2 type for [`crate::Domains`].
#[storage_alias]
pub type Domains<T: crate::Config> = StorageMap<crate::Pallet<T>, Blake2_128Concat, u32, Domain<T>>;

/// V1 type for [`crate::Domain`].
pub type Domain<T> = DomainEntry<
    crate::AccountOf<T>,
    crate::BalanceOf<T>,
    <T as crate::Config>::AggregationSize,
    <T as crate::Config>::MaxPendingPublishQueueSize,
    crate::TicketDomainOf<T>,
>;

use crate::data::DeliveryParams;
pub use crate::data::{AggregateSecurityRules, AggregationEntry, DomainState, User};

// Old v2 layout
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(S, M))]
pub struct DomainEntry<
    A: alloc::fmt::Debug + core::cmp::PartialEq,
    B: alloc::fmt::Debug + core::cmp::PartialEq,
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
    pub aggregate_rules: AggregateSecurityRules,
    pub delivery: DeliveryParams<A, B>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Delivery;
    use crate::{data::Reserve, mock::Test};
    use hp_dispatch::Destination;
    use sp_core::{bytes::to_hex, H256};

    #[test]
    fn v1_domain_entry_encoding_should_never_change() {
        // If this test fails you should get the old layout and redefine it here.
        let v1_domain = Domain::<Test> {
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
            ticket: None,
            aggregate_rules: AggregateSecurityRules::Untrusted,
            delivery: DeliveryParams::new(
                123_u64,
                Delivery {
                    destination: Destination::None,
                    fee: 100,
                    owner_tip: 33,
                },
            ),
        };

        let encoded = to_hex(&v1_domain.encode(), false);

        let expected_encoded = to_hex(
            &hex_literal::hex!(
                "
                        17000000007b00000000000000012a000000000000001000000008c801000000000000e80300
                        00000000000000000000000000d0070000000000000000000000000000000000000000000000
                        0000000000000000000000000000000000000002b84a860c00000000000000d0070000000000
                        000000000000000000e803000000000000000000000000000000000000000000000000000000
                        00000000000000000000000000004acb117a0f0a000000000500000000007b00000000000000
                        006400000000000000000000000000000021000000000000000000000000000000
                        "
            ),
            false,
        );

        assert_eq!(expected_encoded, encoded, "Please check if some of the structs used in domain changed and report here the old version");
    }
}
