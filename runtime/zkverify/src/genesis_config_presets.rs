// Copyright 2024, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.

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

#![allow(clippy::type_complexity)]

use crate::{currency::Balance, types::AccountId, SessionKeys, BABE_GENESIS_EPOCH_CONFIG};
use alloc::{boxed::Box, vec, vec::Vec};
use helper::*;
use polkadot_primitives::{
    node_features, ApprovalVotingParams, AssignmentId, AsyncBackingParams, AuthorityDiscoveryId,
    BlockNumber, NodeFeatures, SchedulerParams, ValidatorId,
};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::Ss58Codec, sr25519};
use sp_genesis_builder::PresetId;

mod helper;

fn session_keys(
    babe: BabeId,
    grandpa: GrandpaId,
    para_validator: ValidatorId,
    para_assignment: AssignmentId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        babe,
        grandpa,
        para_validator,
        para_assignment,
        authority_discovery,
    }
}

fn parachains_host_configuration() -> HostConfiguration<BlockNumber> {
    use polkadot_primitives::{MAX_CODE_SIZE, MAX_POV_SIZE};

    let mut node_features = NodeFeatures::new();
    node_features.resize(
        node_features::FeatureIndex::FirstUnassigned as usize + 1,
        false,
    );
    node_features.set(
        node_features::FeatureIndex::CandidateReceiptV2 as u8 as usize,
        true,
    );

    HostConfiguration {
        validation_upgrade_cooldown: 2u32, // Don't care for now: we work only with system (thus, trusted) parachains
        validation_upgrade_delay: 2, // Don't care for now: we work only with system (thus, trusted) parachains
        code_retention_period: 3600,
        max_code_size: MAX_CODE_SIZE,
        max_pov_size: MAX_POV_SIZE,
        max_head_data_size: 20 * 1024,
        max_upward_queue_count: 128,
        max_upward_queue_size: 128 * 1024, // MaxUpwardQueueCount * MaxUpwardMessageSize * 2
        max_downward_message_size: 10 * 1024,
        max_upward_message_size: 1024,
        max_upward_message_num_per_candidate: 64,
        hrmp_sender_deposit: 0,                     // Don't need HRMP for now
        hrmp_recipient_deposit: 0,                  // Don't need HRMP for now
        hrmp_channel_max_capacity: 8,               // Don't need HRMP for now
        hrmp_channel_max_total_size: 8 * 1024,      // Don't need HRMP for now
        hrmp_max_parachain_inbound_channels: 4,     // Don't need HRMP for now
        hrmp_channel_max_message_size: 1024 * 1024, // Don't need HRMP for now
        hrmp_max_parachain_outbound_channels: 4,    // Don't need HRMP for now
        hrmp_max_message_num_per_candidate: 5,      // Don't need HRMP for now
        dispute_period: 6,
        no_show_slots: 3,
        n_delay_tranches: 25,
        needed_approvals: 3,
        relay_vrf_modulo_samples: 2,
        zeroth_delay_tranche_width: 0,
        minimum_validation_upgrade_delay: 5, // Don't care for now: we work only with system (thus, trusted) parachains
        minimum_backing_votes: 2,
        async_backing_params: AsyncBackingParams {
            max_candidate_depth: 3,
            allowed_ancestry_len: 2,
        },
        scheduler_params: SchedulerParams {
            lookahead: 2,
            max_validators_per_core: Some(5),
            ..Default::default() // Deprecated and unused params, assigning default values
        },
        executor_params: Default::default(),
        max_validators: None, // No Max
        dispute_post_conclusion_acceptance_period: 50,
        pvf_voting_ttl: 2,
        node_features,
        approval_voting_params: ApprovalVotingParams {
            max_approval_coalesce_count: 1,
        },
    }
}

pub fn local_config_genesis() -> serde_json::Value {
    let balances = DEFAULT_ENDOWED_SEEDS
        .into_iter()
        .map(|seed| {
            FundedAccount::from_id(
                get_account_id_from_seed::<sr25519::Public>(seed)
                    .to_ss58check()
                    .as_str(),
                ENDOWMENT,
            )
            .expect("Invalid seed")
        })
        .collect::<Vec<_>>();

    let authorities_num = 2;
    let initial_authorities = DEFAULT_ENDOWED_SEEDS
        .iter()
        .take(authorities_num)
        .map(|seed| {
            ValidatorData::from_ids(
                get_account_id_from_seed::<sr25519::Public>(seed)
                    .to_ss58check()
                    .as_str(),
                get_from_seed::<BabeId>(seed).to_ss58check().as_str(),
                get_from_seed::<GrandpaId>(seed).to_ss58check().as_str(),
                ENDOWMENT,
                STASH_BOND,
            )
            .expect("Invalid seed")
        })
        .collect::<Vec<_>>();

    genesis(
        // Initial PoA authorities
        initial_authorities
            .iter()
            .map(ValidatorData::ids)
            .collect::<Vec<_>>(),
        // Sudo account
        get_account_id_from_seed::<sr25519::Public>(DEFAULT_ENDOWED_SEEDS[0]),
        // Pre-funded accounts
        balances
            .iter()
            .map(FundedAccount::json_data)
            .collect::<Vec<_>>(),
        initial_authorities
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn StakerData>)
            .collect(),
        // min validator count
        2,
        // ideal validator count
        5,
        // max validator count
        None,
        // min validator bond
        0,
        // min nominator bond
        0,
        parachains_host_configuration(),
    )
}

