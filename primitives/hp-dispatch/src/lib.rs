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

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! Traits for aggregation dispatch

use codec::{Decode, Encode, MaxEncodedLen};
use core::fmt::Debug;
use frame_support::{dispatch::DispatchResult, weights::Weight};
use scale_info::TypeInfo;
use sp_core::H256;

/// Trait to dispatch aggregations
pub trait DispatchAggregation<Balance, AccountId> {
    /// forward an aggregation to the destination
    fn dispatch_aggregation(
        domain_id: u32,
        aggregation_id: u64,
        aggregation: H256,
        destination: Destination,
        fee: Balance,
        delivery_owner: AccountId,
    ) -> DispatchResult;

    /// Maximum weight for this dispatch: should be the maximum weight for the dispatch
    /// all type of destination. *The implementation should be simple (ideally a constant)*
    fn max_weight() -> Weight;

    /// The weight for dispatch to a given destination. *Also in this case the implementation
    /// should be simple (ideally a constant)*
    fn dispatch_weight(destination: &Destination) -> Weight;
}

impl<Balance, AccountId> DispatchAggregation<Balance, AccountId> for () {
    fn dispatch_aggregation(
        _domain_id: u32,
        _aggregation_id: u64,
        _aggregation: H256,
        _destination: Destination,
        _fee: Balance,
        _delivery_owner: AccountId,
    ) -> DispatchResult {
        Ok(())
    }

    fn max_weight() -> Weight {
        Default::default()
    }

    fn dispatch_weight(_destination: &Destination) -> Weight {
        Default::default()
    }
}

/// Configuration for Destination
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, Default)]
pub enum Destination {
    /// No Destination
    #[default]
    None,
}
