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
fn chain_properties() -> Properties {
    [
        (
            "ss58Format".to_string(),
            serde_json::Value::from(zkv_runtime::SS58VoltaPrefix::get()),
        ),
        ("tokenSymbol".to_string(), serde_json::Value::from("tVFY")),
        ("tokenDecimals".to_string(), serde_json::Value::from(18_u8)),
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_genesis() {
        assert!(development_config().is_ok());
    }

    #[test]
    fn local_genesis() {
        assert!(local_config().is_ok());
    }
}
