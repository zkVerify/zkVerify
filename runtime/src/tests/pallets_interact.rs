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

//! Here we write tests that verify the correct pallets interaction (if they are correctly configured).

use codec::Encode;
use frame_support::{
    assert_ok,
    dispatch::GetDispatchInfo,
    traits::{EstimateNextNewSession, EstimateNextSessionRotation, Hooks, QueryPreimage},
};
use pallet_verifiers::VkOrHash;
use sp_consensus_babe::Slot;
use sp_core::{crypto::VrfSecret, Pair, H256};
use sp_runtime::{
    traits::{Hash, Header as _},
    AccountId32, Digest, DigestItem,
};
use testsfixtures::get_from_seed;

use super::*;

// Any random values for these constants should do
const BLOCK_NUMBER: BlockNumber = 1;
const SLOT_ID: u64 = 87;
const BABE_AUTHOR_ID: u32 = 1;

// Initialize block #BLOCK_NUMBER, authored at slot SLOT_ID by BABE_AUTHOR_ID using Babe
fn initialize() {
    let slot = Slot::from(SLOT_ID);
    let authority_index = BABE_AUTHOR_ID;
    let transcript = sp_consensus_babe::VrfTranscript::new(b"test", &[]);
    let pair: &sp_consensus_babe::AuthorityPair = &get_from_seed::<BabeId>(
        testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].session_key_seed,
    );
    let vrf_signature = pair.as_ref().vrf_sign(&transcript.into());
    let digest_data = sp_consensus_babe::digests::PreDigest::Primary(
        sp_consensus_babe::digests::PrimaryPreDigest {
            authority_index,
            slot,
            vrf_signature,
        },
    );
    let pre_digest = Digest {
        logs: vec![DigestItem::PreRuntime(
            sp_consensus_babe::BABE_ENGINE_ID,
            digest_data.encode(),
        )],
    };
    System::reset_events();
    System::initialize(&BLOCK_NUMBER, &Default::default(), &pre_digest);
    Babe::on_initialize(BLOCK_NUMBER);
}

fn test() -> sp_io::TestExternalities {
    let mut ex = super::test();
    ex.execute_with(initialize);
    ex
}

mod session {
    use super::*;

    #[test]
    fn uses_babe_session_length() {
        test().execute_with(|| {
            assert_eq!(
                Session::average_session_length(),
                Babe::average_session_length()
            );
        });
    }

    #[test]
    fn notifies_staking() {
        test().execute_with(|| {
            let pre_staking_session = Staking::current_planned_session();
            Session::rotate_session();
            let post_staking_session = Staking::current_planned_session();
            assert_eq!(pre_staking_session + 1, post_staking_session);
        });
    }
}

mod authorship {

    use super::*;

    #[test]
    fn is_configured_with_babe() {
        test().execute_with(|| {
            assert_eq!(
                Authorship::author(),
                Some(AccountId32::new(
                    testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].raw_account
                ))
            );
        });
    }

    // Check that Authorship calls back on ImOnline
    #[test]
    #[cfg(not(feature = "relay"))]
    fn notifies_imonline() {
        test().execute_with(|| {
            assert!(!ImOnline::is_online(BABE_AUTHOR_ID));
            Authorship::on_initialize(BLOCK_NUMBER);
            assert!(ImOnline::is_online(BABE_AUTHOR_ID));
        });
    }

    #[test]
    fn notifies_staking() {
        test().execute_with(|| {
            // Before authoring a block, no points have been given in the active era
            assert!(
                Staking::eras_reward_points(Staking::active_era().expect("No active era").index)
                    .total
                    == 0
            );

            // Pretend we author a block
            Authorship::on_initialize(BLOCK_NUMBER);

            // Authoring a block notifies Staking, which results in a positive points balance
            assert!(
                Staking::eras_reward_points(Staking::active_era().expect("No active era").index)
                    .total
                    > 0
            );
        });
    }
}