pub fn development_config_genesis() -> serde_json::Value {
    let balances = DEFAULT_ENDOWED_SEEDS
        .into_iter()
        .map(|seed| {
            FundedAccount::from_id(
                get_account_id_from_seed::<sr25519::Public>(seed)
                    .to_ss58check()
                    .as_str(),
                ENDOWMENT,
            )
            .expect("Invalid seed")
        })
        .chain([
            // The following is a workaround for pallet_treasury benchmarks which hardcode
            // a payment of 100 (lower than EXISTENTIAL_DEPOSIT) to a given address ([0x0])
            #[cfg(feature = "runtime-benchmarks")]
            (FundedAccount::from_id(
                "5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM",
                ENDOWMENT,
            )
            .expect("Address not valid")),
        ])
        .collect::<Vec<_>>();

    let authorities_num = 1;
    let initial_authorities = DEFAULT_ENDOWED_SEEDS
        .iter()
        .take(authorities_num)
        .map(|seed| {
            ValidatorData::from_ids(
                get_account_id_from_seed::<sr25519::Public>(seed)
                    .to_ss58check()
                    .as_str(),
                get_from_seed::<BabeId>(seed).to_ss58check().as_str(),
                get_from_seed::<GrandpaId>(seed).to_ss58check().as_str(),
                ENDOWMENT,
                STASH_BOND,
            )
            .expect("Invalid seed")
        })
        .collect::<Vec<_>>();

    genesis(
        // Initial PoA authorities
        initial_authorities
            .iter()
            .map(ValidatorData::ids)
            .collect::<Vec<_>>(),
        // Sudo account
        get_account_id_from_seed::<sr25519::Public>(DEFAULT_ENDOWED_SEEDS[0]),
        // Pre-funded accounts
        balances
            .iter()
            .map(FundedAccount::json_data)
            .collect::<Vec<_>>(),
        initial_authorities
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn StakerData>)
            .collect(),
        // min validator count
        1,
        // ideal validator count
        3,
        // max validator count
        None,
        // min validator bond
        0,
        // min nominator bond
        0,
        parachains_host_configuration(),
    )
}

pub fn preset_names() -> Vec<PresetId> {
    vec![PresetId::from("development"), PresetId::from("local")]
}

pub fn get_preset(id: &sp_genesis_builder::PresetId) -> Option<Vec<u8>> {
    let cfg = match id.as_ref() {
        "development" => development_config_genesis(),
        "local" => local_config_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&cfg)
            .expect("genesis cfg must be serializable. qed.")
            .into_bytes(),
    )
}

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn genesis(
    initial_authorities: Vec<Ids>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, Balance)>,
    stakers: Vec<Box<dyn StakerData>>,
    min_validator_count: u32,
    validator_count: u32,
    max_validator_count: Option<u32>,
    min_validator_bond: Balance,
    min_nominator_bond: Balance,
    parachains_host_configuration: HostConfiguration<BlockNumber>,
) -> serde_json::Value {
    serde_json::json!({
        "balances": {
            // Configure endowed accounts with initial balance.
            "balances": endowed_accounts,
        },
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "session": {
            "keys": initial_authorities.iter()
                .cloned()
                .map(|(account, babe, grandpa, para, assign, auth)| { (account.clone(), account, session_keys(babe, grandpa, para, assign, auth)) })
                .collect::<Vec<_>>(),
        },
        "staking": {
            "minimumValidatorCount": min_validator_count,
            "maxValidatorCount": max_validator_count,
            "minValidatorBond": min_validator_bond,
            "minNominatorBond": min_nominator_bond,
            "validatorCount": validator_count,
            "stakers": stakers.iter()
                .map(StakerData::staker_data)
                .collect::<Vec<_>>(),
        },
        "sudo": {
            // Assign network admin rights.
            "key": Some(root_key),
        },
        "configuration": {
            "config": parachains_host_configuration,
        },

    })
}
