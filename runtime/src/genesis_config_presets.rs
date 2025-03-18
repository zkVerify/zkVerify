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
use polkadot_primitives::{AssignmentId, AsyncBackingParams, SchedulerParams, ValidatorId};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_genesis_builder::PresetId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_staking::StakerStatus;
#[cfg(not(feature = "std"))]
use sp_std::alloc::format;
use sp_std::vec;
use sp_std::vec::Vec;

const ENDOWMENT: Balance = 1_000_000 * VFY;
const STASH_BOND: Balance = ENDOWMENT / 100;
const DEFAULT_ENDOWED_SEEDS: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];
const LOCAL_N_AUTH: usize = 2;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

type Ids = (
    AccountId,
    BabeId,
    GrandpaId,
    ValidatorId,
    AssignmentId,
    AuthorityDiscoveryId,
);

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn from_ss58check<T: sp_core::crypto::Ss58Codec>(
    key: &str,
) -> Result<T, sp_core::crypto::PublicError> {
    <T as sp_core::crypto::Ss58Codec>::from_ss58check(key)
}

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

/// Generate a session authority key.
pub fn authority_keys_from_seed(s: &str) -> Ids {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<ValidatorId>(s),
        get_from_seed::<AssignmentId>(s),
        get_from_seed::<AuthorityDiscoveryId>(s),
    )
}

// Generate authority IDs from SS58 addresses.
pub fn authority_ids_from_ss58(
    sr25519_account_key: &str,
    sr25519_session_key: &str,
    ed25519_session_key: &str,
) -> Result<Ids, sp_core::crypto::PublicError> {
    Ok((
        from_ss58check(sr25519_account_key)?,
        from_ss58check(sr25519_session_key)?,
        from_ss58check(ed25519_session_key)?,
        from_ss58check(sr25519_session_key)?,
        from_ss58check(sr25519_session_key)?,
        from_ss58check(sr25519_session_key)?,
    ))
}

trait StakerData {
    fn staker_data(
        &self,
    ) -> (
        AccountId,
        AccountId,
        Balance,
        sp_staking::StakerStatus<AccountId>,
    );
}

impl StakerData for Box<dyn StakerData> {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        self.as_ref().staker_data()
    }
}

impl StakerData for (AccountId, Balance) {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        (
            self.0.clone(),
            self.0.clone(),
            self.1,
            StakerStatus::Validator,
        )
    }
}

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn genesis(
    initial_authorities: Vec<Ids>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, Balance)>,
    stakers: Vec<Box<dyn StakerData>>,
    min_validator_count: u32,
    max_validator_count: Option<u32>,
    min_validator_bond: Balance,
    min_nominator_bond: Balance,
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
            "validatorCount": 10,
            "stakers": stakers.iter()
                .map(StakerData::staker_data)
                .collect::<Vec<_>>(),
        },
        "sudo": {
            // Assign network admin rights.
            "key": Some(root_key),
        },
        "configuration": {
            "config": default_parachains_host_configuration(),
        },

    })
}

fn default_parachains_host_configuration(
) -> polkadot_runtime_parachains::configuration::HostConfiguration<polkadot_primitives::BlockNumber>
{
    use polkadot_primitives::{MAX_CODE_SIZE, MAX_POV_SIZE};

    polkadot_runtime_parachains::configuration::HostConfiguration {
        validation_upgrade_cooldown: 2u32,
        validation_upgrade_delay: 2,
        code_retention_period: 1200,
        max_code_size: MAX_CODE_SIZE,
        max_pov_size: MAX_POV_SIZE,
        max_head_data_size: 32 * 1024,
        max_upward_queue_count: 8,
        max_upward_queue_size: 1024 * 1024,
        max_downward_message_size: 1024 * 1024,
        max_upward_message_size: 50 * 1024,
        max_upward_message_num_per_candidate: 5,
        hrmp_sender_deposit: 0,
        hrmp_recipient_deposit: 0,
        hrmp_channel_max_capacity: 8,
        hrmp_channel_max_total_size: 8 * 1024,
        hrmp_max_parachain_inbound_channels: 4,
        hrmp_channel_max_message_size: 1024 * 1024,
        hrmp_max_parachain_outbound_channels: 4,
        hrmp_max_message_num_per_candidate: 5,
        dispute_period: 6,
        no_show_slots: 2,
        n_delay_tranches: 25,
        needed_approvals: 2,
        relay_vrf_modulo_samples: 2,
        zeroth_delay_tranche_width: 0,
        minimum_validation_upgrade_delay: 5,
        async_backing_params: AsyncBackingParams {
            max_candidate_depth: 3,
            allowed_ancestry_len: 2,
        },
        scheduler_params: SchedulerParams {
            lookahead: 2,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[derive(Clone)]
struct FundedAccount<'a> {
    /// The account-id sr25519 public key
    _account: &'a str,
    /// The account-id
    account_id: AccountId,
    /// Initial balance
    balance: Balance,
}

impl<'a> FundedAccount<'a> {
    fn from_id(
        sr25519_key: &'a str,
        balance: Balance,
    ) -> Result<Self, sp_core::crypto::PublicError> {
        Ok(Self {
            _account: sr25519_key,
            account_id: from_ss58check(sr25519_key)?,
            balance,
        })
    }

    fn json_data(&self) -> (AccountId, Balance) {
        (self.account_id.clone(), self.balance)
    }
}

#[derive(Clone)]
struct ValidatorData<'a> {
    /// The account-id sr25519 public key
    account: FundedAccount<'a>,
    /// The common sr25519 public key (used for others key instead of grandpa and account)
    _sr: &'a str,
    /// The ed25519 public key (used for grandpa)
    _ed: &'a str,
    ///  Bonded data
    bonded: Balance,

    babe_id: BabeId,
    grandpa_id: GrandpaId,
    validator_id: ValidatorId,
    assignment_id: AssignmentId,
    authority_discovery_id: AuthorityDiscoveryId,
}

impl StakerData for ValidatorData<'_> {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        (
            self.account.account_id.clone(),
            self.account.account_id.clone(),
            self.bonded,
            StakerStatus::Validator,
        )
    }
}