mod offences {
    use super::*;
    use frame_system::{EventRecord, Phase};
    use sp_consensus_babe::digests::CompatibleDigestItem;
    use sp_staking::offence::ReportOffence;
    use sp_staking::{offence, Exposure, SessionIndex};

    type OffencesOpaqueTimeSlot = Vec<u8>;

    fn is_offender(
        time_slot: OffencesOpaqueTimeSlot,
        offender_account: &AccountId,
        offence: &[u8; 16],
    ) -> bool {
        pallet_offences::ConcurrentReportsIndex::<Runtime>::get(offence, time_slot)
            .into_iter()
            .any(|offender| {
                pallet_offences::Reports::<Runtime>::get(offender)
                    .expect("Offence not found")
                    .offender
                    .0
                    == *offender_account
            })
    }

    const TEST_SLASH_FRACTION: Perbill = Perbill::one();
    struct TestOffence {
        offender_account: AccountId32,
    }
    impl offence::Offence<(AccountId32, Exposure<AccountId32, u128>)> for TestOffence {
        const ID: offence::Kind = *b"testoffencenooop";
        type TimeSlot = u128;

        fn offenders(&self) -> Vec<(AccountId32, Exposure<AccountId32, u128>)> {
            let exposure =
                pallet_staking::EraInfo::<Runtime>::get_full_exposure(0, &self.offender_account);

            vec![(self.offender_account.clone(), exposure)]
        }
        fn validator_set_count(&self) -> u32 {
            testsfixtures::NUM_VALIDATORS
        }
        fn time_slot(&self) -> Self::TimeSlot {
            0
        }
        fn session_index(&self) -> SessionIndex {
            0
        }
        fn slash_fraction(&self, _offenders_count: u32) -> Perbill {
            TEST_SLASH_FRACTION
        }
    }

