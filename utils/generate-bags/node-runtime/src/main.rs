// Copyright 2024-2026, Horizen Labs, Inc.

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

//! Make the set of bag thresholds to be used with pallet-bags-list.

use clap::Parser;
use generate_bags::generate_thresholds;
use std::path::PathBuf;
use subxt::{OnlineClient, SubstrateConfig};

#[derive(Debug, Parser)]
// #[clap(author, version, about)]
struct Opt {
    /// How many bags to generate.
    #[arg(long, default_value_t = 200)]
    n_bags: usize,

    /// Where to write the output.
    output: PathBuf,

    /// The total issuance of the native currency.
    #[arg(short, long)]
    total_issuance: Option<u128>,

    /// The minimum account balance (i.e. existential deposit) for the native currency.
    #[arg(short, long)]
    minimum_balance: Option<u128>,

    /// The WebSocket URL of the zk-verify node.
    #[arg(long, default_value = "ws://127.0.0.1:9944")]
    node_url: String,
}

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "../artifacts/zk_verify_runtime_metadata.scale")]
pub mod zk_verify {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Opt {
        n_bags,
        output,
        total_issuance,
        minimum_balance,
        node_url,
    } = Opt::parse();

    let api = OnlineClient::<SubstrateConfig>::from_url(node_url).await?;

    let total_issuance_ = match total_issuance {
        Some(issuance) => issuance,
        None => api
            .storage()
            .at_latest()
            .await?
            .fetch(&zk_verify::storage().balances().total_issuance())
            .await?
            .unwrap_or_default(),
    };

    let minimum_balance_ = match minimum_balance {
        Some(balance) => balance,
        None => {
            let existential_deposit_query =
                &zk_verify::constants().balances().existential_deposit();
            api.constants().at(existential_deposit_query)?
        }
    };

    generate_thresholds::<zkv_runtime::Runtime>(
        n_bags,
        &output,
        total_issuance_,
        minimum_balance_,
    )?;

    Ok(())
}