impl<'a> ValidatorData<'a> {
    fn from_ids(
        sr25519_account_key: &'a str,
        sr25519_common_key: &'a str,
        ed25519_key: &'a str,
        balance: Balance,
        bonded: Balance,
    ) -> Result<Self, sp_core::crypto::PublicError> {
        let (account_id, babe_id, grandpa_id, validator_id, assignment_id, authority_discovery_id) =
            authority_ids_from_ss58(sr25519_account_key, sr25519_common_key, ed25519_key)?;
        Ok(Self {
            account: FundedAccount {
                _account: sr25519_account_key,
                account_id,
                balance,
            },
            _sr: sr25519_common_key,
            _ed: ed25519_key,
            bonded,
            babe_id,
            grandpa_id,
            validator_id,
            assignment_id,
            authority_discovery_id,
        })
    }

    fn ids(&self) -> Ids {
        (
            self.account.account_id.clone(),
            self.babe_id.clone(),
            self.grandpa_id.clone(),
            self.validator_id.clone(),
            self.assignment_id.clone(),
            self.authority_discovery_id.clone(),
        )
    }
}

#[derive(Clone)]
struct NominatorData<'a> {
    /// The account-id sr25519 public key
    account: FundedAccount<'a>,
    bonded: Balance,
    voted: Vec<AccountId>,
}

impl StakerData for NominatorData<'_> {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        (
            self.account.account_id.clone(),
            self.account.account_id.clone(),
            self.bonded,
            StakerStatus::Nominator(self.voted.clone()),
        )
    }
}

