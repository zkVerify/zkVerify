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

//! Here we write the integration tests that just check pallets weighs are correctly linked.

use crate::{OnChainSeqPhragmen, Runtime};

mod verifiers;

#[test]
fn frame_system() {
    use frame_system::WeightInfo;

    assert_eq!(
        <Runtime as frame_system::Config>::SystemWeightInfo::set_heap_pages(),
        crate::weights::frame_system::ZKVWeight::<Runtime>::set_heap_pages()
    );
}

#[test]
fn db() {
    assert_eq!(
        <Runtime as frame_system::Config>::DbWeight::get(),
        crate::weights::db::constants::RocksDbWeight::get()
    );
}

#[test]
fn pallet_babe() {
    use pallet_babe::WeightInfo;

    assert_eq!(
        <Runtime as pallet_babe::Config>::WeightInfo::report_equivocation(42, 42),
        crate::weights::pallet_babe::ZKVWeight::<Runtime>::report_equivocation(42, 42)
    );
}

#[test]
fn pallet_grandpa() {
    use pallet_grandpa::WeightInfo;

    assert_eq!(
        <Runtime as pallet_grandpa::Config>::WeightInfo::report_equivocation(42, 42),
        crate::weights::pallet_grandpa::ZKVWeight::<Runtime>::report_equivocation(42, 42)
    );
}

#[test]
fn pallet_balances() {
    use pallet_balances::WeightInfo;

    assert_eq!(
        <Runtime as pallet_balances::Config>::WeightInfo::transfer_allow_death(),
        crate::weights::pallet_balances::ZKVWeight::<Runtime>::transfer_allow_death()
    );
}

#[test]
fn pallet_session() {
    use pallet_session::WeightInfo;

    assert_eq!(
        <Runtime as pallet_session::Config>::WeightInfo::set_keys(),
        crate::weights::pallet_session::ZKVWeight::<Runtime>::set_keys()
    );
}

#[test]
fn frame_election_provider_support() {
    use frame_election_provider_support::WeightInfo;

    assert_eq!(
        <OnChainSeqPhragmen as frame_election_provider_support::onchain::Config>::WeightInfo::phragmen(42, 42, 42),
        crate::weights::frame_election_provider_support::ZKVWeight::<Runtime>::phragmen(42, 42, 42)
    );
}

#[test]
fn pallet_sudo() {
    use pallet_sudo::WeightInfo;

    assert_eq!(
        <Runtime as pallet_sudo::Config>::WeightInfo::sudo(),
        crate::weights::pallet_sudo::ZKVWeight::<Runtime>::sudo()
    );
}

#[test]
fn pallet_multisig() {
    use pallet_multisig::WeightInfo;

    assert_eq!(
        <Runtime as pallet_multisig::Config>::WeightInfo::as_multi_approve(3, 100),
        crate::weights::pallet_multisig::ZKVWeight::<Runtime>::as_multi_approve(3, 100)
    );
}

#[test]
fn pallet_bags_list() {
    use pallet_bags_list::WeightInfo;

    assert_eq!(
        <Runtime as pallet_bags_list::Config<pallet_bags_list::Instance1>>::WeightInfo::put_in_front_of(),
        crate::weights::pallet_bags_list::ZKVWeight::<Runtime>::put_in_front_of()
    );
}

#[test]
fn pallet_utility() {
    use pallet_utility::WeightInfo;

    assert_eq!(
        <Runtime as pallet_utility::Config>::WeightInfo::dispatch_as(),
        crate::weights::pallet_utility::ZKVWeight::<Runtime>::dispatch_as()
    );
}

#[test]
fn pallet_vesting() {
    use pallet_vesting::WeightInfo;

    assert_eq!(
        <Runtime as pallet_vesting::Config>::WeightInfo::force_vested_transfer(1, 2),
        crate::weights::pallet_vesting::ZKVWeight::<Runtime>::force_vested_transfer(1, 2)
    );
}

#[test]
fn pallet_preimage() {
    use pallet_preimage::WeightInfo;

    assert_eq!(
        <Runtime as pallet_preimage::Config>::WeightInfo::note_preimage(100),
        crate::weights::pallet_preimage::ZKVWeight::<Runtime>::note_preimage(100)
    );
}

