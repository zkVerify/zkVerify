// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::RuntimeCall;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::traits::InstanceFilter;
use sp_runtime::RuntimeDebug;

/// The type used to represent the kinds of proxying allowed.
#[derive(
    Copy,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    DecodeWithMemTracking,
    RuntimeDebug,
    MaxEncodedLen,
    scale_info::TypeInfo,
)]
pub enum ProxyType {
    #[default]
    Any = 0,
    // Don't add any new proxy types here. Anyway don't add a new type that isn't a
    // a `NonTransfer` subset without reconsider carefully the `is_superset()`
    // implementation
    NonTransfer = 1,
    Governance = 2,
    Staking = 3,
    CancelProxy = 4,
}

impl ProxyType {
    fn is_a_submit_proof_extrinsic(c: &RuntimeCall) -> bool {
        matches!(
            c,
            RuntimeCall::SettlementEzklPallet(pallet_verifiers::Call::submit_proof { .. })
                | RuntimeCall::SettlementFFlonkPallet(pallet_verifiers::Call::submit_proof { .. })
                | RuntimeCall::SettlementGroth16Pallet(pallet_verifiers::Call::submit_proof { .. })
                | RuntimeCall::SettlementRisc0Pallet(pallet_verifiers::Call::submit_proof { .. })
                | RuntimeCall::SettlementUltrahonkPallet(
                    pallet_verifiers::Call::submit_proof { .. }
                )
                | RuntimeCall::SettlementUltraplonkPallet(
                    pallet_verifiers::Call::submit_proof { .. }
                )
                | RuntimeCall::SettlementPlonky2Pallet(pallet_verifiers::Call::submit_proof { .. })
                | RuntimeCall::SettlementSp1Pallet(pallet_verifiers::Call::submit_proof { .. })
        )
    }
}

impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::NonTransfer => {
                matches!(
                    c,
                    RuntimeCall::System(..) |
				RuntimeCall::Scheduler(..) |
				RuntimeCall::Babe(..) |
				RuntimeCall::Timestamp(..) |
				// Specifically omitting Indices `transfer`, `force_transfer`
				// Specifically omitting the entire Balances pallet
				RuntimeCall::Staking(..) |
				RuntimeCall::Session(..) |
				RuntimeCall::Grandpa(..) |
				RuntimeCall::Treasury(..) |
				RuntimeCall::Bounties(..) |
				RuntimeCall::ChildBounties(..) |
				RuntimeCall::ConvictionVoting(..) |
				RuntimeCall::Referenda(..) |
				RuntimeCall::Vesting(pallet_vesting::Call::vest{..}) |
				RuntimeCall::Vesting(pallet_vesting::Call::vest_other{..}) |
				// Specifically omitting Vesting `vested_transfer`, and `force_vested_transfer`
				RuntimeCall::Utility(..) |
				RuntimeCall::Proxy(..) |
				RuntimeCall::Multisig(..) |
				RuntimeCall::VoterList(..) |
                // zkVerify specifics
                RuntimeCall::Aggregate(..) |
                RuntimeCall::SettlementEzklPallet(..) |
                RuntimeCall::SettlementFFlonkPallet(..) |
                RuntimeCall::SettlementGroth16Pallet(..) |
                RuntimeCall::SettlementRisc0Pallet(..) |
                RuntimeCall::SettlementUltrahonkPallet(..) |
                RuntimeCall::SettlementUltraplonkPallet(..) |
                RuntimeCall::SettlementPlonky2Pallet(..) |
                RuntimeCall::SettlementSp1Pallet(..)
                ) && !Self::is_a_submit_proof_extrinsic(c)
            }
            ProxyType::Governance => matches!(
                c,
                RuntimeCall::Treasury(..)
                    | RuntimeCall::Bounties(..)
                    | RuntimeCall::Utility(..)
                    | RuntimeCall::ChildBounties(..)
                    | RuntimeCall::ConvictionVoting(..)
                    | RuntimeCall::Referenda(..)
            ),
            ProxyType::Staking => {
                matches!(
                    c,
                    RuntimeCall::Staking(..)
                        | RuntimeCall::Session(..)
                        | RuntimeCall::Utility(..)
                        | RuntimeCall::VoterList(..)
                )
            }
            ProxyType::CancelProxy => {
                matches!(
                    c,
                    RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
                )
            }
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => true,
            _ => false,
        }
    }
}