#[allow(unused)]
pub fn zkv_testnet_config_genesis() -> Result<serde_json::Value, sp_core::crypto::PublicError> {
    const COMMUNITY_CUSTODIAL_BASE_BALANCE: Balance = 100 * MILLIONS;
    const COMMUNITY_CUSTODIAL_BUFFER_BALANCE: Balance = 50 * MILLIONS;
    const FOUNDATION_BASE_BALANCE: Balance = (313 * MILLIONS + 750 * THOUSANDS) / 2;
    const CONTRIBUTOR_BALANCE: Balance = (196 * MILLIONS + 250 * THOUSANDS) / 2;
    const INVESTOR_BALANCE: Balance = (140 * MILLIONS) / 2;
    const VALIDATOR_BALANCE: Balance = 100 * THOUSANDS;
    const VALIDATOR_BOND: Balance = VALIDATOR_BALANCE;
    const NOMINATOR_BALANCE: Balance = 10 * MILLIONS;
    const NOMINATOR_BOND: Balance = NOMINATOR_BALANCE;
    const TREASURY_BALANCE: Balance = MILLIONS;
    const SUDO_BALANCE: Balance = 50 * VFY;

    // From "modlzk/trsry" padded right with zeros.
    let treasury_account = FundedAccount::from_id(
        "5EYCAe5kjJEU9CJ4QMep83WeQNvGwkWpknkU7r3Q3w7n13iV",
        TREASURY_BALANCE,
    )?;

    let community_custodians = [
        FundedAccount::from_id(
            "5DfhBXU2nzQeGtx7fNkRdRC3vajrfLPvVGsDKQ6jbHiiUJqa",
            COMMUNITY_CUSTODIAL_BASE_BALANCE,
        )?,
        FundedAccount::from_id(
            "5FpQiJJXZFGGGUBiX6xS81igb6qQANrPnX9FKPaGuBqb3Zay",
            COMMUNITY_CUSTODIAL_BASE_BALANCE,
        )?,
        // Used to found the old owners, treasury and claim pallets
        FundedAccount::from_id(
            "5CPfe6NMYzy4HH4sYvjRnubCu5yfhY8AXhaYkoM5CeuYFT5c",
            COMMUNITY_CUSTODIAL_BASE_BALANCE + COMMUNITY_CUSTODIAL_BUFFER_BALANCE
                - (TREASURY_BALANCE + EXISTENTIAL_DEPOSIT),
        )?,
    ];

    let contributors_custody = [
        FundedAccount::from_id(
            "5Fhb9daJKNaASmVdsM4pmsLDtJv5zS8W7Kemh4iyFWgtnYgt",
            CONTRIBUTOR_BALANCE,
        )?,
        FundedAccount::from_id(
            "5CaLGMj5FkpnoBfTkXBT4gHXZLzdmqPeTWE1tyuYZB3EcyEE",
            CONTRIBUTOR_BALANCE,
        )?,
    ];

    let investors_custody = [
        FundedAccount::from_id(
            "5FZboFp74WMi3ogERtYbmLwiZmbhTbxhfApAmf4jSgdw98kR",
            INVESTOR_BALANCE,
        )?,
        FundedAccount::from_id(
            "5FTyReguLB6VN2iWzv9ZqHfM8ZtsE9CPiXyWS89A3xaheEF7",
            INVESTOR_BALANCE,
        )?,
    ];

    let initial_authorities = [
        ValidatorData::from_ids(
            "5Hb48vxYpQZQHRbzLMak1AKhtuww6wNK1oMFyigysK8zvHyW",
            "5Hb48vxYpQZQHRbzLMak1AKhtuww6wNK1oMFyigysK8zvHyW",
            "5CpBzkEqzW5RPjDqMMqJuTSkaNrdbo3WxYC8arVjysd96DCN",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5G6win2P1ty9X6DYuvfSFkijHGVx8yceVp1uNFntENyu7H4j",
            "5G6win2P1ty9X6DYuvfSFkijHGVx8yceVp1uNFntENyu7H4j",
            "5CkhLKX285NbSy4FioEBTwF7Q5X69wmr2d8z58tWr1V8sZK4",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5DXDWSgAioftJk5CBhnE1WM7hJVAiCsXALSLxzcV5ouKLo5p",
            "5DXDWSgAioftJk5CBhnE1WM7hJVAiCsXALSLxzcV5ouKLo5p",
            "5E6s3Ho4svXABHokXUCcVpZ6UraRR7JpA6VBmYHDau1ZLep1",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5C5PKzsRhThtFNbUTeKAaaubFK862Pg75AQM4GavWfghvVK5",
            "5C5PKzsRhThtFNbUTeKAaaubFK862Pg75AQM4GavWfghvVK5",
            "5HHKm8QwxoogA1crujuDLQhjxAcGX6VCACqhjYTffBW2nENj",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5H6TGvsWATCc6YzJD4iNdeLE54RMvDcy5BgbqamNPJNNbr5L",
            "5H6TGvsWATCc6YzJD4iNdeLE54RMvDcy5BgbqamNPJNNbr5L",
            "5G5pgFvQJXiQph63HbtAQey6bNRdzpqbJezt2iAe3xGYtRyt",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5CuCBRRajfFFBwRTJ4EPjyPMgoMDKzRSfBa6ktcw8DVcydwM",
            "5CuCBRRajfFFBwRTJ4EPjyPMgoMDKzRSfBa6ktcw8DVcydwM",
            "5HdvRwJirHLhC98fkyp22EK5XXfD6dxGe9o45UT49JEES712",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
        ValidatorData::from_ids(
            "5FgWmvTtUMBTKaMa53MgJskPXUwakJwyx61VGxEjP8YPEyQv",
            "5FgWmvTtUMBTKaMa53MgJskPXUwakJwyx61VGxEjP8YPEyQv",
            "5GcR9PeGmeFzKpkbVEbP95U4WWSK4z6CWGC9pN6tQpTxbVGS",
            VALIDATOR_BALANCE,
            VALIDATOR_BOND,
        )?,
    ];

    let nominators: &[NominatorData] = &[];

    let foundation_custody = [
        FundedAccount::from_id(
            "5DHe8Sm15RHkdWztyyStTqyjMfo7mdkWaHNaUWn6vqtx9a9j",
            FOUNDATION_BASE_BALANCE,
        )?,
        FundedAccount::from_id(
            "5D57RN5zN3KdEaWgFQMuriAPSH6KbZjwCjRYaAELyGexxK9a",
            FOUNDATION_BASE_BALANCE
                - (VALIDATOR_BALANCE * initial_authorities.len() as Balance
                    + NOMINATOR_BALANCE * nominators.len() as Balance
                    + SUDO_BALANCE),
        )?,
    ];

    let sudo_account = FundedAccount::from_id(
        "5DaF8tEbzij76QpxHK9jnjoVDAFRuwcwCisswN5iAyss5CqR",
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
        .chain(sp_std::iter::once(&treasury_account))
        .chain(sp_std::iter::once(&sudo_account))
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
        // max validator count
        Some(200),
        // min validator bond
        10 * THOUSANDS,
        // min nominator bond
        10 * VFY,
    ))
}