#[test]
fn pallet_scheduler() {
    use pallet_scheduler::WeightInfo;

    assert_eq!(
        <Runtime as pallet_scheduler::Config>::WeightInfo::schedule(10),
        crate::weights::pallet_scheduler::ZKVWeight::<Runtime>::schedule(10)
    );
}

#[test]
fn pallet_referenda() {
    use pallet_referenda::WeightInfo;

    assert_eq!(
        <Runtime as pallet_referenda::Config>::WeightInfo::submit(),
        crate::weights::pallet_referenda::ZKVWeight::<Runtime>::submit()
    );
}

#[test]
fn pallet_whitelist() {
    use pallet_whitelist::WeightInfo;

    assert_eq!(
        <Runtime as pallet_whitelist::Config>::WeightInfo::whitelist_call(),
        crate::weights::pallet_whitelist::ZKVWeight::<Runtime>::whitelist_call()
    );
}

#[test]
fn pallet_conviction_voting() {
    use pallet_conviction_voting::WeightInfo;

    assert_eq!(
        <Runtime as pallet_conviction_voting::Config>::WeightInfo::vote_new(),
        crate::weights::pallet_conviction_voting::ZKVWeight::<Runtime>::vote_new()
    );
}

#[test]
fn pallet_treasury() {
    use pallet_treasury::WeightInfo;

    assert_eq!(
        <Runtime as pallet_treasury::Config>::WeightInfo::check_status(),
        crate::weights::pallet_treasury::ZKVWeight::<Runtime>::check_status()
    );
}

#[test]
fn pallet_bounties() {
    use pallet_bounties::WeightInfo;

    assert_eq!(
        <Runtime as pallet_bounties::Config>::WeightInfo::propose_curator(),
        crate::weights::pallet_bounties::ZKVWeight::<Runtime>::propose_curator()
    );
}

#[test]
fn pallet_timestamp() {
    use pallet_timestamp::WeightInfo;

    assert_eq!(
        <Runtime as pallet_timestamp::Config>::WeightInfo::set(),
        crate::weights::pallet_timestamp::ZKVWeight::<Runtime>::set()
    );
}

#[test]
#[cfg(not(feature = "relay"))]
fn pallet_im_online() {
    use pallet_im_online::WeightInfo;

    assert_eq!(
        <Runtime as pallet_im_online::Config>::WeightInfo::validate_unsigned_and_then_heartbeat(42),
        crate::weights::pallet_im_online::ZKVWeight::<Runtime>::validate_unsigned_and_then_heartbeat(42)
    );
}

#[test]
fn pallet_proxy() {
    use pallet_proxy::WeightInfo;

    assert_eq!(
        <Runtime as pallet_proxy::Config>::WeightInfo::create_pure(1),
        crate::weights::pallet_proxy::ZKVWeight::<Runtime>::create_pure(1)
    );
}

#[test]
fn pallet_poe() {
    use pallet_poe::WeightInfo;

    assert_eq!(
        <Runtime as pallet_poe::Config>::WeightInfo::publish_attestation(),
        crate::weights::pallet_poe::ZKVWeight::<Runtime>::publish_attestation()
    );
}

#[test]
fn pallet_aggregate() {
    use pallet_aggregate::WeightInfo;

    assert_eq!(
        <Runtime as pallet_aggregate::Config>::WeightInfo::register_domain(),
        crate::weights::pallet_aggregate::ZKVWeight::<Runtime>::register_domain()
    );
}

#[test]
fn pallet_hyperbridge_aggregations() {
    use pallet_hyperbridge_aggregations::WeightInfo;

    assert_eq!(
        <Runtime as pallet_hyperbridge_aggregations::Config>::WeightInfo::dispatch_aggregation(),
        crate::weights::pallet_hyperbridge_aggregations::ZKVWeight::<Runtime>::dispatch_aggregation(
        )
    );
}

