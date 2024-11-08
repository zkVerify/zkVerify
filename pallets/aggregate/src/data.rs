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
use scale_info::TypeInfo;
use sp_core::{Get, H256};
use sp_runtime::{traits::Keccak256, BoundedBTreeMap, BoundedVec};

/// Type used for the size of the aggregation.
pub type AggregationSize = u32;

#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// The statement data: all the data need to manage.
///
/// Types:
/// - `A`: The type of the account identifier.
/// - `B`: The type of the balance.
pub struct StatementEntry<A, B> {
    /// The statement owner: the who tha submit the proof and hold his found for publishing the aggregation.
    pub account: A,
    /// The amount of the reserve that the statement owner holds, it's the amount he will be used for the aggregation.
    pub reserve: B,
    /// The hash of the statement that will be used in the aggregation.
    pub statement: H256,
}

impl<A, B> StatementEntry<A, B> {
    pub fn new(account: A, reserve: B, statement: H256) -> Self {
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
        T::get() as u32
    }
}

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(S))]
/// The aggregation data. That is the entry where we put all the [`StatementEntry`]
/// that should be aggregated.
///
/// Types:
/// - `A`: The type of the account identifier.
/// - `B`: The type of the balance.
/// - `S`: The type of the maximum aggregation size.
pub struct AggregationEntry<A, B, S: Get<AggregationSize>> {
    /// The unique identifier (in the domain) of the aggregation.
    pub id: u64,
    /// The maximum number of statements that this aggregation can aggregate: should be less or equal
    /// to the configured maximum size (`S::get()``).
    pub size: AggregationSize,
    /// The statements that this aggregation will aggregate.
    pub statements: BoundedVec<StatementEntry<A, B>, VecSize<S>>,
}

impl<A: sp_std::fmt::Debug, B: sp_std::fmt::Debug, S: Get<AggregationSize>> sp_std::fmt::Debug
    for AggregationEntry<A, B, S>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AggregationEntry")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("statements", &self.statements)
            .finish()
    }
}

impl<A: PartialEq, B: PartialEq, S: Get<AggregationSize>> PartialEq for AggregationEntry<A, B, S> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.size == other.size && self.statements == other.statements
    }
}

impl<A, B, S: Get<AggregationSize>> AggregationEntry<A, B, S> {
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
    pub fn create_next(&self, size: AggregationSize) -> Self {
        Self::create(self.id + 1, size)
    }

    fn space_left(&self) -> usize {
        (self.size as usize).saturating_sub(self.statements.len())
    }

    pub fn compute(&self) -> H256 {
        binary_merkle_tree::merkle_root::<Keccak256, _>(
            self.statements.iter().map(|s| s.statement.as_ref()),
        )
    }

    fn encoded_size(size: AggregationSize) -> usize
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

impl<A, B, S: Get<AggregationSize>> Default for AggregationEntry<A, B, S> {
    fn default() -> Self {
        Self::create(1, S::get())
    }
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(S, M))]
/// The data stored fo a domain.
///
/// Types:
/// - `A`: The type of the account.
/// - `B`: The type of the balance.
/// - `S`: The type of the maximum aggregation size.
/// - `M`: The type of the maximum number of entries in the `should_publish` map.
/// - `T`: The type of consideration ticket used to hold the ballance for the space used
/// by domain storage.
pub struct DomainEntry<
    A,
    B,
    S: Get<AggregationSize>,
    M: Get<u32>,
    T: Encode + Decode + TypeInfo + MaxEncodedLen,
> {
    /// The unique identifier of the domain.
    pub id: u32,
    /// The account that owns this domain.
    pub owner: User<A>,
    /// The aggregation that is not already completed.
    pub next: AggregationEntry<A, B, S>,
    /// The maximum size of the aggregation for this domain.
    pub max_aggregation_size: AggregationSize,
    /// The attestations that are already completed but not published yet.
    pub should_publish: BoundedBTreeMap<u64, AggregationEntry<A, B, S>, M>,
    /// The maximum number of attestations that are waiting to be published: should be less equal to `M::get()`.
    pub publish_queue_size: u32,
    /// The consideration ticket used to hold the ballance for the space used by domain storage. The manager will
    /// not hold any balance.
    pub ticket: Option<T>,
}

impl<
        A,
        B,
        S: Get<AggregationSize>,
        M: Get<u32>,
        Ticket: Encode + Decode + TypeInfo + MaxEncodedLen,
    > DomainEntry<A, B, S, M, Ticket>
{
    /// Create a new domain.
    ///
    pub fn create(
        id: u32,
        owner: User<A>,
        next_attestation_id: u64,
        max_attestation_size: AggregationSize,
        publish_queue_size: u32,
        ticket: Option<Ticket>,
    ) -> Self {
        assert!(
            max_attestation_size <= S::get(),
            "Max aggregation size must be less or equal than Config::AggregationSize"
        );
        assert!(
            publish_queue_size <= M::get(),
            "Publish queue size must be less or equal than Config::MaxPendingPublishQueueSize"
        );
        Self {
            id,
            owner,
            next: AggregationEntry::create(next_attestation_id, max_attestation_size),
            max_aggregation_size: max_attestation_size,
            should_publish: Default::default(),
            publish_queue_size,
            ticket,
        }
    }

    /// Return true iff it's possible to add a new statement. In other words if there is some room in the
    /// should publish queue for new attestation or in the next aggregation there is space
    /// for more than one statement.
    pub fn can_add_statement(&self) -> bool {
        (self.publish_queue_size as usize).saturating_sub(self.should_publish.len()) > 0
            || self.next.space_left() > 1
    }

    /// Return the size in bytes for this domain that should be reserved in the storage.
    ///
    /// - `max_attestation_size`: The maximum size of the aggregations for this domain.
    /// - `publish_queue_size`: the publish queue size for this domain.
    pub fn encoded_size(max_attestation_size: AggregationSize, publish_queue_size: u32) -> usize
    where
        AggregationEntry<A, B, S>: MaxEncodedLen,
        Self: MaxEncodedLen,
        BoundedVec<StatementEntry<A, B>, VecSize<S>>: MaxEncodedLen,
        StatementEntry<A, B>: MaxEncodedLen,
    {
        let upper = Self::max_encoded_len();
        let aggregation_size = AggregationEntry::<A, B, S>::encoded_size(max_attestation_size);
        upper
            .saturating_sub(AggregationEntry::<A, B, S>::max_encoded_len())
            .saturating_sub(BoundedBTreeMap::<u64, AggregationEntry<A, B, S>, M>::max_encoded_len())
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
    Owner(A),
    /// The manager
    Manager,
}

impl<A> From<A> for User<A> {
    fn from(value: A) -> Self {
        User::Owner(value)
    }
}

impl<A> User<A> {
    /// return the owner account if any
    pub fn owner(&self) -> Option<&A> {
        match self {
            User::Owner(owner) => Some(owner),
            _ => None,
        }
    }
}