pub fn zkv_local_config_genesis() -> serde_json::Value {
    let balances = DEFAULT_ENDOWED_SEEDS
        .into_iter()
        .map(|seed| (get_account_id_from_seed::<sr25519::Public>(seed), ENDOWMENT))
        .collect::<Vec<_>>();

    genesis(
        // Initial PoA authorities
        DEFAULT_ENDOWED_SEEDS
            .into_iter()
            .map(|seed| authority_keys_from_seed(seed))
            .take(LOCAL_N_AUTH)
            .collect::<Vec<_>>(),
        // Sudo account
        get_account_id_from_seed::<sr25519::Public>(DEFAULT_ENDOWED_SEEDS[0]),
        // Pre-funded accounts
        balances.clone(),
        balances
            .into_iter()
            .map(|(a, _)| Box::new((a, STASH_BOND)) as Box<dyn StakerData>)
            .collect(),
        // min validator count
        1,
        // max validator count
        None,
        // min validator bond
        0,
        // min nominator bond
        0,
    )
}

pub fn zkv_development_config_genesis() -> serde_json::Value {
    let balances = DEFAULT_ENDOWED_SEEDS
        .into_iter()
        .map(|seed| (get_account_id_from_seed::<sr25519::Public>(seed), ENDOWMENT))
        .take(2)
        .chain([
            // The following is a workaround for pallet_treasury benchmarks which hardcode
            // a payment of 100 (lower than EXISTENTIAL_DEPOSIT) to a given address ([0x0])
            #[cfg(feature = "runtime-benchmarks")]
            (
                from_ss58check("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM").unwrap(),
                ENDOWMENT,
            ),
        ])
        .collect::<Vec<_>>();

    genesis(
        // Initial PoA authorities
        DEFAULT_ENDOWED_SEEDS
            .into_iter()
            .map(|seed| authority_keys_from_seed(seed))
            .take(1)
            .collect::<Vec<_>>(),
        // Sudo account
        get_account_id_from_seed::<sr25519::Public>(DEFAULT_ENDOWED_SEEDS[0]),
        // Pre-funded accounts
        balances.clone(),
        balances
            .into_iter()
            .map(|(a, _)| Box::new((a, STASH_BOND)) as Box<dyn StakerData>)
            .collect(),
        // min validator count
        1,
        // max validator count
        None,
        // min validator bond
        0,
        // min nominator bond
        0,
    )
}

pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from("development"),
        PresetId::from("local"),
        PresetId::from("testnet"),
    ]
}

pub fn get_preset(id: &sp_genesis_builder::PresetId) -> Option<sp_std::vec::Vec<u8>> {
    let cfg = match id.try_into() {
        Ok("development") => zkv_development_config_genesis(),
        Ok("local") => zkv_local_config_genesis(),
        Ok("testnet") => zkv_testnet_config_genesis().unwrap(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&cfg)
            .expect("genesis cfg must be serializable. qed.")
            .into_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    // The following test verifies whether we added session configuration in the genesis block
    // by checking that the json returned by testnet_genesis() contains the field "session"
    #[test]
    fn testnet_genesis_should_set_session_keys() {
        let initial_authorities = vec![authority_keys_from_seed("Alice")];
        let root_key = get_account_id_from_seed::<sr25519::Public>("Alice");

        let ret_val: serde_json::Value = genesis(
            initial_authorities.clone(),
            root_key,
            vec![],
            vec![Box::new((initial_authorities[0].0.clone(), 7 * VFY)) as Box<dyn StakerData>],
            1,
            None,
            0,
            0,
        );

        let session_config = &ret_val["session"];

        // Check that we have the field "session" in the genesis config
        assert!(!session_config.is_null());

        let auth_len = session_config
            .as_object()
            .map(|inner| inner["keys"].as_array().unwrap().len())
            .unwrap();
        let staker = &ret_val["staking"]["stakers"][0];

        // ret_val.clone()["staking"]["stakers"][0];
        // Check that we have one "keys" set
        assert_eq!(1, auth_len);
        assert_eq!(
            Value::Number((7 * VFY).into()),
            staker.as_array().unwrap()[2]
        );
    }
}