#[test]
fn ismp_grandpa() {
    use ismp_grandpa::WeightInfo;

    assert_eq!(
        <Runtime as ismp_grandpa::Config>::WeightInfo::add_state_machines(42),
        crate::weights::ismp_grandpa::ZKVWeight::<Runtime>::add_state_machines(42)
    );
}

#[test]
fn pallet_staking() {
    use pallet_staking::WeightInfo;

    assert_eq!(
        <Runtime as pallet_staking::Config>::WeightInfo::bond(),
        crate::weights::pallet_staking::ZKVWeight::<Runtime>::bond()
    );
}

#[test]
fn pallet_verifiers() {
    use pallet_verifiers::common::WeightInfo;

    assert_eq!(
        <Runtime as pallet_verifiers::common::Config>::CommonWeightInfo::on_verify_disabled_verifier(),
        <Runtime as pallet_verifiers::common::WeightInfo>::on_verify_disabled_verifier()
    );
}

#[cfg(feature = "relay")]
#[test]
fn pallet_message_queue() {
    use pallet_message_queue::WeightInfo;

    assert_eq!(
            <<Runtime as pallet_message_queue::Config>::WeightInfo as pallet_message_queue::WeightInfo>::ready_ring_knit(),
            crate::weights::pallet_message_queue::ZKVWeight::<Runtime>::ready_ring_knit()
        )
}

mod parachains {
    #![cfg(feature = "relay")]

    use super::*;
    use crate::parachains::*;

    #[test]
    fn configuration() {
        use configuration::WeightInfo;

        assert_eq!(
            <<Runtime as configuration::Config>::WeightInfo as
            configuration::WeightInfo>::set_config_with_block_number(),
            crate::weights::parachains::configuration::ZKVWeight::<Runtime>::set_config_with_block_number()
        )
    }

    #[test]
    fn disputes() {
        use disputes::WeightInfo;

        assert_eq!(
            <<Runtime as disputes::Config>::WeightInfo as disputes::WeightInfo>::force_unfreeze(),
            crate::weights::parachains::disputes::ZKVWeight::<Runtime>::force_unfreeze()
        )
    }

    #[test]
    fn hrmp() {
        use hrmp::WeightInfo;

        assert_eq!(
            <<Runtime as hrmp::Config>::WeightInfo as hrmp::WeightInfo>::hrmp_init_open_channel(),
            crate::weights::parachains::hrmp::ZKVWeight::<Runtime>::hrmp_init_open_channel()
        )
    }

    #[test]
    fn initializer() {
        use initializer::WeightInfo;

        assert_eq!(
            <<Runtime as initializer::Config>::WeightInfo as initializer::WeightInfo>::force_approve(42),
            crate::weights::parachains::initializer::ZKVWeight::<Runtime>::force_approve(42)
        )
    }

    #[test]
    fn paras_inherent() {
        use paras_inherent::WeightInfo;

        assert_eq!(
            <<Runtime as paras_inherent::Config>::WeightInfo as paras_inherent::WeightInfo>::enter_bitfields(),
            crate::weights::parachains::paras_inherent::ZKVWeight::<Runtime>::enter_bitfields()
        )
    }

    #[test]
    fn paras() {
        use paras::WeightInfo;

        assert_eq!(
            <<Runtime as paras::Config>::WeightInfo as paras::WeightInfo>::force_set_most_recent_context(),
            crate::weights::parachains::paras::ZKVWeight::<Runtime>::force_set_most_recent_context()
        )
    }

    #[test]
    fn inclusion() {
        use inclusion::WeightInfo;

        assert_eq!(
            <<Runtime as inclusion::Config>::WeightInfo as inclusion::WeightInfo>::receive_upward_messages(42),
            crate::weights::parachains::inclusion::ZKVWeight::<Runtime>::receive_upward_messages(42)
        )
    }

    #[test]
    fn slashing() {
        use slashing::WeightInfo;

        assert_eq!(
            <<Runtime as slashing::Config>::WeightInfo as slashing::WeightInfo>::report_dispute_lost(12),
            crate::weights::parachains::slashing::ZKVWeight::<Runtime>::report_dispute_lost(12)
        )
    }
}