    #[test]
    fn notifies_staking() {
        test().execute_with(|| {
            let offender_account = sp_runtime::AccountId32::new(
                testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].raw_account,
            );

            let expected_slashing_event = EventRecord {
                phase: Phase::Initialization,
                event: RuntimeEvent::Staking(pallet_staking::Event::SlashReported {
                    validator: offender_account.clone(),
                    fraction: TEST_SLASH_FRACTION,
                    slash_era: 0,
                }),
                topics: vec![],
            };

            // Make sure that no slash events for offender_account is published
            assert!(!System::events().contains(&expected_slashing_event));

            // Make pallet_offences report an offence
            let offence = TestOffence {
                offender_account: offender_account.clone(),
            };
            assert_ok!(Offences::report_offence(vec![], offence));

            // Check that pallet_staking generates the related event (i.e. it has been notified of
            // the offence)
            assert!(System::events().contains(&expected_slashing_event));
        });
    }

    #[test]
    #[cfg(not(feature = "relay"))]
    fn notified_by_imonline() {
        test().execute_with(|| {
            let session = Session::current_index();
            let offender_account =
                AccountId32::new(testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].raw_account);

            const EQUIVOCATION_KIND: &offence::Kind = b"im-online:offlin";
            // Check that no previous offences were reported
            assert!(!is_offender(
                session.encode(),
                &offender_account,
                EQUIVOCATION_KIND
            ));

            // BABE_AUTHOR_ID is considered offline
            assert!(!ImOnline::is_online(BABE_AUTHOR_ID));

            // Advance to next session w/o offender being online
            System::set_block_number(System::block_number() + 1);
            Session::rotate_session();

            // Check that the offline offence for the last session was received by pallet_offences
            assert!(is_offender(
                session.encode(),
                &offender_account,
                EQUIVOCATION_KIND
            ));
        });
    }

    #[test]
    fn notified_by_grandpa() {
        test().execute_with(|| {
            let offender_account =
                AccountId32::new(testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].raw_account);
            let offender = get_from_seed::<GrandpaId>(
                testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].session_key_seed,
            );

            const EQUIVOCATION_KIND: &offence::Kind = b"grandpa:equivoca";

            let round = 0;
            let set_id = Grandpa::current_set_id();
            let time_slot = pallet_grandpa::TimeSlot { set_id, round };

            // Make sure no previous reports for this offence/offender pair
            assert!(!is_offender(
                time_slot.encode(),
                &offender_account,
                EQUIVOCATION_KIND
            ));

            // Make Grandpa report an offence for a double vote on different hashes for the
            // same target block in the same Grandpa round
            let target_number = BLOCK_NUMBER;
            let create_signed_prevote = |target_hash| {
                let prevote = finality_grandpa::Prevote {
                    target_hash,
                    target_number,
                };
                let prevote_msg = finality_grandpa::Message::Prevote(prevote.clone());
                let payload = sp_consensus_grandpa::localized_payload(round, set_id, &prevote_msg);
                let signed = offender.sign(&payload);
                (prevote, signed)
            };
            let first_vote = create_signed_prevote(H256::random());
            let second_vote = create_signed_prevote(H256::random());
            let equivocation_proof = sp_consensus_grandpa::EquivocationProof::<H256, u32>::new(
                set_id,
                sp_consensus_grandpa::Equivocation::Prevote(finality_grandpa::Equivocation {
                    round_number: round,
                    identity: offender.public(),
                    first: first_vote,
                    second: second_vote,
                }),
            );
            let key = (sp_consensus_grandpa::KEY_TYPE, &offender.public());
            let key_owner_proof = Historical::prove(key).unwrap();

            assert_ok!(Grandpa::report_equivocation_unsigned(
                RuntimeOrigin::none(),
                Box::new(equivocation_proof),
                key_owner_proof,
            ));

            // Check report for this offence/offender pair has been received by pallet_offences
            assert!(is_offender(
                time_slot.encode(),
                &offender_account,
                EQUIVOCATION_KIND
            ));
        });
    }

    #[test]
    fn notified_by_babe() {
        test().execute_with(|| {
            let offender_account =
                AccountId32::new(testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].raw_account);
            let offender = get_from_seed::<BabeId>(
                testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].session_key_seed,
            );

            let seal_header = |mut header: Header| {
                let pre_hash = header.hash();
                let seal = <DigestItem as CompatibleDigestItem>::babe_seal(
                    offender.sign(pre_hash.as_ref()),
                );
                header.digest_mut().push(seal);
                header
            };

            // Produce two different block headers for the same height
            let h1 = seal_header(System::finalize());
            // Need to initialize again
            initialize();
            let h2 = seal_header(System::finalize());

            let slot = Slot::from(SLOT_ID);
            const EQUIVOCATION_KIND: &offence::Kind = b"babe:equivocatio";

            // Make sure no previous reports for this offence/offender pair
            assert!(!is_offender(
                slot.encode(),
                &offender_account,
                EQUIVOCATION_KIND
            ));

            // Make Babe report the offence for authoring two different blocks for the same
            // target height
            let equivocation_proof = sp_consensus_babe::EquivocationProof {
                slot,
                offender: offender.public(),
                first_header: h1,
                second_header: h2,
            };
            let key = (sp_consensus_babe::KEY_TYPE, &offender.public());
            let key_owner_proof = Historical::prove(key).unwrap();

            assert_ok!(Babe::report_equivocation_unsigned(
                RuntimeOrigin::none(),
                Box::new(equivocation_proof),
                key_owner_proof
            ));

            // Check report for this offence/offender pair has been received by pallet_offences
            assert!(is_offender(
                slot.encode(),
                &offender_account,
                EQUIVOCATION_KIND
            ));
        });
    }
}

mod staking {
    use super::*;
    use sp_staking::{
        offence::{OffenceDetails, OnOffenceHandler},
        Exposure,
    };

