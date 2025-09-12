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
use sc_network::config::MultiaddrWithPeerId;
use sc_service::{ChainType, Properties};
use sc_sync_state_rpc::LightSyncStateExtension;
use serde::{Deserialize, Serialize};
use telemetry::TelemetryEndpoints;

/// The extensions for the [`VoltaChainSpec`].
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
pub struct Extensions {
    light_sync_state: LightSyncStateExtension,
}

// The URL for the telemetry server.
const TELEMETRY_URL: &str = "wss://telemetry.zkverify.io/submit/";

/// The Generic `ChainSpec`.
pub type GenericChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The `ChainSpec` parameterized for the zkv runtime.
pub type ZkvChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The `ChainSpec` parameterized for the volta runtime.
pub type VoltaChainSpec = sc_service::GenericChainSpec<Extensions>;

fn boot_nodes(address_id: &[(&str, &str)]) -> Vec<MultiaddrWithPeerId> {
    const PROTOCOL_PATHS: &[&str] = &["tcp/30333/p2p", "tcp/30334/ws/p2p", "tcp/443/wss/p2p"];

    address_id
        .iter()
        .flat_map(|&(dns, id)| {
            PROTOCOL_PATHS.iter().map(move |&p| {
                format!("/dns/{dns}/{p}/{id}")
                    .parse()
                    .expect("MultiaddrWithPeerId")
            })
        })
        .collect()
}

fn volta_chain_properties() -> Properties {
    [
        (
            "ss58Format".to_string(),
            serde_json::Value::from(volta_runtime::SS58Prefix::get()),
        ),
        ("tokenSymbol".to_string(), serde_json::Value::from("tVFY")),
        ("tokenDecimals".to_string(), serde_json::Value::from(18_u8)),
    ]
    .into_iter()
    .collect()
}

pub fn volta_development_config() -> Result<VoltaChainSpec, String> {
    Ok(VoltaChainSpec::builder(
        volta_runtime::WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("Development")
    .with_id("volta_dev")
    .with_chain_type(ChainType::Development)
    .with_properties(volta_chain_properties())
    .with_genesis_config_preset_name("development")
    .build())
}

pub fn volta_local_config() -> Result<VoltaChainSpec, String> {
    Ok(VoltaChainSpec::builder(
        volta_runtime::WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("Volta Local Testnet")
    .with_id("volta_local")
    .with_protocol_id("tzkv")
    .with_chain_type(ChainType::Local)
    .with_properties(volta_chain_properties())
    .with_genesis_config_preset_name("local")
    .build())
}

/// To be used when building new testnet chain-spec
pub fn volta_staging_config() -> Result<VoltaChainSpec, String> {
    const VOLTA_BOOTNODE_1_DNS: &str = "boot-node-tn-volta-1.zkverify.io";
    const VOLTA_BOOTNODE_1_PEER_ID: &str = "12D3KooWCso7aZ93X8uY82CExtCqSDsvr8p5NAEVD43iVebF72VR";
    const VOLTA_BOOTNODE_2_DNS: &str = "boot-node-tn-volta-2.zkverify.io";
    const VOLTA_BOOTNODE_2_PEER_ID: &str = "12D3KooWKXzW6nAjfwpbHJ5oqHyCsKpMFG8UxJzEYjcrRzEry5SC";

    Ok(VoltaChainSpec::builder(
        volta_runtime::WASM_BINARY.ok_or_else(|| "Testnet wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("Volta Staging Testnet")
    .with_id("volta_staging")
    .with_protocol_id("tzkv")
    .with_chain_type(ChainType::Live)
    .with_boot_nodes(boot_nodes(&[
        (VOLTA_BOOTNODE_1_DNS, VOLTA_BOOTNODE_1_PEER_ID),
        (VOLTA_BOOTNODE_2_DNS, VOLTA_BOOTNODE_2_PEER_ID),
    ]))
    .with_telemetry_endpoints(
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), telemetry::CONSENSUS_INFO)])
            .expect("Horizen Labs telemetry url is valid; qed"),
    )
    .with_properties(volta_chain_properties())
    .with_genesis_config_preset_name("staging")
    .build())
}

