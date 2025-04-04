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

//! Chain specifications for the test runtime.

use babe_primitives::AuthorityId as BabeId;
use grandpa::AuthorityId as GrandpaId;
use pallet_staking::Forcing;
use polkadot_primitives::{AccountId, AssignmentId, ValidatorId, MAX_CODE_SIZE, MAX_POV_SIZE};
use polkadot_service::chain_spec::{get_account_id_from_seed, get_from_seed};
use sc_chain_spec::{ChainSpec, ChainSpecExtension, ChainType};
pub use sc_consensus_grandpa as grandpa;
use sc_sync_state_rpc::LightSyncStateExtension;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
pub use sp_consensus_babe as babe_primitives;
use sp_core::sr25519;
use sp_runtime::Perbill;
use test_runtime::BABE_GENESIS_EPOCH_CONFIG;
use test_runtime_constants::currency::DOTS;

const DEFAULT_PROTOCOL_ID: &str = "dot";

/// The extensions for the [`ChainSpec`].
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
pub struct Extensions {
    light_sync_state: LightSyncStateExtension,
}

/// The `ChainSpec` parameterized for polkadot test runtime.
pub type PolkadotChainSpec = sc_service::GenericChainSpec<Extensions>;

/// Returns the properties for the [`PolkadotChainSpec`].
pub fn polkadot_chain_spec_properties() -> serde_json::map::Map<String, serde_json::Value> {
    serde_json::json!({
        "tokenDecimals": 10,
    })
    .as_object()
    .expect("Map given; qed")
    .clone()
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn polkadot_local_testnet_config() -> PolkadotChainSpec {
    PolkadotChainSpec::builder(
        test_runtime::WASM_BINARY.expect("Wasm binary must be built for testing"),
        Default::default(),
    )
    .with_name("Local Testnet")
    .with_id("local_testnet")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch(polkadot_local_testnet_genesis())
    .with_protocol_id(DEFAULT_PROTOCOL_ID)
    .with_properties(polkadot_chain_spec_properties())
    .build()
}

/// Local testnet genesis config (multivalidator Alice + Bob)
pub fn polkadot_local_testnet_genesis() -> serde_json::Value {
    polkadot_testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
    )
}

/// Helper function to generate stash, controller and session key from seed
fn get_authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    BabeId,
    GrandpaId,
    ValidatorId,
    AssignmentId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<ValidatorId>(seed),
        get_from_seed::<AssignmentId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

fn testnet_accounts() -> Vec<AccountId> {
    vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
        get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
    ]
}

/// Helper function to create polkadot `RuntimeGenesisConfig` for testing
fn polkadot_testnet_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        BabeId,
        GrandpaId,
        ValidatorId,
        AssignmentId,
        AuthorityDiscoveryId,
    )>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
) -> serde_json::Value {
    use test_runtime as runtime;

    let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

    const ENDOWMENT: u128 = 1_000_000 * DOTS;
    const STASH: u128 = 100 * DOTS;

    serde_json::json!({
        "balances": {
            "balances": endowed_accounts.iter().map(|k| (k.clone(), ENDOWMENT)).collect::<Vec<_>>(),
        },
        "session": {
            "keys": initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        runtime::SessionKeys {
                            babe: x.2.clone(),
                            grandpa: x.3.clone(),
                            para_validator: x.4.clone(),
                            para_assignment: x.5.clone(),
                            authority_discovery: x.6.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
        },
        "staking": {
            "minimumValidatorCount": 1,
            "validatorCount": 2,
            "stakers": initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), STASH, runtime::StakerStatus::<AccountId>::Validator))
                .collect::<Vec<_>>(),
            "invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
            "forceEra": Forcing::NotForcing,
            "slashRewardFraction": Perbill::from_percent(10),
        },
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "sudo": { "key": Some(root_key) },
        "configuration": {
            "config": polkadot_runtime_parachains::configuration::HostConfiguration {
                validation_upgrade_cooldown: 10u32,
                validation_upgrade_delay: 5,
                code_retention_period: 1200,
                max_code_size: MAX_CODE_SIZE,
                max_pov_size: MAX_POV_SIZE,
                max_head_data_size: 32 * 1024,
                //group_rotation_frequency: 20,
                //paras_availability_period: 4,
                no_show_slots: 10,
                minimum_validation_upgrade_delay: 5,
                ..Default::default()
            },
        }
    })
}

/// Can be called for a `Configuration` to check if it is a configuration for the `Test` network.
pub trait IdentifyVariant {
    /// Returns if this is a configuration for the `Test` network.
    fn is_test(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
    fn is_test(&self) -> bool {
        self.id().starts_with("test")
    }
}
