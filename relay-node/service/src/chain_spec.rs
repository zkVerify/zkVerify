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

use polkadot_primitives::vstaging::SchedulerParams;
use polkadot_primitives::{AssignmentId, AsyncBackingParams, ValidatorId};
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use sc_sync_state_rpc::LightSyncStateExtension;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use telemetry::TelemetryEndpoints;
use zkv_runtime::currency::{Balance, ACME};
use zkv_runtime::{currency, AccountId, SessionKeysRelay as SessionKeys, Signature, WASM_BINARY};

/// The extensions for the [`ChainSpec`].
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
pub struct Extensions {
    light_sync_state: LightSyncStateExtension,
}

// The connection strings for bootnodes
const BOOTNODE_1_DNS: &str = "bootnode-tn-1.zkverify.io";
const BOOTNODE_1_PEER_ID: &str = "12D3KooWNhvf6iSowraUY4tZnjpNZXEe85oy9zDWYRKFBnWivukc";
const BOOTNODE_2_DNS: &str = "bootnode-tn-2.zkverify.io";
const BOOTNODE_2_PEER_ID: &str = "12D3KooWEjVadU1YWyfDGvyRXPbCq2rXhzJtXaG4RxJZBkGE9Aug";

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://testnet-telemetry.zkverify.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

const ENDOWMENT: Balance = 1_000_000 * ACME;
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
pub fn authority_keys_from_seed(
    s: &str,
) -> (
    AccountId,
    BabeId,
    GrandpaId,
    ValidatorId,
    AssignmentId,
    AuthorityDiscoveryId,
) {
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
    sr25519_key: &str,
    ed25519_key: &str,
) -> Result<
    (
        AccountId,
        BabeId,
        GrandpaId,
        ValidatorId,
        AssignmentId,
        AuthorityDiscoveryId,
    ),
    String,
> {
    Ok((
        from_ss58check(sr25519_key).map_err(|error| {
            format!(
                "An error occurred while converting SS58 to AccountId: {}",
                error
            )
        })?,
        from_ss58check(sr25519_key).map_err(|error| {
            format!(
                "An error occurred while converting SS58 to BabeId: {}",
                error
            )
        })?,
        from_ss58check(ed25519_key).map_err(|error| {
            format!(
                "An error occurred while converting SS58 to GrandpaId: {}",
                error
            )
        })?,
        from_ss58check(sr25519_key).map_err(|error| {
            format!(
                "An error occurred while converting SS58 to ValidatorId: {}",
                error
            )
        })?,
        from_ss58check(sr25519_key).map_err(|error| {
            format!(
                "An error occurred while converting SS58 to AssignmentId: {}",
                error
            )
        })?,
        from_ss58check(sr25519_key).map_err(|error| {
            format!(
                "An error occurred while converting SS58 to AuthorityDiscoveryId: {}",
                error
            )
        })?,
    ))
}

fn chain_properties() -> Properties {
    [
        ("tokenSymbol".to_string(), serde_json::Value::from("ACME")),
        ("tokenDecimals".to_string(), serde_json::Value::from(18_u8)),
    ]
    .into_iter()
    .collect()
}

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("Development")
    .with_id("dev")
    .with_chain_type(ChainType::Development)
    .with_properties(chain_properties())
    .with_genesis_config_patch(genesis(
        // Initial PoA authorities
        DEFAULT_ENDOWED_SEEDS
            .into_iter()
            .map(|seed| (authority_keys_from_seed(seed), STASH_BOND))
            .take(1)
            .collect::<Vec<_>>(),
        // Sudo account
        get_account_id_from_seed::<sr25519::Public>(DEFAULT_ENDOWED_SEEDS[0]),
        // Pre-funded accounts
        DEFAULT_ENDOWED_SEEDS
            .into_iter()
            .map(|seed| (get_account_id_from_seed::<sr25519::Public>(seed), ENDOWMENT))
            .take(2)
            .collect::<Vec<_>>(),
        true,
    ))
    .build())
}

pub fn local_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("NH Local")
    .with_id("nh_local")
    .with_protocol_id("lacme")
    .with_chain_type(ChainType::Local)
    .with_properties(chain_properties())
    .with_genesis_config_patch(genesis(
        // Initial PoA authorities
        DEFAULT_ENDOWED_SEEDS
            .into_iter()
            .map(|seed| (authority_keys_from_seed(seed), STASH_BOND))
            .take(LOCAL_N_AUTH)
            .collect::<Vec<_>>(),
        // Sudo account
        get_account_id_from_seed::<sr25519::Public>(DEFAULT_ENDOWED_SEEDS[0]),
        // Pre-funded accounts
        DEFAULT_ENDOWED_SEEDS
            .into_iter()
            .map(|seed| (get_account_id_from_seed::<sr25519::Public>(seed), ENDOWMENT))
            .collect::<Vec<_>>(),
        true,
    ))
    .build())
}

