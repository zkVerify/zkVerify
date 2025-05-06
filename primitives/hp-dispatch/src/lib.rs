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

//! Traits for hyperbridge

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{dispatch::DispatchResult, weights::Weight};
use ismp::host::StateMachine;
use scale_info::TypeInfo;
use sp_core::{H160, H256};
use sp_std::fmt::Debug;

/// Trait to dispatch aggregations
pub trait DispatchAggregation<B> {
    /// forward an aggregation to the destination
    fn dispatch_aggregation(
        domain_id: u32,
        aggregation_id: u64,
        aggregation: H256,
        destination: Destination,
        fee: B,
    ) -> DispatchResult;

    /// Maximum weight for this dispatch: should be the maximum weight for the dispatch
    /// all type of destination. *The implementation should be simple (ideally a constant)*
    fn max_weight() -> Weight;

    /// The weight for dispatch to a given destination. *Also in this case the implementation
    /// should be simple (ideally a constant)*
    fn dispatch_weight(destination: &Destination) -> Weight;
}

impl<B> DispatchAggregation<B> for () {
    fn dispatch_aggregation(
        _domain_id: u32,
        _aggregation_id: u64,
        _aggregation: H256,
        _destination: Destination,
        _fee: B,
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
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
pub enum Destination {
    /// No Destination
    None,
    /// Hyperbridge Destination
    Hyperbridge(HyperbridgeDispatchParameters),
}

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

impl From<BoundedStateMachine> for StateMachine {
    fn from(bsm: BoundedStateMachine) -> Self {
        match bsm {
            BoundedStateMachine::Evm(id) => StateMachine::Evm(id),
            BoundedStateMachine::Polkadot(id) => StateMachine::Polkadot(id),
            BoundedStateMachine::Kusama(id) => StateMachine::Kusama(id),
            BoundedStateMachine::Substrate(id) => StateMachine::Substrate(id),
            BoundedStateMachine::Tendermint(id) => StateMachine::Tendermint(id),
        }
    }
}