fn zkv_chain_properties() -> Properties {
    [
        (
            "ss58Format".to_string(),
            serde_json::Value::from(volta_runtime::SS58Prefix::get()),
        ),
        ("tokenSymbol".to_string(), serde_json::Value::from("VFY")),
        ("tokenDecimals".to_string(), serde_json::Value::from(18_u8)),
    ]
    .into_iter()
    .collect()
}

pub fn zkverify_development_config() -> Result<VoltaChainSpec, String> {
    Ok(ZkvChainSpec::builder(
        zkv_runtime::WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("zkVerify Development")
    .with_id("zkv_mainnet_dev")
    .with_chain_type(ChainType::Development)
    .with_properties(zkv_chain_properties())
    .with_genesis_config_preset_name("development")
    .build())
}

pub fn zkverify_local_config() -> Result<VoltaChainSpec, String> {
    Ok(ZkvChainSpec::builder(
        zkv_runtime::WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("zkVerify Local Testnet")
    .with_id("zkv_mainnet_local")
    .with_protocol_id("zkv")
    .with_chain_type(ChainType::Local)
    .with_properties(zkv_chain_properties())
    .with_genesis_config_preset_name("local")
    .build())
}

/// To be used when building new testnet chain-spec
pub fn zkverify_staging_config() -> Result<VoltaChainSpec, String> {
    const ZKV_BOOTNODE_1_DNS: &str = "boot-node-zkverify-1.horizenlabs.io";
    const ZKV_BOOTNODE_1_PEER_ID: &str = "12D3KooWAuwa5TH5y6h6zeWVKrYTfawqU9R2t8r8Gw4wJBCEADzX";
    const ZKV_BOOTNODE_2_DNS: &str = "boot-node-zkverify-1.zkverify.io";
    const ZKV_BOOTNODE_2_PEER_ID: &str = "12D3KooWQukFbZbQUbaG2iJ6ecMJhy2HTeyWaEaRvZA2BwasVCCT";
    const ZKV_BOOTNODE_3_DNS: &str = "boot-node-zkverify-1.horizen.io";
    const ZKV_BOOTNODE_3_PEER_ID: &str = "12D3KooWBw26BpQuNwau2dt4b65c5bB72q6kmyis4m6NX8ZeycbC";

    Ok(ZkvChainSpec::builder(
        zkv_runtime::WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("zkVerify Staging")
    .with_id("zkv_mainnet_staging")
    .with_protocol_id("zkv")
    .with_chain_type(ChainType::Live)
    .with_boot_nodes(boot_nodes(&[
        (ZKV_BOOTNODE_1_DNS, ZKV_BOOTNODE_1_PEER_ID),
        (ZKV_BOOTNODE_2_DNS, ZKV_BOOTNODE_2_PEER_ID),
        (ZKV_BOOTNODE_3_DNS, ZKV_BOOTNODE_3_PEER_ID),
    ]))
    .with_telemetry_endpoints(
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), telemetry::CONSENSUS_INFO)])
            .expect("Horizen Labs telemetry url is valid; qed"),
    )
    .with_properties(zkv_chain_properties())
    .with_genesis_config_preset_name("staging")
    .build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volta_development_genesis_should_be_valid() {
        assert!(volta_development_config().is_ok());
    }

    #[test]
    fn volta_local_genesis_should_be_valid() {
        assert!(volta_local_config().is_ok());
    }

    #[test]
    fn volta_staging_genesis_should_be_valid() {
        assert!(volta_staging_config().is_ok());
    }

    #[test]
    fn zkverify_development_genesis_should_be_valid() {
        assert!(zkverify_development_config().is_ok());
    }

    #[test]
    fn zkverify_local_genesis_should_be_valid() {
        assert!(zkverify_local_config().is_ok());
    }

    #[test]
    fn zkverify_staging_genesis_should_be_valid() {
        assert!(zkverify_staging_config().is_ok());
    }
}