/// To be used when building new testnet chain-spec
pub fn testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("ZKV Testnet")
    .with_id("zkv_testnet")
    .with_protocol_id("tacme")
    .with_chain_type(ChainType::Live)
    .with_boot_nodes(vec![
        format!("/dns/{BOOTNODE_1_DNS}/tcp/30333/p2p/{BOOTNODE_1_PEER_ID}")
            .parse()
            .expect("MultiaddrWithPeerId"),
        format!("/dns/{BOOTNODE_1_DNS}/tcp/30334/ws/p2p/{BOOTNODE_1_PEER_ID}")
            .parse()
            .expect("MultiaddrWithPeerId"),
        format!("/dns/{BOOTNODE_1_DNS}/tcp/443/wss/p2p/{BOOTNODE_1_PEER_ID}")
            .parse()
            .expect("MultiaddrWithPeerId"),
        format!("/dns/{BOOTNODE_2_DNS}/tcp/30333/p2p/{BOOTNODE_2_PEER_ID}")
            .parse()
            .expect("MultiaddrWithPeerId"),
        format!("/dns/{BOOTNODE_2_DNS}/tcp/30334/ws/p2p/{BOOTNODE_2_PEER_ID}")
            .parse()
            .expect("MultiaddrWithPeerId"),
        format!("/dns/{BOOTNODE_2_DNS}/tcp/443/wss/p2p/{BOOTNODE_2_PEER_ID}")
            .parse()
            .expect("MultiaddrWithPeerId"),
    ])
    .with_telemetry_endpoints(
        TelemetryEndpoints::new(vec![(
            STAGING_TELEMETRY_URL.to_string(),
            telemetry::CONSENSUS_INFO,
        )])
        .expect("Horizen Labs telemetry url is valid; qed"),
    )
    .with_properties(chain_properties())
    .with_genesis_config_patch(genesis(
        // Initial PoA authorities
        vec![
            // nh-validator-t1
            (
                authority_ids_from_ss58(
                    "5ETuZEyLnfVzQCaDM8aQCcsNnz6xjPKvQCtqynCLqwng8QLd",
                    "5CbPYnSSw7KpKAJR3caCYv7KP3ChaAUgw5BgC3GPppnfiK5E",
                )?,
                280 * currency::MILLIONS,
            ),
            // nh-validator-t2
            (
                authority_ids_from_ss58(
                    "5D29UEzgStCBTnjKNdkurDNvd3FHePHgTkPEUvjXYvg3brJj",
                    "5H5XnaSsh5eebN2BSTx19qCUweMziyJATuUVb9qdWVDhHQ3K",
                )?,
                280 * currency::MILLIONS,
            ),
            // nh-validator-t3
            (
                authority_ids_from_ss58(
                    "5DiMVAp8WmFyWAwaTwAr7sU4K3brXcgNCBDbHoBWj3M46PiP",
                    "5CYcXe9bodJ31HE6pLT9EyUK5mrB3otXDNhDXVFSSmTAYm4f",
                )?,
                140 * currency::MILLIONS,
            ),
        ],
        // Sudo account [nh-sudo-t1]
        from_ss58check("5D9txxK9DTvgCznTjJo7q1cxAgmWa83CzHvcz8zhBtLgaLBV")
            .map_err(|error| error.to_string())?,
        // Initial balances
        vec![
            // nh-validator-t1
            (
                from_ss58check("5ETuZEyLnfVzQCaDM8aQCcsNnz6xjPKvQCtqynCLqwng8QLd")
                    .map_err(|error| error.to_string())?,
                280 * currency::MILLIONS + 1000 * currency::ACME,
            ),
            // nh-validator-t2
            (
                from_ss58check("5D29UEzgStCBTnjKNdkurDNvd3FHePHgTkPEUvjXYvg3brJj")
                    .map_err(|error| error.to_string())?,
                280 * currency::MILLIONS + 1000 * currency::ACME,
            ),
            // nh-validator-t3
            (
                from_ss58check("5DiMVAp8WmFyWAwaTwAr7sU4K3brXcgNCBDbHoBWj3M46PiP")
                    .map_err(|error| error.to_string())?,
                140 * currency::MILLIONS + 1000 * currency::ACME,
            ),
            // nh-sudo-t1
            (
                from_ss58check("5EhREncHsntgJaax9YQphk1xN3LxPu2Rzbz4A3g7Ut8cRXWq")
                    .map_err(|error| error.to_string())?,
                7 * currency::MILLIONS,
            ),
            // nh-wallet-custody-t1
            (
                from_ss58check("5C84NU2477uHCUF1A8rHb89sP2D2ZsnquPaGa2Htv75FN9gm")
                    .map_err(|error| error.to_string())?,
                70 * currency::MILLIONS,
            ),
            // nh-wallet-custody-t2
            (
                from_ss58check("5HdZjrmNAkWQhYQUPNv7YRYnT4vyQswjbNm8eXBvULNQz5wH")
                    .map_err(|error| error.to_string())?,
                70 * currency::MILLIONS,
            ),
            // nh-wallet-automated-t1
            (
                from_ss58check("5HjFLKpiCStQgRm6ZM1fT1R5pLKAqQdUG3uh7pvzaQfhdFuB")
                    .map_err(|error| error.to_string())?,
                35 * currency::MILLIONS,
            ),
            // nh-wallet-user-t1
            (
                from_ss58check("5ECktCamcAtBFJirEzvvJmXFxgLMCTAejhqZwLT1Dxn2fwB1")
                    .map_err(|error| error.to_string())?,
                7 * currency::MILLIONS,
            ),
            // nh-wallet-faucet-t1
            (
                from_ss58check("5EZbvFqx3j7ejqBSPWseif8xL3PwoqMQHdMT8rs9qWoHcdR3")
                    .map_err(|error| error.to_string())?,
                70 * currency::MILLIONS,
            ),
        ],
        true,
    ))
    .build())
}

