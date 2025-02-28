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
use codec::Encode;
use frame_support::{
    assert_ok,
    traits::{schedule::DispatchTime, Currency, StorePreimage, VestingSchedule},
};
use hex_literal::hex;
use pallet_conviction_voting::{AccountVote, Vote};
use pallet_hyperbridge_aggregations::Params;
use pallet_verifiers::VkOrHash;
use sp_core::H256;
use sp_runtime::traits::Zero;
use sp_runtime::{traits::Hash, AccountId32, MultiAddress};

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
        let issuer: AccountId32 = testsfixtures::SAMPLE_USERS[0].raw_account.into();
        let account_ids: Vec<_> = testsfixtures::SAMPLE_USERS
            .iter()
            .skip(1)
            .map(|u| u.raw_account.into())
            .collect();
        let call = Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(issuer.clone()),
            value: 5000 * currency::ACME,
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
        let dest_1: AccountId32 = testsfixtures::SAMPLE_USERS[0].raw_account.into();
        let dest_2: AccountId32 = testsfixtures::SAMPLE_USERS[1].raw_account.into();

        let call_1 = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(dest_1.clone()),
            value: 5000 * currency::ACME,
        });
        let call_2 = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(dest_2.clone()),
            value: 5000 * currency::ACME,
        });
        assert_ok!(Utility::batch(RuntimeOrigin::root(), vec![call_1, call_2]));
    });
}

#[test]
fn pallet_vesting() {
    test().execute_with(|| {
        assert!(
            Vesting::vesting_balance(&testsfixtures::SAMPLE_USERS[0].raw_account.into()).is_none()
        );
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
            dest: MultiAddress::Id(testsfixtures::SAMPLE_USERS[2].raw_account.into()),
            value: 5000 * currency::ACME,
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
            dest: MultiAddress::Id(testsfixtures::SAMPLE_USERS[1].raw_account.into()),
            value: 5000 * currency::ACME,
        });
        let proposal = <Preimage as StorePreimage>::bound(call).unwrap();

        let origin = RuntimeOrigin::signed(testsfixtures::SAMPLE_USERS[1].raw_account.into());
        let proposal_origin = Box::new(frame_system::RawOrigin::Root.into());
        let enactment_moment = DispatchTime::At(10);

        assert_ok!(Referenda::submit(
            origin,
            proposal_origin,
            proposal,
            enactment_moment
        ));

        let origin = RuntimeOrigin::signed(testsfixtures::SAMPLE_USERS[1].raw_account.into());
        assert_ok!(ConvictionVoting::vote(origin, 0, aye(10_u128, 0)));
    });
}

#[test]
fn pallet_whitelist() {
    test().execute_with(|| {
        let origin = RuntimeOrigin::root();
        let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: MultiAddress::Id(testsfixtures::SAMPLE_USERS[1].raw_account.into()),
            value: 5000 * currency::ACME,
        });

        let encoded_call = call.encode();
        let call_hash = <Runtime as frame_system::Config>::Hashing::hash_of(&encoded_call);

        assert_ok!(Whitelist::whitelist_call(origin, call_hash));
    });
}

#[test]
fn pallet_treasury() {
    test().execute_with(|| {
        let asset_kind = Box::new(());
        let amount = 1000 * ACME;
        let beneficiary = Box::new(testsfixtures::SAMPLE_USERS[2].raw_account.into());
        let valid_from = None;

        let treasury_account = Treasury::account_id();
        let _ = Balances::make_free_balance_be(&treasury_account, 10000 * ACME);

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
        let sender = testsfixtures::SAMPLE_USERS[0].raw_account.into();
        let origin = RuntimeOrigin::signed(sender);
        let proxy_type = crate::proxy::ProxyType::Any;

        assert_ok!(Proxy::create_pure(origin, proxy_type, 0, 0));
    });
}

#[test]
fn pallet_bounties() {
    test().execute_with(|| {
        let proposer = testsfixtures::SAMPLE_USERS[2].raw_account.into();
        let origin = RuntimeOrigin::signed(proposer);

        let value = 1000 * ACME;
        let description = vec![0; 100];

        assert_ok!(Bounties::propose_bounty(origin, value, description.clone()));
    });
}

#[test]
fn pallet_zksync() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);
        let dummy_proof = [0; pallet_zksync_verifier::PROOF_SIZE];
        let dummy_pubs = [0; pallet_zksync_verifier::PUBS_SIZE];
        assert!(SettlementZksyncPallet::submit_proof(
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
        let dummy_proof = pallet_risc0_verifier::Proof::V1_1(vec![]);
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
fn pallet_proofofsql_availability() {
    test().execute_with(|| {
        let dummy_origin = AccountId32::new([0; 32]);

        let dummy_vk = Vec::new();
        let dummy_proof = Vec::new();
        let dummy_pubs = Vec::new();

        assert!(SettlementProofOfSqlPallet::submit_proof(
            RuntimeOrigin::signed(dummy_origin),
            VkOrHash::Vk(Box::new(dummy_vk.into())),
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

// Test definition and execution. Test body must be written in the execute_with closure.
#[test]
fn pallet_poe() {
    test().execute_with(|| {
        assert_ok!(Poe::publish_attestation(RuntimeOrigin::root()));
        // just checking code builds, hence the pallet is available to the runtime
    });
}

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
fn pallet_ismp() {
    test().execute_with(|| {
        assert!(Ismp::handle_unsigned(RuntimeOrigin::root(), Vec::new()).is_err());
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_ismp_grandpa() {
    test().execute_with(|| {
        assert_ok!(IsmpGrandpa::remove_state_machines(
            RuntimeOrigin::root(),
            Vec::new()
        ));
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_hyperbridge_aggregations() {
    test().execute_with(|| {
        let default_empty_aggr: [u8; 32] =
            hex!("290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563");
        let test_contract: [u8; 20] = hex!("d9145CCE52D386f254917e481eB44e9943F39138");
        let dummy_origin = AccountId32::new([0; 32]);

        let params = Params {
            aggregation_id: 1u64,
            aggregation: sp_core::H256(default_empty_aggr),
            module: sp_core::H160(test_contract),
            destination: StateMachine::Kusama(4009),
            timeout: 0,
            fee: Zero::zero(),
        };

        assert_ok!(HyperbridgeAggregations::dispatch_aggregation(
            RuntimeOrigin::signed(dummy_origin),
            params
        ));
        // just checking code builds, hence the pallet is available to the runtime
    });
}

#[test]
fn pallet_claim() {
    test().execute_with(|| {
        assert_ok!(Claim::end_airdrop(RuntimeOrigin::root()));
        // just checking code builds, hence the pallet is available to the runtime
    });
}
