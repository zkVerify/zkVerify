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

fn parachains_host_configuration() -> HostConfiguration<BlockNumber> {
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
        n_delay_tranches: 2,
        needed_approvals: 5,
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
    const COMMUNITY_CUSTODIAL_BASE_BALANCE: Balance = 100 * MILLIONS;
    const COMMUNITY_CUSTODIAL_BUFFER_BALANCE: Balance = 50 * MILLIONS;
    const FOUNDATION_BASE_BALANCE: Balance = (313 * MILLIONS + 750 * THOUSANDS) / 2;
    const CONTRIBUTOR_BALANCE: Balance = (196 * MILLIONS + 250 * THOUSANDS) / 2;
    const INVESTOR_BALANCE: Balance = (140 * MILLIONS) / 2;
    const VALIDATOR_BALANCE: Balance = 100 * THOUSANDS;
    const VALIDATOR_BOND: Balance = VALIDATOR_BALANCE;
    const NOMINATOR_BALANCE: Balance = 10 * MILLIONS;
    // const NOMINATOR_BOND: Balance = NOMINATOR_BALANCE;
    const TREASURY_BALANCE: Balance = MILLIONS;
    const SUDO_BALANCE: Balance = 50 * VFY;

    // From "modlzk/trsry" padded right with zeros.
    let treasury_account = FundedAccount::from_id(
        "5EYCAe5kjJEU9CJ4QMep83WeQNvGwkWpknkU7r3Q3w7n13iV",
        TREASURY_BALANCE,
    )?;

    let community_custodians = [
        FundedAccount::from_id(
            "5CBFC7PE6rq2UmUvHwXfejTctAP6KHUDdLxS65PDcFWHkoXP",
            COMMUNITY_CUSTODIAL_BASE_BALANCE,
        )?,
        FundedAccount::from_id(
            "5DnrLV6bBntEa65b7NM9RBPJv3jBVgzSFqFG1eUuNUGAze4h",
            COMMUNITY_CUSTODIAL_BASE_BALANCE,
        )?,
        // Used to found the old owners, treasury and claim pallets
        FundedAccount::from_id(
            "5EFBAFR49Y2XZ1rBwQXM4Wo2rdenNpi5xyiDEv3oxJmSa2Ut",
            COMMUNITY_CUSTODIAL_BASE_BALANCE + COMMUNITY_CUSTODIAL_BUFFER_BALANCE
                - (TREASURY_BALANCE + EXISTENTIAL_DEPOSIT),
        )?,
    ];

    let contributors_custody = [
        FundedAccount::from_id(
            "5DF9kBMrPbqE3kSjVt4QDHBYZYze8WWmBssrhriXhut4DrKP",
            CONTRIBUTOR_BALANCE,
        )?,
        FundedAccount::from_id(
            "5CtatxhAjwJnsW9kjsuUpuMZazA2EZ9yb85i7xyPWETyy8Wt",
            CONTRIBUTOR_BALANCE,
        )?,
    ];

    let investors_custody = [
        FundedAccount::from_id(
            "5DSVahZ7bhzTyoxkEjQeNJ7yU6M6tidUe89cC2PTSW9FYVaP",
            INVESTOR_BALANCE,
        )?,
        FundedAccount::from_id(
            "5FHz3zPfdJbDhPesCrVB1vb7EwT6jHWGTYSoQrLpAeWKJXP7",
            INVESTOR_BALANCE,
        )?,
    ];

    let initial_authorities = [
        ValidatorData::from_ids(
            "5DcJTvFvLUh9qHxdjZmBtXzzG7ttNFSmDtBwohc7ws89gHuW",
            "5DcJTvFvLUh9qHxdjZmBtXzzG7ttNFSmDtBwohc7ws89gHuW",
            "5G3Sqt3NqQUDYP5PXzZtiwEaWkVUQ6w6bLd58A729yQdaJ5U",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5CwWnDWg5wLAhyV66Synjjtt2qBDic5F2q1314eivUh8KVXv",
            "5CwWnDWg5wLAhyV66Synjjtt2qBDic5F2q1314eivUh8KVXv",
            "5GShazf2w8XURufZJFhtGR1JBUsNhX9LRLjyD7G5Vizem74m",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5EvNz6NKE6MTSFVZPX7S4tqPExrmgqbELUxSp8ycCGb6UKAH",
            "5EvNz6NKE6MTSFVZPX7S4tqPExrmgqbELUxSp8ycCGb6UKAH",
            "5FD8ihBkU7fbMfhKqREfd5htRiqNGpQ8Gy1rUgmoyGuzU1z6",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5GdnMHVWPJK4rjnDXNd6oLQFGytcUGgrq1zgsWYg7hVZLL2q",
            "5GdnMHVWPJK4rjnDXNd6oLQFGytcUGgrq1zgsWYg7hVZLL2q",
            "5Cnu6KyiC3M1FoZx4R39GGxaVbDqDpnW66Vmaqxn9U5hJnHN",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5DM8WWWYvPUJShXV4hhGXWCYjHRzCKKs3mHq4mfiaLJFxetC",
            "5DM8WWWYvPUJShXV4hhGXWCYjHRzCKKs3mHq4mfiaLJFxetC",
            "5GHthqbUX4TGAJa72Jdu5BsvU5cRw2EhuaF4PiyFfEoYzb1M",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5HdVFRoy8AGN58rkR3dbd1MHaUSx4uuhdEUrJFzr8FGBJZMB",
            "5HdVFRoy8AGN58rkR3dbd1MHaUSx4uuhdEUrJFzr8FGBJZMB",
            "5EPaF1sa2Kt83yz1g8b3LCAw1DrvxE3Hspm9TLdud2ojhdaT",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5GGygViDPJXaPFRsABSbsuxZvueWLfZGgAin8YoKuLCYF5MZ",
            "5GGygViDPJXaPFRsABSbsuxZvueWLfZGgAin8YoKuLCYF5MZ",
            "5HSHuhE2WebrwXZ85Gs4n7vdWzU6PsnK9F3KGoxVeupxoH9X",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
    ];

    let nominators: &[NominatorData] = &[];

    let foundation_custody = [
        FundedAccount::from_id(
            "5HianRRmyp6HgfTFvbmobPX5YrvzyWRvjeqcWmMMBweEaa6m",
            FOUNDATION_BASE_BALANCE,
        )?,
        FundedAccount::from_id(
            "5H4Rcaj63MBL3fuYNoMuM6XFY2EvAPKD4PasFyCCZA4WrAH3",
            FOUNDATION_BASE_BALANCE
                - (VALIDATOR_BALANCE * initial_authorities.len() as Balance
                    + NOMINATOR_BALANCE * nominators.len() as Balance
                    + SUDO_BALANCE),
        )?,
    ];

    let sudo_account = FundedAccount::from_id(
        "5GReicw1vfMB4Pq9vZfhPc2epjRkAtM1Zch5c3Z5gxfmzM7k",
        SUDO_BALANCE,
    )?;

    let balances = initial_authorities
        .iter()
        .map(|a| &a.account)
        .chain(nominators.iter().map(|n| &n.account))
        .chain(community_custodians.iter())
        .chain(foundation_custody.iter())
        .chain(contributors_custody.iter())
        .chain(investors_custody.iter())
        .chain(core::iter::once(&treasury_account))
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
        3,
        10,
        // max validator count
        Some(200),
        // min validator bond
        10 * THOUSANDS,
        // min nominator bond
        10 * VFY,
        parachains_host_configuration(),
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