    #[test]
    fn slashes_go_to_treasury() {
        test().execute_with(|| {
            let offender_account = &sp_runtime::AccountId32::new(
                testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].raw_account,
            );
            let offender_balance =
                testsfixtures::SAMPLE_USERS[BABE_AUTHOR_ID as usize].starting_balance;

            let pre_balance = Balances::free_balance(&Treasury::account_id());

            // Let Staking slash offender's balance
            Staking::on_offence(
                &[OffenceDetails {
                    offender: (
                        offender_account.clone(),
                        Exposure {
                            total: offender_balance,
                            own: offender_balance,
                            others: vec![],
                        },
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(100)],
                0,
            );

            // Check that treasury balance increased
            assert!(pre_balance < Balances::free_balance(&Treasury::account_id()))
        });
    }
}

mod sudo {
    use frame_support::assert_ok;
    use sp_runtime::{AccountId32, MultiAddress};

    use super::*;

    #[test]
    fn and_multisig() {
        test().execute_with(|| {
            // Extract the account IDs from SAMPLE_USERS
            let account_ids: Vec<AccountId32> = testsfixtures::SAMPLE_USERS
                .iter()
                .map(|user| AccountId32::from(user.raw_account))
                .collect();
            let multi = Multisig::multi_account_id(&account_ids[..3], 2);

            assert_ok!(Balances::transfer_allow_death(
                RuntimeOrigin::signed(account_ids[0].clone()),
                MultiAddress::Id(multi.clone()),
                1 * currency::ACME
            ));

            // Setting the multisig account as the new sudo account
            assert_ok!(Sudo::set_key(
                RuntimeOrigin::signed(account_ids[0].clone()),
                MultiAddress::Id(multi.clone())
            ));

            // Prepare a sudo call to change the sudo key again to a new account (account_ids[1])
            let sudo_call = pallet_sudo::Call::set_key {
                new: MultiAddress::Id(account_ids[1].clone()),
            };

            let sudo_call_weight = sudo_call.get_dispatch_info().weight;

            // First part of multisig approval (propose the sudo call)
            assert_ok!(Multisig::as_multi(
                RuntimeOrigin::signed(account_ids[0].clone()),
                2,
                vec![account_ids[1].clone(), account_ids[2].clone()],
                None,
                Box::new(RuntimeCall::Sudo(sudo_call.clone())),
                Weight::zero()
            ));
            // Second part of multisig approval (approve the sudo call)
            assert_ok!(Multisig::as_multi(
                RuntimeOrigin::signed(account_ids[1].clone()),
                2,
                vec![account_ids[0].clone(), account_ids[2].clone()],
                Some(Multisig::timepoint()),
                Box::new(RuntimeCall::Sudo(sudo_call.clone())),
                sudo_call_weight
            ));

            // Ensure the sudo key has been updated correctly to account_ids[1]
            assert_ok!(Sudo::remove_key(RuntimeOrigin::signed(
                account_ids[1].clone()
            )));
        });
    }
}

mod scheduler {
    use super::*;

    #[test]
    fn uses_preimage() {
        test().execute_with(|| {
            // We need a call bigger than 128 bytes to trigger preimage usage
            let call = Box::new(
                RuntimeCall::SettlementFFlonkPallet(pallet_verifiers::Call::<
                    Runtime,
                    pallet_fflonk_verifier::Fflonk,
                >::new_call_variant_submit_proof(
                    VkOrHash::from_hash(H256::zero()),
                    [0; pallet_fflonk_verifier::PROOF_SIZE].into(),
                    [0; pallet_fflonk_verifier::PUBS_SIZE].into(),
                    None,
                )),
            );
            let call_hash = <Runtime as frame_system::Config>::Hashing::hash_of(&call);

            assert_ok!(Scheduler::schedule(
                RuntimeOrigin::root(),
                100,
                None,
                0,
                call
            ));

            assert!(Preimage::len(&call_hash).is_some());
        });
    }
}

mod claim {
    use super::*;

    #[test]
    fn unclaimed_go_to_treasury() {
        test().execute_with(|| {
            let pre_balance = Balances::free_balance(&Treasury::account_id());

            assert_ok!(Claim::end_airdrop(RuntimeOrigin::root()));

            // Check that treasury balance increased
            assert_eq!(
                pre_balance.saturating_add(testsfixtures::TOTAL_BALANCE.clone()),
                Balances::free_balance(&Treasury::account_id())
            )
        });
    }
}
