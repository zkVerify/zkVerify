use crate::RuntimeCall;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::InstanceFilter;
use sp_runtime::RuntimeDebug;

/// The type used to represent the kinds of proxying allowed.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    scale_info::TypeInfo,
)]
pub enum ProxyType {
    Any = 0,
    // Don't add any new proxy types here. Anyway don't add a new type that isn't a
    // a `NonTransfer` subset without reconsider carefully the `is_superset()`
    // implementation
    NonTransfer = 1,
    Governance = 2,
    Staking = 3,
    CancelProxy = 4,
}

impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}

impl ProxyType {
    fn is_a_submit_proof_extrinsic(c: &RuntimeCall) -> bool {
        matches!(
            c,
            RuntimeCall::SettlementFFlonkPallet(pallet_verifiers::Call::submit_proof { .. })
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
                RuntimeCall::SettlementFFlonkPallet(..) |
                RuntimeCall::SettlementGroth16Pallet(..) |
                RuntimeCall::SettlementRisc0Pallet(..) |
                RuntimeCall::SettlementUltrahonkPallet(..) |
                RuntimeCall::SettlementUltraplonkPallet(..) |
                RuntimeCall::SettlementPlonky2Pallet(..)
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
