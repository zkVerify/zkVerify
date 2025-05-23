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
use sp_core::crypto::Ss58Codec;
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
struct FundedAccount {
    /// The account-id
    account_id: AccountId,
    /// Initial balance
    balance: Balance,
}

impl FundedAccount {
    fn from_id(sr25519_key: &str, balance: Balance) -> Result<Self, sp_core::crypto::PublicError> {
        Ok(Self {
            account_id: from_ss58check(sr25519_key)?,
            balance,
        })
    }

    fn json_data(&self) -> (AccountId, Balance) {
        (self.account_id.clone(), self.balance)
    }
}

#[derive(Clone)]
struct ValidatorData {
    /// The account-id sr25519 public key
    account: FundedAccount,
    ///  Bonded data
    bonded: Balance,

    babe_id: BabeId,
    grandpa_id: GrandpaId,
    validator_id: ValidatorId,
    assignment_id: AssignmentId,
    authority_discovery_id: AuthorityDiscoveryId,
}

impl StakerData for ValidatorData {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        (
            self.account.account_id.clone(),
            self.account.account_id.clone(),
            self.bonded,
            StakerStatus::Validator,
        )
    }
}

impl ValidatorData {
    fn from_ids(
        sr25519_account_key: &str,
        sr25519_common_key: &str,
        ed25519_key: &str,
        balance: Balance,
        bonded: Balance,
    ) -> Result<Self, sp_core::crypto::PublicError> {
        let (account_id, babe_id, grandpa_id, validator_id, assignment_id, authority_discovery_id) =
            authority_ids_from_ss58(sr25519_account_key, sr25519_common_key, ed25519_key)?;
        Ok(Self {
            account: FundedAccount {
                account_id,
                balance,
            },
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
struct NominatorData {
    /// The account-id sr25519 public key
    account: FundedAccount,
    bonded: Balance,
    voted: Vec<AccountId>,
}

impl StakerData for NominatorData {
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
    let cfg = match id.as_ref() {
        "development" => zkv_development_config_genesis(),
        "local" => zkv_local_config_genesis(),
        "testnet" => zkv_testnet_config_genesis().unwrap(),
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

    #[cfg(feature = "runtime-benchmarks")]
    #[test]
    fn development_genesis_config_unchanged() {
        // This test checks that the genesis config that will be used for benchmarks is as
        // expected, and that no change goes unnoticed, as it may break benchmarks.
        // If changes are necessary and tested not to break any benchmark, then please update to
        // golden reference at "tests/genesis_dev_golden.json".
        // "zkv-relay build-spec --chain dev | jq -rc '.genesis.runtimeGenesis.patch' > genesis_dev_golden.json"
        let genesis = super::zkv_development_config_genesis();
        let file = std::fs::File::open("tests/genesis_dev_golden.json")
            .expect("could not open golden file");
        let genesis_golden: serde_json::Value =
            serde_json::from_reader(file).expect("could not parse golden genesis patch");
        assert_eq!(genesis, genesis_golden);
    }
}
