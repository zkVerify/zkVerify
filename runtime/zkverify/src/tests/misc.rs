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

//! Here we write miscellaneous tests that don't fall in the other categories.

use frame_support::traits::ExistenceRequirement;
use pallet_verifiers::VkOrHash;

use super::*;

// Test definition and execution. Test body must be written in the execute_with closure.
#[test]
fn check_starting_balances_and_existential_limit() {
    test().execute_with(|| {
        use frame_support::traits::{fungible::Inspect, Currency};
        // This creates a few public keys used to be converted to AccountId

        for sample_user in testsfixtures::SAMPLE_USERS {
            assert_eq!(
                Balances::balance(&sample_user.raw_account.into()),
                sample_user.starting_balance
            );
        }

        // Now perform a withdraw on the fourth account, leaving its balance under the EXISTENTIAL_DEPOSIT limit
        // This should kill the account, when executed with the ExistenceRequirement::AllowDeath option
        let _id_3_withdraw = Balances::withdraw(
            &sample_user_account(3),
            testsfixtures::EXISTENTIAL_DEPOSIT_REMAINDER, // Withdrawing more th
            WithdrawReasons::TIP,
            ExistenceRequirement::AllowDeath,
        );

        // Verify that the fourth account balance is now 0
        assert_eq!(Balances::balance(&sample_user_account(3)), 0);
    });
}

#[test]
fn submit_proof_weights_composition() {
    test().execute_with(|| {
        use frame_support::dispatch::GetDispatchInfo;
        use pallet_aggregate::WeightInfo;
        use pallet_groth16_verifier::Groth16;

        let info =
            pallet_verifiers::Call::<Runtime, pallet_groth16_verifier::Groth16<Runtime>>::submit_proof {
                vk_or_hash: VkOrHash::from_hash(H256::zero()),
                proof: pallet_groth16_verifier::Proof::default().into(),
                pubs: Box::new(Vec::new()),
                domain_id: Some(2),
            }
            .get_dispatch_info();
        let ref_time = info.call_weight.ref_time();
        let proof_size = info.call_weight.proof_size();

        let verify_time = <<Runtime as pallet_verifiers::Config<Groth16<Runtime>>>::WeightInfo as
            pallet_verifiers::WeightInfo<Groth16<Runtime>>>
            ::verify_proof(
            &pallet_groth16_verifier::Proof::default(),
            &Vec::new()
        ).ref_time();

        // We don't want here check the complete logic for the ref time (unit tests should be enough)
        assert!(ref_time > verify_time);
        assert_eq!(proof_size, <Runtime as pallet_aggregate::Config>::WeightInfo::on_proof_verified().proof_size());
    });
}

#[test]
fn submit_proof_weights_composition_should_ignore_aggregate_if_no_domain() {
    test().execute_with(|| {
        use frame_support::dispatch::GetDispatchInfo;
        use pallet_aggregate::WeightInfo;

        let info =
            pallet_verifiers::Call::<Runtime, pallet_groth16_verifier::Groth16<Runtime>>::submit_proof {
                vk_or_hash: VkOrHash::from_hash(H256::zero()),
                proof: pallet_groth16_verifier::Proof::default().into(),
                pubs: Box::new(Vec::new()),
                domain_id: None,
            }
            .get_dispatch_info();

        let proof_size = info.call_weight.proof_size();

        // We check that is lesser than half of the aggregate weight... just a reference to be sure that not use it
        assert!(proof_size < <Runtime as pallet_aggregate::Config>::WeightInfo::on_proof_verified().proof_size()/2);
    });
}

mod aggregate {
    use super::*;
    use frame_support::traits::fungible::InspectHold;
    use pallet_aggregate::*;

