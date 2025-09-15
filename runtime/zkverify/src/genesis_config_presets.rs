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

use crate::*;
use alloc::{boxed::Box, vec, vec::Vec};
use polkadot_primitives::{
    AssignmentId, AsyncBackingParams, BlockNumber, SchedulerParams, ValidatorId,
};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::crypto::Ss58Codec;
use sp_core::sr25519;
use sp_genesis_builder::PresetId;

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

fn main_parachains_host_configuration() -> HostConfiguration<BlockNumber> {
    use polkadot_primitives::{MAX_CODE_SIZE, MAX_POV_SIZE};

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
        node_features: NodeFeatures::EMPTY,
        approval_voting_params: ApprovalVotingParams {
            max_approval_coalesce_count: 1,
        },
    }
}

pub fn staging_config_genesis() -> Result<serde_json::Value, sp_core::crypto::PublicError> {
    const EDS_LOCKED_BALANCE: Balance = 271 * MILLIONS + 300 * THOUSANDS + VFY;
    const ZKV_FOUNDATION_LOCKED_BALANCE: Balance = 133 * MILLIONS + 162 * THOUSANDS + 741 * VFY;
    const ZKV_FOUNDATION_LIQUID_BALANCE: Balance = 116 * MILLIONS + 592 * THOUSANDS + 379 * VFY;
    const ZKV_COMMUNITY_LOCKED_BALANCE: Balance = 264 * MILLIONS + 343 * THOUSANDS + 112 * VFY;
    const ZKV_COMMUNITY_LIQUID_BALANCE: Balance = 108 * MILLIONS + 721 * THOUSANDS + 765 * VFY;
    const TOKEN_LAUNCH_LOCKED_BALANCE: Balance = 79 * MILLIONS + 950 * THOUSANDS + VFY;
    const HORIZEN_LABS_LOCKED_BALANCE: Balance = 25 * MILLIONS + VFY;
    const VALIDATOR_BALANCE: Balance = 100 * THOUSANDS;
    const VALIDATOR_BOND: Balance = VALIDATOR_BALANCE;
    const SUDO_BALANCE: Balance = 10 * THOUSANDS;
    const DEV_WALLET_BALANCE: Balance = 10 * THOUSANDS;

    let institutional = [
        //EDS Locked
        FundedAccount::from_id(
            "5E2bar9bJQfCm7bdBFbTD6j5AGfuEPbWohCQL8cNgvRppBxN",
            EDS_LOCKED_BALANCE,
        )?,
        //ZKV Foundation Locked
        FundedAccount::from_id(
            "5H4GcaHqZN8t2sg25dx71BZUQyp5bkNeAECtTv2hbQMZL2d9",
            ZKV_FOUNDATION_LOCKED_BALANCE,
        )?,
        //ZKV Foundation Liquid
        FundedAccount::from_id(
            "5H7BYine8UZrd8aWfhGr3ANYHtrqZpo47WBuAjrvArWzs6TU",
            ZKV_FOUNDATION_LIQUID_BALANCE,
        )?,
        //ZKV Community Locked
        FundedAccount::from_id(
            "5HnS3BKnDXSgpoyYdtSJTuKuiU3mqnhzWeez4WSTpjYsPgvD",
            ZKV_COMMUNITY_LOCKED_BALANCE,
        )?,
        //ZKV Community Liquid
        FundedAccount::from_id(
            "5EtxUrj7UrXJWSGxaSyBz1dGLRXP5t1Q1rDq2t7Uw8PWrpwE",
            ZKV_COMMUNITY_LIQUID_BALANCE,
        )?,
        //Token Launch Locked
        FundedAccount::from_id(
            "5EaXuxjzL29Hx4oUauScAUCeAX9Nhy7Gq82FesrSfiADrK9N",
            TOKEN_LAUNCH_LOCKED_BALANCE,
        )?,
        //Horizen Labs Locked
        FundedAccount::from_id(
            "5CKQcFAbn2MGaV8D6XhBt84JkPRGWBD1jhbvkhbQcLWp2R32",
            HORIZEN_LABS_LOCKED_BALANCE,
        )?,
    ];

    const VALIDATORS_ADDRESS: &[(&str, &str)] = &[
        (
            "5HinfZ1FcDLWp5hpMWQyuuTHPmrp2oSaivpZdhbL33kt8dXh",
            "5E1891vFuA2DhdzRk5LCQenReaMtsQPUiQ2BAwGrh7oD93vv",
        ),
        (
            "5EAS2dekMepc8T5DUQ9MSWSegzymQLee7NeMV59pYbjYEbmQ",
            "5F2saXmrUVN9CfXLese8iGKAuVwEyJ8K3Q2NFrhgzsmwVnpn",
        ),
        (
            "5Hozb8oD4gfhYwwcxFM2RZv81mbtEtqUetkjbJNoLCmFiHPa",
            "5C8FxmWrrCG5FnSjch56cGEY7ktypi7vzg8SSXD3xWvECu6n",
        ),
        (
            "5F2DhCx7XM3c3PbzeiTT9oqpRBPmj64EHGqy6epSDnLNWPSB",
            "5DZweKpPQtVEbw4uof2cEDoZXGRz8rdFNwKGqhvnFfACrMJw",
        ),
        (
            "5FhkW4X1KYU1fRStCTL4VoG4oMarB9ZM6gUS56Hf8XdyFF3K",
            "5Gj3xgnHKh6CuCwhqsrVUZZezX6aEKyQPAvLq1SsjDq86GyX",
        ),
        (
            "5GQxAhBih3uPtXnqnshrPSgWa3QvaDZbrUJ9wZCwTfydhWhU",
            "5GRCL51gzF94gyGFsTSmeUqWH3fu69Jdka9bJSVtJrLFGzSr",
        ),
        (
            "5CiAdXr8gnCi3VnxcyRftVBBH82TDXEnNSBxdBdAfBBRpPKs",
            "5CBRYsmfLC49DSnavggu7wd94ukjsADzv2Vd5FYutCguTuqa",
        ),
        (
            "5Escp7EXeZj9E9pKDhQDYTkFc7RGxJhNKXFS6noeWJmNWouC",
            "5Gs2QGQAXYvRyNUmgViDJFdaYyxhWfRxrF6ymHEXsfBygwDD",
        ),
        (
            "5FHqXrqQ9fVRHVspNuEM2qELssszUntppAo4sn12D3UW9dYY",
            "5ESGmfG7NHsbcF7JaXFe9kns1LxqsmXEQUaYwoMb65o4WQXE",
        ),
    ];

    let initial_authorities = VALIDATORS_ADDRESS
        .iter()
        .map(|(sr25519, ed25519)| {
            ValidatorData::from_ids(sr25519, sr25519, ed25519, VALIDATOR_BALANCE, VALIDATOR_BOND)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let nominators: &[NominatorData] = &[];

    let sudo_account = FundedAccount::from_id(
        "5HbqGciWqzKymMVFfDYjgeM3irRHuzV49oeqqSbMYBguc67Q",
        SUDO_BALANCE,
    )?;

    let administrative_account = [
        FundedAccount::from_id(
            "5F6U2FsQQndhFnszdmA5ibHGiHatQ9MRxy3PtxK8arSkXJJf",
            DEV_WALLET_BALANCE,
        )?,
        // Dev Wallet 2 HL
        FundedAccount::from_id(
            "5DnhJzvueoHbpYe5LLaLKeEZBEjHMs6wba7JCgd8iTDLpc1w",
            DEV_WALLET_BALANCE,
        )?,
    ];

    let balances = initial_authorities
        .iter()
        .map(|a| &a.account)
        .chain(nominators.iter().map(|n| &n.account))
        .chain(institutional.iter())
        .chain(administrative_account.iter())
        .chain(core::iter::once(&sudo_account))
        .map(FundedAccount::json_data)
        .collect::<Vec<_>>();
    let staker = initial_authorities
        .iter()
        .cloned()
        .map(|v| Box::new(v) as Box<dyn StakerData>)
        .chain(
            nominators
                .iter()
                .cloned()
                .map(|v| Box::new(v) as Box<dyn StakerData>),
        )
        .collect();

    Ok(genesis(
        // Initial PoA authorities
        initial_authorities
            .iter()
            .map(ValidatorData::ids)
            .collect::<Vec<_>>(),
        // Sudo account [nh-sudo-t1]
        sudo_account.account_id.clone(),
        // Initial balances
        balances,
        staker,
        // min validator count
        5,
        // ideal validator count
        15,
        // max validator count
        Some(200),
        // min validator bond
        10 * THOUSANDS,
        // min nominator bond
        10 * VFY,
        main_parachains_host_configuration(),
    ))
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
        main_parachains_host_configuration(),
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
        main_parachains_host_configuration(),
    )
}

pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from("development"),
        PresetId::from("local"),
        PresetId::from("staging"),
    ]
}

pub fn get_preset(id: &sp_genesis_builder::PresetId) -> Option<Vec<u8>> {
    let cfg = match id.as_ref() {
        "development" => development_config_genesis(),
        "local" => local_config_genesis(),
        "staging" => staging_config_genesis().unwrap(),
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
