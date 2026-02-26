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

//! Here we write the integration tests that just check pallets are available to the runtime.

use super::*;
use frame_support::{
    assert_ok,
    traits::{schedule::DispatchTime, Currency, StorePreimage, VestingSchedule},
};
use pallet_conviction_voting::{AccountVote, Vote};
use pallet_verifiers::VkOrHash;
use sp_core::H256;
use sp_runtime::{AccountId32, MultiAddress};

#[test]
fn pallet_ezkl() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);

        let dummy_vk = pallet_ezkl_verifier::EzklVk::new(
            [0u8; pallet_ezkl_verifier::MAX_VK_LENGTH as usize].to_vec(),
        );
        let dummy_proof = Vec::new();
        let dummy_pubs = Vec::new();

        assert!(SettlementEzklPallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::Vk(dummy_vk.into()),
            dummy_proof.into(),
            dummy_pubs.into(),
            None,
        )
        .is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_fflonk() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);
        let dummy_proof: pallet_fflonk_verifier::Proof = [0; pallet_fflonk_verifier::PROOF_SIZE];
        let dummy_pubs: pallet_fflonk_verifier::Pubs = [0; pallet_fflonk_verifier::PUBS_SIZE];
        assert!(SettlementFFlonkPallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::from_hash(H256::zero()),
            dummy_proof.into(),
            dummy_pubs.into(),
            None,
        )
        .is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_multisig() {
    test().execute_with(|| {
        let issuer = sample_user_account(0);
        let account_ids: Vec<_> = testsfixtures::SAMPLE_USERS
            .iter()
            .skip(1)
            .map(|u| u.raw_account.into())
            .collect();
        let call = Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(issuer.clone()),
            value: 5000 * currency::VFY,
        }));
        assert_ok!(Multisig::as_multi(
            RuntimeOrigin::signed(issuer),
            2,
            account_ids,
            None,
            call,
            Weight::zero()
        ));
    })
}

#[test]
fn pallet_utility() {
    test().execute_with(|| {
        let dest_1 = sample_user_account(0);
        let dest_2 = sample_user_account(1);

        let call_1 = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(dest_1.clone()),
            value: 5000 * currency::VFY,
        });
        let call_2 = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(dest_2.clone()),
            value: 5000 * currency::VFY,
        });
        assert_ok!(Utility::batch(RuntimeOrigin::root(), vec![call_1, call_2]));
    });
}

#[test]
fn pallet_vesting() {
    test().execute_with(|| {
        assert!(Vesting::vesting_balance(&sample_user_account(0)).is_none());
    });
}

#[test]
fn pallet_preimage() {
    test().execute_with(|| {
        assert_ok!(Preimage::note_preimage(
            RuntimeOrigin::root(),
            vec![0xCA, 0xFE, 0xBA, 0xBE]
        ));
    });
}

#[test]
fn pallet_scheduler() {
    test().execute_with(|| {
        let call = Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(sample_user_account(1)),
            value: 5000 * currency::VFY,
        }));

        assert_ok!(Scheduler::schedule(
            RuntimeOrigin::root(),
            100,
            None,
            0,
            call
        ));
    });
}

fn aye(amount: Balance, conviction: u8) -> AccountVote<Balance> {
    let vote = Vote {
        aye: true,
        conviction: conviction.try_into().unwrap(),
    };
    AccountVote::Standard {
        vote,
        balance: amount,
    }
}
#[test]
fn pallet_referenda_and_conviction_voting() {
    test().execute_with(|| {
        let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(sample_user_account(1)),
            value: 5000 * currency::VFY,
        });
        let proposal = <Preimage as StorePreimage>::bound(call).unwrap();

        let origin = RuntimeOrigin::signed(sample_user_account(1));
        let proposal_origin = Box::new(frame_system::RawOrigin::Root.into());
        let enactment_moment = DispatchTime::At(10);

        assert_ok!(Referenda::submit(
            origin,
            proposal_origin,
            proposal,
            enactment_moment
        ));

        let origin = RuntimeOrigin::signed(sample_user_account(1));
        assert_ok!(ConvictionVoting::vote(origin, 0, aye(10_u128, 0)));
    });
}

