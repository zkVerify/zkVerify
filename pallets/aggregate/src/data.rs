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

use core::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use educe::Educe;
use frame_support::{PartialEqNoBound, RuntimeDebugNoBound};
use hp_dispatch::Destination;
use scale_info::TypeInfo;
use sp_core::{Get, H256};
use sp_runtime::{traits::Keccak256, BoundedBTreeMap, BoundedVec};

/// Type used for the size of the aggregation.
pub type AggregationSize = u32;

#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// Reserved currency.
///
/// Types:
/// - `A`: The type of the account identifier.
/// - `B`: The type of the balance.
pub struct Reserve<B> {
    /// Balance reserved for aggregation
    pub aggregate: B,
    /// Balance reserved for delivery
    pub delivery: B,
}

impl<B> Reserve<B> {
    pub fn new(aggregate: B, delivery: B) -> Self {
        Self {
            aggregate,
            delivery,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// The statement data.
///
/// Types:
/// - `A`: The type of the account identifier.
/// - `B`: The type of the balance.
pub struct StatementEntry<A, B> {
    /// The statement owner: the one who submits the proof and holds his funds for publishing the aggregation.
    pub account: A,
    /// The amount of the reserve that the statement owner holds, it's the amount he will be used for the aggregation and delivering.
    pub reserve: Reserve<B>,
    /// The hash of the statement that will be used in the aggregation.
    pub statement: H256, // IMPORTANT NOTE: Must NOT be 64 bytes in length in order to avoid risks of proof forgery and leaf-branch ambiguity.
}

impl<A, B> StatementEntry<A, B> {
    pub fn new(account: A, reserve: Reserve<B>, statement: H256) -> Self {
        Self {
            account,
            reserve,
            statement,
        }
    }
}

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VecSize<T>(PhantomData<T>);
impl<T: Get<AggregationSize>> Get<u32> for VecSize<T> {
    fn get() -> u32 {
        T::get()
    }
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebugNoBound, PartialEqNoBound)]
#[educe(Clone)]
#[scale_info(skip_type_params(S))]
/// The aggregation data. That is the entry where we put all the [`StatementEntry`]
/// that should be aggregated.
///
/// Types:
/// - `A`: The type of the account identifier.
/// - `B`: The type of the balance.
/// - `S`: The type of the maximum aggregation size.
pub struct AggregationEntry<
    A: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
    B: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
    S: Get<AggregationSize>,
> {
    /// The unique identifier (in the domain) of the aggregation.
    pub id: u64,
    /// The maximum number of statements that this aggregation can aggregate: should be less or equal
    /// to the configured maximum size (`S::get()``).
    pub size: AggregationSize,
    /// The statements that this aggregation will aggregate.
    pub statements: BoundedVec<StatementEntry<A, B>, VecSize<S>>,
}

impl<
        A: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
        B: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
        S: Get<AggregationSize>,
    > AggregationEntry<A, B, S>
{
    fn new(
        id: u64,
        size: AggregationSize,
        statements: BoundedVec<StatementEntry<A, B>, VecSize<S>>,
    ) -> Self {
        assert!(size <= S::get(), "Aggregation size is out of bound");
        Self {
            id,
            size,
            statements,
        }
    }

    /// Create a new aggregation entry with the given ID and size.
    pub fn create(id: u64, size: AggregationSize) -> Self {
        Self::new(id, size, BoundedVec::with_bounded_capacity(size as usize))
    }

    /// Create a new aggregation entry with the given size. Just increment
    /// the id.
    pub fn create_next(&self, size: AggregationSize) -> Option<Self> {
        self.id
            .checked_add(1)
            .map(|next_id| Self::create(next_id, size))
    }

    fn space_left(&self) -> usize {
        (self.size as usize).saturating_sub(self.statements.len())
    }

    pub fn completed(&self) -> bool {
        self.space_left() == 0
    }

    pub fn compute_receipt(&self) -> H256 {
        binary_merkle_tree::merkle_root::<Keccak256, _>(
            self.statements.iter().map(|s| s.statement.as_ref()),
        )
    }

    pub(crate) fn compute_encoded_size(size: AggregationSize) -> usize
    where
        Self: MaxEncodedLen,
        BoundedVec<StatementEntry<A, B>, VecSize<S>>: MaxEncodedLen,
        StatementEntry<A, B>: MaxEncodedLen,
    {
        let dyn_size = codec::Compact(S::get()).encoded_size().saturating_add(
            (size as usize).saturating_mul(StatementEntry::<A, B>::max_encoded_len()),
        );

        Self::max_encoded_len()
            .saturating_sub(BoundedVec::<StatementEntry<A, B>, VecSize<S>>::max_encoded_len())
            .saturating_add(dyn_size)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// The state of a domain.
pub enum DomainState {
    /// Active and can receive new statements.
    Ready,
    /// Cannot receive new statements. Can just publish the aggregation that are
    /// already to be published queue.
    Hold,
    /// This Hold domain can be removed. There are no statements in this domain
    /// and it can be removed.
    Removable,
    /// This domain is removed.
    Removed,
}

impl<
        A: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
        B: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
        S: Get<AggregationSize>,
    > Default for AggregationEntry<A, B, S>
{
    fn default() -> Self {
        Self::create(1, S::get())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// The rules that describe the when accept or reject the aggregate extrinsic call.
pub enum AggregateSecurityRules {
    /// Accept any aggregate extrinsic call from any user.
    Untrusted,
    /// Only owner and manager can call aggregate on this domain.
    OnlyOwner,
    /// Only owner and manager can call aggregate on this domain for uncompleted aggregations.
    OnlyOwnerUncompleted,
}

/// Delivering aggregations data
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
pub struct Delivery<B: sp_std::fmt::Debug + sp_std::cmp::PartialEq> {
    /// Destination
    pub destination: Destination,
    /// Price
    pub price: B,
}

impl<B: sp_std::fmt::Debug + sp_std::cmp::PartialEq> Delivery<B> {
    pub fn new(destination: Destination, price: B) -> Self {
        Self { destination, price }
    }
}

/// Configuration for delivering aggregations
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
pub struct DeliveryParams<A, B: sp_std::fmt::Debug + sp_std::cmp::PartialEq> {
    /// The delivery channel owner
    pub owner: A,
    /// The delivery data
    data: Delivery<B>,
}

impl<A, B: sp_std::fmt::Debug + sp_std::cmp::PartialEq> DeliveryParams<A, B> {
    pub fn new(owner: A, data: Delivery<B>) -> Self {
        Self { owner, data }
    }

    /// The delivery aggregation price
    pub fn price(&self) -> &B {
        &self.data.price
    }

    /// Set the delivery aggregation price
    pub fn set_price(&mut self, price: B) {
        self.data.price = price
    }

    /// The delivery destination
    pub fn destination(&self) -> &Destination {
        &self.data.destination
    }
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(S, M))]
/// The data stored for a domain.
///
/// Types:
/// - `A`: The type of the account.
/// - `B`: The type of the balance.
/// - `S`: The type of the maximum aggregation size.
/// - `M`: The type of the maximum number of entries in the `should_publish` map.
/// - `T`: The type of consideration ticket used to hold the balance for the space used
///   by domain storage.
pub struct DomainEntry<
    A: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
    B: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
    S: Get<AggregationSize>,
    M: Get<u32>,
    T: Encode + Decode + TypeInfo + MaxEncodedLen,
> {
    /// The unique identifier of the domain.
    pub id: u32,
    /// The account that owns this domain.
    pub owner: User<A>,
    /// The state of the domain.
    pub state: DomainState,
    /// The aggregation that is not yet completed.
    pub next: AggregationEntry<A, B, S>,
    /// The maximum size of the aggregation for this domain.
    pub max_aggregation_size: AggregationSize,
    /// The aggregations that are already completed but not published yet.
    pub should_publish: BoundedBTreeMap<u64, AggregationEntry<A, B, S>, M>,
    /// The maximum number of aggregations that are waiting to be published: should be less equal to `M::get()`.
    pub publish_queue_size: u32,
    /// The consideration ticket used to hold the balance for the space used by domain storage. The manager will
    /// not hold any balance.
    pub ticket: Option<T>,
    /// Configure the rules that describe the when accept or reject the aggregate extrinsic call.
    pub aggregate_rules: AggregateSecurityRules,
    /// Configuration params for destination chain to delivery aggregations
    pub delivery: DeliveryParams<A, B>,
}

impl<
        A: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
        B: sp_std::fmt::Debug + sp_std::cmp::PartialEq,
        S: Get<AggregationSize>,
        M: Get<u32>,
        Ticket: Encode + Decode + TypeInfo + MaxEncodedLen,
    > DomainEntry<A, B, S, M, Ticket>
{
    /// Create a new domain.
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: u32,
        owner: User<A>,
        next_aggregation_id: u64,
        max_aggregation_size: AggregationSize,
        publish_queue_size: u32,
        aggregate_rules: AggregateSecurityRules,
        ticket: Option<Ticket>,
        delivery: DeliveryParams<A, B>,
    ) -> Self {
        assert!(
            max_aggregation_size <= S::get(),
            "Max aggregation size must be less or equal than Config::AggregationSize"
        );
        assert!(
            publish_queue_size <= M::get(),
            "Publish queue size must be less or equal than Config::MaxPendingPublishQueueSize"
        );
        Self {
            id,
            owner,
            state: DomainState::Ready,
            next: AggregationEntry::create(next_aggregation_id, max_aggregation_size),
            max_aggregation_size,
            should_publish: Default::default(),
            publish_queue_size,
            aggregate_rules,
            ticket,
            delivery,
        }
    }

    /// Return true iff it's possible to add a new statement. In other words if there is some room in the
    /// should publish queue for new aggregation or in the next aggregation there is space
    /// for more than one statement.
    pub fn can_add_statement(&self) -> bool {
        (self.publish_queue_size as usize).saturating_sub(self.should_publish.len()) > 0
            || self.next.space_left() > 1
    }

    /// Update the hold state according to the domain state.
    pub fn update_hold_state(&mut self) {
        self.state = if self.should_publish.is_empty() && self.next.statements.is_empty() {
            DomainState::Removable
        } else {
            DomainState::Hold
        };
    }

    /// Return the size in bytes for this domain that should be reserved in the storage.
    ///
    /// - `max_aggregation_size`: The maximum size of the aggregations for this domain.
    /// - `publish_queue_size`: The publish queue size for this domain.
    /// - `destination`: The destination chain to delivery aggregations.
    pub fn compute_encoded_size(
        max_aggregation_size: AggregationSize,
        publish_queue_size: u32,
        destination: &Destination,
    ) -> usize
    where
        AggregationEntry<A, B, S>: MaxEncodedLen,
        Self: MaxEncodedLen,
        BoundedVec<StatementEntry<A, B>, VecSize<S>>: MaxEncodedLen,
        StatementEntry<A, B>: MaxEncodedLen,
        Destination: MaxEncodedLen,
    {
        let upper = Self::max_encoded_len();
        let aggregation_size =
            AggregationEntry::<A, B, S>::compute_encoded_size(max_aggregation_size);
        upper
            .saturating_sub(AggregationEntry::<A, B, S>::max_encoded_len())
            .saturating_sub(BoundedBTreeMap::<u64, AggregationEntry<A, B, S>, M>::max_encoded_len())
            .saturating_sub(Destination::max_encoded_len())
            .saturating_add(destination.encoded_size())
            .saturating_add(aggregation_size)
            .saturating_add(
                (publish_queue_size as usize)
                    .saturating_mul(u64::max_encoded_len().saturating_add(aggregation_size))
                    .saturating_add(codec::Compact(M::get()).encoded_size()),
            )
    }
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Debug, PartialEq, Clone)]
/// User wrapper
///
/// `A` is the account type
pub enum User<A> {
    /// A account owner
    Account(A),
    /// The manager
    Manager,
}

impl<A> From<A> for User<A> {
    fn from(value: A) -> Self {
        User::Account(value)
    }
}

impl<A> User<A> {
    /// return the owner account if any
    pub fn account(&self) -> Option<&A> {
        match self {
            User::Account(account) => Some(account),
            _ => None,
        }
    }
}