/// Configure initial storage state for FRAME modules.
fn genesis(
    initial_authorities: Vec<(
        (
            AccountId,
            BabeId,
            GrandpaId,
            ValidatorId,
            AssignmentId,
            AuthorityDiscoveryId,
        ),
        Balance,
    )>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, Balance)>,
    _enable_println: bool,
) -> serde_json::Value {
    serde_json::json!({
        "balances": {
            // Configure endowed accounts with initial balance.
            "balances": endowed_accounts,
        },
        "babe": {
            "epochConfig": Some(zkv_runtime::BABE_GENESIS_EPOCH_CONFIG),
        },
        "session": {
            "keys": initial_authorities.iter()
                .cloned()
                .map(|((account, babe, grandpa, para, assign, auth), _staking)| { (account.clone(), account, session_keys(babe, grandpa, para, assign, auth)) })
                .collect::<Vec<_>>(),
        },
        "staking": {
            "minimumValidatorCount": 2,
            "validatorCount": 3,
            "stakers": initial_authorities.iter()
                .cloned()
                .map(|((account, ..), staking)| (account.clone(), account, staking, sp_staking::StakerStatus::Validator::<AccountId>))
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

// This is a sample unit test
// Following Rust convention, unit tests are appended in the same file as the module they are
// testing. This is acceptable and should not create confusion, as long as the tests have a
// very narrow scope - i.e. for verifying the behavior of a single function of a module.
#[cfg(test)]
mod tests {
    use super::*;

    // The following test verifies whether we added session configuration in the genesis block
    // by checking that the json returned by testnet_genesis() contains the field "session"
    #[test]
    fn testnet_genesis_should_set_session_keys() {
        let initial_authorities = vec![(authority_keys_from_seed("Alice"), 7 * currency::ACME)];
        let root_key = get_account_id_from_seed::<sr25519::Public>("Alice");

        let ret_val: serde_json::Value = genesis(initial_authorities, root_key, vec![], false);

        let session_config = &ret_val["session"];

        // Check that we have the field "session" in the genesis config
        assert!(!session_config.is_null());

        let auth_len = session_config
            .as_object()
            .map(|inner| inner["keys"].as_array().unwrap().len())
            .unwrap();
        // Check that we have one "keys" set
        assert_eq!(1, auth_len);
    }

    // This test checks whether the local testnet genesis configuration is generated correctly
    #[test]
    fn local_testnet_genesis_should_be_valid() {
        assert!(testnet_config().is_ok());
    }
}
