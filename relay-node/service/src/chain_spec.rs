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

use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use sc_sync_state_rpc::LightSyncStateExtension;
use serde::{Deserialize, Serialize};
use telemetry::TelemetryEndpoints;
use zkv_runtime::WASM_BINARY;

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

fn chain_properties() -> Properties {
    [
        (
            "ss58Format".to_string(),
            serde_json::Value::from(zkv_runtime::SS58Prefix::get()),
        ),
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
    .with_genesis_config_preset_name("development")
    .build())
}

pub fn local_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("ZKV Local")
    .with_id("zkv_local")
    .with_protocol_id("lacme")
    .with_chain_type(ChainType::Local)
    .with_properties(chain_properties())
    .with_genesis_config_preset_name("local")
    .build())
}

/// To be used when building new testnet chain-spec
pub fn testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("zkVerify Testnet")
    .with_id("zkv_testnet")
    .with_protocol_id("tzkv")
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
    .with_genesis_config_preset_name("testnet")
    .build())
}

// This is a sample unit test
// Following Rust convention, unit tests are appended in the same file as the module they are
// testing. This is acceptable and should not create confusion, as long as the tests have a
// very narrow scope - i.e. for verifying the behavior of a single function of a module.
#[cfg(test)]
mod tests {
    use super::*;

    // This test checks whether the local testnet genesis configuration is generated correctly
    #[test]
    fn local_testnet_genesis_should_be_valid() {
        assert!(testnet_config().is_ok());
    }
}