    #[test]
    fn hold_expected_amount_for_domain() {
        test().execute_with(|| {
            let register_account = sample_user_account(0);
            //Sanity check
            assert_eq!(
                0,
                Balances::balance_on_hold(&AggregateDomainHoldReason::get(), &register_account)
            );

            Aggregate::register_domain(
                RuntimeOrigin::signed(register_account.clone()),
                16,
                None,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::Untrusted,
                Default::default(),
                None,
            )
            .unwrap();

            let hold =
                Balances::balance_on_hold(&AggregateDomainHoldReason::get(), &register_account);

            // Sure that is greater than domain minimum size and less than domain maximum size (approx)
            assert!(
                hold > AggregateDomainBaseDeposit::get() + AggregateDomainByteDeposit::get() * 1500
            );
            assert!(
                hold < AggregateDomainBaseDeposit::get()
                    + AggregateDomainByteDeposit::get() * 40000
            );
        });
    }

    #[test]
    fn hold_expected_amount_for_allow_list() {
        test().execute_with(|| {
            let register_account = sample_user_account(0);
            //Sanity check
            assert_eq!(
                0,
                Balances::balance_on_hold(&AggregateAllowlistHoldReason::get(), &register_account)
            );

            Aggregate::register_domain(
                RuntimeOrigin::signed(register_account.clone()),
                16,
                None,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::Untrusted,
                Default::default(),
                None,
            )
            .unwrap();

            //Sanity check
            assert_eq!(
                0,
                Balances::balance_on_hold(&AggregateAllowlistHoldReason::get(), &register_account)
            );

            Aggregate::register_domain(
                RuntimeOrigin::signed(register_account.clone()),
                16,
                None,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::OnlyOwner,
                Default::default(),
                None,
            )
            .unwrap();

            //Sanity check
            assert_eq!(
                0,
                Balances::balance_on_hold(&AggregateAllowlistHoldReason::get(), &register_account)
            );

            Aggregate::register_domain(
                RuntimeOrigin::signed(register_account.clone()),
                16,
                None,
                AggregateSecurityRules::Untrusted,
                ProofSecurityRules::OnlyAllowlisted,
                Default::default(),
                None,
            )
            .unwrap();
            let domain_id = 2;

            let hold =
                Balances::balance_on_hold(&AggregateAllowlistHoldReason::get(), &register_account);
            // Just the base
            assert_eq!(hold, AggregateAllowlistHoldBaseDeposit::get());

            Aggregate::allowlist_proof_submitters(
                RuntimeOrigin::signed(register_account.clone()),
                domain_id,
                vec![
                    sample_user_account(1),
                    sample_user_account(2),
                    sample_user_account(3),
                    sample_user_account(1),
                ],
            )
            .unwrap();

            let hold =
                Balances::balance_on_hold(&AggregateAllowlistHoldReason::get(), &register_account);
            // Add 3 account
            assert_eq!(
                hold,
                AggregateAllowlistHoldBaseDeposit::get()
                    + 3 * AggregateAllowlistHoldSingleElementDeposit::get()
            );

            Aggregate::remove_proof_submitters(
                RuntimeOrigin::signed(register_account.clone()),
                domain_id,
                vec![sample_user_account(2), sample_user_account(1)],
            )
            .unwrap();

            let hold =
                Balances::balance_on_hold(&AggregateAllowlistHoldReason::get(), &register_account);
            // Just one remained
            assert_eq!(
                hold,
                AggregateAllowlistHoldBaseDeposit::get()
                    + AggregateAllowlistHoldSingleElementDeposit::get()
            );
        });
    }
}

#[test]
fn check_version() {
    let v_str = std::env!("CARGO_PKG_VERSION");
    let convert = |v: &str| {
        v.split('.')
            .map(|x| x.parse::<u32>().unwrap())
            .rev()
            .enumerate()
            .fold(0, |a, (step, dec)| a + dec * 1000_u32.pow(step as u32))
    };

    let v_num = convert(v_str);
    use sp_api::runtime_decl_for_core::CoreV5;
    let s_ver = Runtime::version().spec_version;
    assert_eq!(
        s_ver, v_num,
        "Version mismatch. Crate version = {v_str}, but spec_version is {s_ver} != {v_num}"
    );

    // Sanity checks
    assert_eq!(1_002_003, convert("1.2.3"));
    assert_eq!(3_002_001, convert("3.2.1"));
    assert_eq!(0, convert("0.0.0"));
    assert_eq!(5_010, convert("0.5.10"));
    assert_eq!(1_000_000, convert("1.0.0"));
}
