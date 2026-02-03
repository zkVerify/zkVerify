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
use core::fmt::Debug;
use frame_support::Blake2_128Concat;
use frame_support::{pallet_prelude::*, storage_alias};
use sp_core::{MaxEncodedLen, H160};

type AggregationSize = u32;

/// V3 type for [`crate::Domains`].
#[storage_alias]
pub type Domains<T: crate::Config> = StorageMap<crate::Pallet<T>, Blake2_128Concat, u32, Domain<T>>;

/// V3 type for [`crate::Domain`].
pub type Domain<T> = DomainEntry<
    crate::AccountOf<T>,
    crate::BalanceOf<T>,
    <T as crate::Config>::AggregationSize,
    <T as crate::Config>::MaxPendingPublishQueueSize,
    crate::TicketDomainOf<T>,
    crate::TicketAllowListOf<T>,
>;

use crate::data::CountableTicket;
pub use crate::data::{AggregateSecurityRules, AggregationEntry, DomainState, User};
use crate::ProofSecurityRules;
// Old v3 layout

/// Bounded version for State Machine
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum BoundedStateMachine {
    /// Evm state machines
    #[codec(index = 0)]
    Evm(u32),
    /// Polkadot parachains
    #[codec(index = 1)]
    Polkadot(u32),
    /// Kusama parachains
    #[codec(index = 2)]
    Kusama(u32),
    /// Substrate-based standalone chain
    #[codec(index = 3)]
    Substrate([u8; 4]),
    /// Tendermint chains
    #[codec(index = 4)]
    Tendermint([u8; 4]),
}

/// Configuration for Hyperbridge Dispatch params
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
pub struct HyperbridgeDispatchParameters {
    /// The destination state machine
    pub destination_chain: BoundedStateMachine,
    /// Module identifier of the receiving module
    pub destination_module: H160,
    /// Relative from the current timestamp at which this request expires in seconds.
    pub timeout: u64,
}

/// Configuration for Destination
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, Default)]
pub enum Destination {
    /// No Destination
    #[default]
    None,
    /// Hyperbridge Destination
    Hyperbridge(HyperbridgeDispatchParameters),
}

/// Delivering aggregations data
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, Default)]
pub struct Delivery<B: Debug + PartialEq> {
    /// Destination
    pub destination: Destination,
    /// fee
    pub fee: B,
    /// Tip for the delivery owner
    pub owner_tip: B,
}

/// Configuration for delivering aggregations
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
pub struct DeliveryParams<A, B: Debug + PartialEq> {
    /// The delivery channel owner
    pub owner: A,
    /// The delivery data
    pub data: Delivery<B>,
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(S, M))]
pub struct DomainEntry<
    A: alloc::fmt::Debug + core::cmp::PartialEq,
    B: alloc::fmt::Debug + core::cmp::PartialEq,
    S: Get<AggregationSize>,
    M: Get<u32>,
    T1: Encode + Decode + TypeInfo + MaxEncodedLen,
    T2: Encode + Decode + TypeInfo + MaxEncodedLen,
> {
    pub id: u32,
    pub owner: User<A>,
    pub state: DomainState,
    pub next: AggregationEntry<A, B, S>,
    pub max_aggregation_size: crate::AggregationSize,
    pub should_publish: BoundedBTreeMap<u64, AggregationEntry<A, B, S>, M>,
    pub publish_queue_size: u32,
    pub ticket_domain: Option<T1>,
    pub ticket_allowlist: Option<CountableTicket<T2>>,
    pub aggregate_rules: AggregateSecurityRules,
    pub proof_rules: ProofSecurityRules,
    pub delivery: DeliveryParams<A, B>,
}
