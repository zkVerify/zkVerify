// Copyright 2024, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Here we write the integration tests for proxy logic.

use frame_support::traits::InstanceFilter as _;
use rstest::rstest;

use crate::proxy::ProxyType;
use crate::RuntimeCall;

#[rstest]
#[case::ezkl_submit_proof(
    RuntimeCall::SettlementEzklPallet(pallet_verifiers::Call::submit_proof{
        vk_or_hash: Default::default(),
        proof: Default::default(),
        pubs: Default::default(),
        domain_id: None,
        })
)]
#[case::fflonk_submit_proof(
    RuntimeCall::SettlementFFlonkPallet(pallet_verifiers::Call::submit_proof{
        vk_or_hash: Default::default(),
        proof: [0u8; 768].into(),
        pubs: Default::default(),
        domain_id: None,
        })
)]
#[case::groth16_submit_proof(
    RuntimeCall::SettlementGroth16Pallet(pallet_verifiers::Call::submit_proof {
        vk_or_hash: Default::default(),
        proof: Default::default(),
        pubs: Default::default(),
        domain_id: None,
    })
)]
#[case::plonky2_submit_proof(
    RuntimeCall::SettlementPlonky2Pallet(pallet_verifiers::Call::submit_proof{
        vk_or_hash: Default::default(),
        proof: Default::default(),
        pubs: Default::default(),
        domain_id: None,
        })
)]
#[case::risc0_submit_proof(
    RuntimeCall::SettlementRisc0Pallet(pallet_verifiers::Call::submit_proof {
        vk_or_hash: Default::default(),
        proof: pallet_risc0_verifier::Proof::V2_1(Default::default()).into(),
        pubs: Default::default(),
        domain_id: None,
    })
)]
#[case::ultrahonk_submit_proof(
    RuntimeCall::SettlementUltrahonkPallet(pallet_verifiers::Call::submit_proof {
        vk_or_hash: Default::default(),
        proof: Default::default(),
        pubs: Default::default(),
        domain_id: None,
        })
)]
#[case::ultraplonk_submit_proof(
    RuntimeCall::SettlementUltraplonkPallet(pallet_verifiers::Call::submit_proof{
        vk_or_hash: Default::default(),
        proof: Default::default(),
        pubs: Default::default(),
        domain_id: None,
        })
)]
#[case::sp1(
    RuntimeCall::SettlementSp1Pallet(pallet_verifiers::Call::submit_proof{
        vk_or_hash: Default::default(),
        proof: Default::default(),
        pubs: Default::default(),
        domain_id: None,
        })
)]
fn nontransfer_deny_extrinsic(#[case] call: RuntimeCall) {
    let proxy = ProxyType::NonTransfer;

    assert!(!proxy.filter(&call))
}

#[rstest]
#[case::ezkl_unregister_vk(
    RuntimeCall::SettlementEzklPallet(pallet_verifiers::Call::unregister_vk{
        vk_hash: Default::default(),
        })
)]
#[case::fflonk_unregister_vk(
    RuntimeCall::SettlementFFlonkPallet(pallet_verifiers::Call::unregister_vk{
        vk_hash: Default::default(),
        })
)]
#[case::groth16_submit_proof(
    RuntimeCall::SettlementGroth16Pallet(pallet_verifiers::Call::unregister_vk{
        vk_hash: Default::default(),
        })
)]
#[case::plonky2_submit_proof(
    RuntimeCall::SettlementPlonky2Pallet(pallet_verifiers::Call::unregister_vk{
        vk_hash: Default::default(),
        })
)]
#[case::risc0_submit_proof(
    RuntimeCall::SettlementRisc0Pallet(pallet_verifiers::Call::unregister_vk{
        vk_hash: Default::default(),
        })
)]
#[case::ultrahonk_submit_proof(
    RuntimeCall::SettlementUltrahonkPallet(pallet_verifiers::Call::unregister_vk{
        vk_hash: Default::default(),
        })
)]
#[case::ultraplonk_submit_proof(
    RuntimeCall::SettlementUltraplonkPallet(pallet_verifiers::Call::unregister_vk{
        vk_hash: Default::default(),
        })
)]
#[case::sp1(
    RuntimeCall::SettlementSp1Pallet(pallet_verifiers::Call::unregister_vk{
        vk_hash: Default::default(),
        })
)]
fn nontransfer_verifier_accept_extrinsic(#[case] call: RuntimeCall) {
    let proxy = ProxyType::NonTransfer;

    assert!(proxy.filter(&call))
}