#[test]
fn pallet_treasury() {
    test().execute_with(|| {
        let asset_kind = Box::new(());
        let amount = 1000 * VFY;
        let beneficiary = Box::new(sample_user_account(2));
        let valid_from = None;

        let treasury_account = Treasury::account_id();
        let _ = Balances::make_free_balance_be(&treasury_account, 10000 * VFY);

        assert_ok!(Treasury::spend(
            RuntimeOrigin::root(),
            asset_kind,
            amount,
            beneficiary,
            valid_from
        ));
    });
}

#[test]
fn pallet_proxy() {
    test().execute_with(|| {
        let sender = sample_user_account(0);
        let origin = RuntimeOrigin::signed(sender);
        let proxy_type = crate::proxy::ProxyType::Any;

        assert_ok!(Proxy::create_pure(origin, proxy_type, 0, 0));
    });
}

#[test]
fn pallet_bounties() {
    test().execute_with(|| {
        let proposer = sample_user_account(2);
        let origin = RuntimeOrigin::signed(proposer);

        let value = 1000 * VFY;
        let description = vec![0; 100];

        assert_ok!(Bounties::propose_bounty(origin, value, description.clone()));
    });
}

#[test]
fn pallet_groth16() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);
        assert!(SettlementGroth16Pallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::from_hash(H256::zero()),
            pallet_groth16_verifier::Proof::default().into(),
            Box::new(Vec::new()),
            None,
        )
        .is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_risc0() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);

        let dummy_vk = H256::default();
        let dummy_proof = pallet_risc0_verifier::Proof::V2_1(vec![]);
        let dummy_pubs = vec![];

        assert!(SettlementRisc0Pallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::Vk(dummy_vk.into()),
            dummy_proof.into(),
            dummy_pubs.into(),
            None,
        )
        .is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_ultrahonk() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);

        const VK_SIZE: usize = pallet_ultrahonk_verifier::VK_SIZE_V3_0;

        let dummy_vk = pallet_ultrahonk_verifier::VersionedVk::V3_0([0; VK_SIZE]);
        let dummy_proof = pallet_ultrahonk_verifier::VersionedProof::V3_0(
            pallet_ultrahonk_verifier::Proof::new(pallet_ultrahonk_verifier::ProofType::ZK, vec![]),
        );
        let dummy_pubs = Vec::new();

        assert!(SettlementUltrahonkPallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::Vk(dummy_vk.into()),
            Box::new(dummy_proof),
            dummy_pubs.into(),
            None,
        )
        .is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_ultraplonk() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);

        let dummy_vk = [0; pallet_ultraplonk_verifier::VK_SIZE];
        let dummy_proof = vec![0; pallet_ultraplonk_verifier::PROOF_SIZE];
        let dummy_pubs = Vec::new();

        assert!(SettlementUltraplonkPallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::Vk(dummy_vk.into()),
            dummy_proof.into(),
            dummy_pubs.into(),
            None,
        )
        .is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_plonky2_availability() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);

        let dummy_vk = pallet_plonky2_verifier::VkWithConfig::default();
        let dummy_proof = pallet_plonky2_verifier::Proof::default();
        let dummy_pubs = Vec::new();

        assert!(SettlementPlonky2Pallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::Vk(Box::new(dummy_vk)),
            dummy_proof.into(),
            dummy_pubs.into(),
            None,
        )
        .is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_sp1_availability() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);

        let dummy_vk = H256::default();
        let dummy_proof = Vec::new();
        let dummy_pubs = Vec::new();

        assert!(SettlementSp1Pallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::Vk(Box::new(dummy_vk)),
            dummy_proof.into(),
            dummy_pubs.into(),
            None,
        )
        .is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

// Test definition and execution. Test body must be written in the execute_with closure.
#[test]
fn pallet_bags_list() {
    test().execute_with(|| {
        assert!(VoterList::list_bags_get(12).is_none());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_aggregate() {
    test().execute_with(|| {
        assert!(Aggregate::aggregate(RuntimeOrigin::root(), 42, 24).is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_token_claim() {
    test().execute_with(|| {
        assert_ok!(TokenClaim::end_claim(RuntimeOrigin::root()));
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_identity() {
    test().execute_with(|| {
        let dummy_identity = pallet_identity::legacy::IdentityInfo::default();
        assert_ok!(Identity::set_identity(
            RuntimeOrigin::signed(sample_user_account(0)),
            Box::new(dummy_identity),
        ));
    });
}
