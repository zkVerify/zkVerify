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

/// The extensions for the [`VoltaChainSpec`].
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
pub struct Extensions {
    light_sync_state: LightSyncStateExtension,
}

/// The Generic `ChainSpec`.
pub type GenericChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The `ChainSpec` parameterized for the zkv runtime.
pub type ZkvChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The `ChainSpec` parameterized for the volta runtime.
pub type VoltaChainSpec = sc_service::GenericChainSpec<Extensions>;

pub fn volta_development_config() -> Result<VoltaChainSpec, String> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "volta")] {
            volta::development_config()
        } else {
            Err("`volta` testnet feature is not enabled".to_string())
        }
    }
}

pub fn volta_local_config() -> Result<VoltaChainSpec, String> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "volta")] {
            volta::local_config()
        } else {
            Err("`volta` testnet feature is not enabled".to_string())
        }
    }
}

cfg_if::cfg_if! {

    if #[cfg(feature = "volta")] {
        mod volta {
            use super::*;


            pub fn development_config() -> Result<VoltaChainSpec, String> {
                Ok(VoltaChainSpec::builder(
                    zkv_runtime::WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
                    Default::default(),
                )
                    .with_name("Development")
                    .with_id("volta_dev")
                    .with_chain_type(ChainType::Development)
                    .with_properties(chain_properties())
                    .with_genesis_config_preset_name("development")
                    .build())
            }

            pub fn local_config() -> Result<VoltaChainSpec, String> {
                Ok(VoltaChainSpec::builder(
                    zkv_runtime::WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
                    Default::default(),
                )
                    .with_name("Volta Local Testnet")
                    .with_id("volta_local")
                    .with_protocol_id("tzkv")
                    .with_chain_type(ChainType::Local)
                    .with_properties(chain_properties())
                    .with_genesis_config_preset_name("local")
                    .build())
            }
        }
    } else {
        mod zkv {
            use super::*;

            pub fn development_config() -> Result<ZkvChainSpec, String> {
                Ok(ZkvChainSpec::builder(
                    zkv_runtime::WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?,
                    Default::default(),
                )
                .with_name("zkVerify Development")
                .with_id("zkv_mainnet_dev")
                .with_chain_type(ChainType::Development)
                .with_properties(chain_properties())
                .with_genesis_config_preset_name("development")
                .build())
            }

            pub fn local_config() -> Result<ZkvChainSpec, String> {
                Ok(ZkvChainSpec::builder(
                    zkv_runtime::WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?,
                    Default::default(),
                )
                .with_name("zkVerify Local Testnet")
                .with_id("zkv_mainnet_local")
                .with_protocol_id("zkv")
                .with_chain_type(ChainType::Local)
                .with_properties(chain_properties())
                .with_genesis_config_preset_name("local")
                .build())
            }
        }
    }
}
fn chain_properties() -> Properties {
    [
        (
            "ss58Format".to_string(),
            serde_json::Value::from(zkv_runtime::SS58Prefix::get()),
        ),
        (
            "tokenSymbol".to_string(),
            serde_json::Value::from(zkv_runtime::TokenSymbol::get()),
        ),
        ("tokenDecimals".to_string(), serde_json::Value::from(18_u8)),
    ]
    .into_iter()
    .collect()
}

pub fn zkverify_development_config() -> Result<ZkvChainSpec, String> {
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "volta"))] {
            zkv::development_config()
        } else {
            Err("`volta` testnet feature should not be enabled".to_string())
        }
    }
}

pub fn zkverify_local_config() -> Result<ZkvChainSpec, String> {
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "volta"))] {
            zkv::local_config()
        } else {
            Err("`volta` testnet feature should not be enabled".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(not(feature = "volta"), should_panic)]
    fn volta_development_genesis() {
        assert!(volta_development_config().is_ok());
    }

    #[test]
    #[cfg_attr(not(feature = "volta"), should_panic)]
    fn volta_local_genesis() {
        assert!(volta_local_config().is_ok());
    }

    #[test]
    #[cfg_attr(feature = "volta", should_panic)]
    fn zkverify_development_genesis() {
        assert!(zkverify_development_config().is_ok());
    }

    #[test]
    #[cfg_attr(feature = "volta", should_panic)]
    fn zkverify_local_genesis() {
        assert!(zkverify_local_config().is_ok());
    }
}
