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

use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use zkv_runtime::currency::{Balance, ACME};
use zkv_runtime::{currency, AccountId, SessionKeysBase, Signature, WASM_BINARY};

// The connection strings for bootnodes
const BOOTNODE_1_DNS: &str = "bootnode-tn-1.zkverify.io";
const BOOTNODE_1_PEER_ID: &str = "12D3KooWNhvf6iSowraUY4tZnjpNZXEe85oy9zDWYRKFBnWivukc";
const BOOTNODE_2_DNS: &str = "bootnode-tn-2.zkverify.io";
const BOOTNODE_2_PEER_ID: &str = "12D3KooWEjVadU1YWyfDGvyRXPbCq2rXhzJtXaG4RxJZBkGE9Aug";

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://testnet-telemetry.zkverify.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
type Extensions = Option<()>;
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

fn session_keys(babe: BabeId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeysBase {
    SessionKeysBase {
        babe,
        grandpa,
        im_online,
    }
}

/// Generate a session authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, BabeId, GrandpaId, ImOnlineId) {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<ImOnlineId>(s),
    )
}

// Generate authority IDs from SS58 addresses.
pub fn authority_ids_from_ss58(
    sr25519_key: &str,
    ed25519_key: &str,
) -> Result<(AccountId, BabeId, GrandpaId, ImOnlineId), String> {
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
                "An error occurred while converting SS58 to ImOnlineId: {}",
                error
            )
        })?,
    ))
}

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        None,
    )
    .with_name("Development")
    .with_id("dev")
    .with_chain_type(ChainType::Development)
    .with_properties({
        let mut props = Properties::new();
        props.insert("tokenSymbol".into(), "ACME".into());
        props.insert("tokenDecimals".into(), 18.into());
        props.insert("ss58Format".into(), zkv_runtime::SS58Prefix::get().into());
        props
    })
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
        None,
    )
    .with_name("ZKV Local")
    .with_id("zkv_local")
    .with_protocol_id("lacme")
    .with_chain_type(ChainType::Local)
    .with_properties({
        let mut props = Properties::new();
        props.insert("tokenSymbol".into(), "ACME".into());
        props.insert("tokenDecimals".into(), 18.into());
        props.insert("ss58Format".into(), zkv_runtime::SS58Prefix::get().into());
        props
    })
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

pub fn testnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../chain-specs/zkverify_testnet.json")[..])
}

/// To be used when building new testnet chain-spec
#[allow(dead_code)]
pub fn testnet_config_build() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        None,
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
            sc_telemetry::CONSENSUS_INFO,
        )])
        .expect("Horizen Labs telemetry url is valid; qed"),
    )
    .with_properties({
        let mut props = Properties::new();
        props.insert("tokenSymbol".into(), "ACME".into());
        props.insert("tokenDecimals".into(), 18.into());
        props.insert("ss58Format".into(), zkv_runtime::SS58Prefix::get().into());
        props
    })
    .with_genesis_config_patch(genesis(
        // Initial PoA authorities
        vec![
            // zkv-validator-t1
            (
                authority_ids_from_ss58(
                    "5ETuZEyLnfVzQCaDM8aQCcsNnz6xjPKvQCtqynCLqwng8QLd",
                    "5CbPYnSSw7KpKAJR3caCYv7KP3ChaAUgw5BgC3GPppnfiK5E",
                )?,
                280 * currency::MILLIONS,
            ),
            // zkv-validator-t2
            (
                authority_ids_from_ss58(
                    "5D29UEzgStCBTnjKNdkurDNvd3FHePHgTkPEUvjXYvg3brJj",
                    "5H5XnaSsh5eebN2BSTx19qCUweMziyJATuUVb9qdWVDhHQ3K",
                )?,
                280 * currency::MILLIONS,
            ),
            // zkv-validator-t3
            (
                authority_ids_from_ss58(
                    "5DiMVAp8WmFyWAwaTwAr7sU4K3brXcgNCBDbHoBWj3M46PiP",
                    "5CYcXe9bodJ31HE6pLT9EyUK5mrB3otXDNhDXVFSSmTAYm4f",
                )?,
                140 * currency::MILLIONS,
            ),
        ],
        // Sudo account [zkv-sudo-t1]
        from_ss58check("5D9txxK9DTvgCznTjJo7q1cxAgmWa83CzHvcz8zhBtLgaLBV")
            .map_err(|error| error.to_string())?,
        // Initial balances
        vec![
            // zkv-validator-t1
            (
                from_ss58check("5ETuZEyLnfVzQCaDM8aQCcsNnz6xjPKvQCtqynCLqwng8QLd")
                    .map_err(|error| error.to_string())?,
                280 * currency::MILLIONS + 1000 * currency::ACME,
            ),
            // zkv-validator-t2
            (
                from_ss58check("5D29UEzgStCBTnjKNdkurDNvd3FHePHgTkPEUvjXYvg3brJj")
                    .map_err(|error| error.to_string())?,
                280 * currency::MILLIONS + 1000 * currency::ACME,
            ),
            // zkv-validator-t3
            (
                from_ss58check("5DiMVAp8WmFyWAwaTwAr7sU4K3brXcgNCBDbHoBWj3M46PiP")
                    .map_err(|error| error.to_string())?,
                140 * currency::MILLIONS + 1000 * currency::ACME,
            ),
            // zkv-sudo-t1
            (
                from_ss58check("5EhREncHsntgJaax9YQphk1xN3LxPu2Rzbz4A3g7Ut8cRXWq")
                    .map_err(|error| error.to_string())?,
                7 * currency::MILLIONS,
            ),
            // zkv-wallet-custody-t1
            (
                from_ss58check("5C84NU2477uHCUF1A8rHb89sP2D2ZsnquPaGa2Htv75FN9gm")
                    .map_err(|error| error.to_string())?,
                70 * currency::MILLIONS,
            ),
            // zkv-wallet-custody-t2
            (
                from_ss58check("5HdZjrmNAkWQhYQUPNv7YRYnT4vyQswjbNm8eXBvULNQz5wH")
                    .map_err(|error| error.to_string())?,
                70 * currency::MILLIONS,
            ),
            // zkv-wallet-automated-t1
            (
                from_ss58check("5HjFLKpiCStQgRm6ZM1fT1R5pLKAqQdUG3uh7pvzaQfhdFuB")
                    .map_err(|error| error.to_string())?,
                35 * currency::MILLIONS,
            ),
            // zkv-wallet-user-t1
            (
                from_ss58check("5ECktCamcAtBFJirEzvvJmXFxgLMCTAejhqZwLT1Dxn2fwB1")
                    .map_err(|error| error.to_string())?,
                7 * currency::MILLIONS,
            ),
            // zkv-wallet-faucet-t1
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
    initial_authorities: Vec<((AccountId, BabeId, GrandpaId, ImOnlineId), Balance)>,
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
                .map(|((account, babe, grandpa, imonline), _staking)| { (account.clone(), account, session_keys(babe, grandpa, imonline)) })
                .collect::<Vec<_>>(),
        },
        "staking": {
            "minimumValidatorCount": initial_authorities.len(), // must be 1 for pallet-session benchmarks
            "validatorCount": 3,
            "stakers": initial_authorities.iter()
                .cloned()
                .map(|((account, _babe, _grandpa, _imonline), staking)| (account.clone(), account, staking, sp_staking::StakerStatus::Validator::<AccountId>))
                .collect::<Vec<_>>(),
        },
        "sudo": {
            // Assign network admin rights.
            "key": Some(root_key),
        },
    })
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
