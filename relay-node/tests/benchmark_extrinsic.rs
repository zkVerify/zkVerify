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

use assert_cmd::cargo::cargo_bin;
use std::{process::Command, result::Result};

mod common;

static EXTRINSICS: [(&str, &str); 2] = [("system", "remark"), ("balances", "transfer_keep_alive")];

/// `benchmark extrinsic` works for all dev runtimes and some extrinsics.
#[test]
fn benchmark_extrinsic_works() {
    let chain = "dev";
    for (pallet, extrinsic) in EXTRINSICS {
        assert!(benchmark_extrinsic(chain, pallet, extrinsic).is_ok());
    }
}

/// `benchmark extrinsic` rejects all non-dev runtimes.
#[test]
fn benchmark_extrinsic_rejects_non_dev_runtimes() {
    assert!(benchmark_extrinsic("local", "system", "remark").is_err());
}

fn benchmark_extrinsic(runtime: &str, pallet: &str, extrinsic: &str) -> Result<(), String> {
    let status = Command::new(cargo_bin(common::NODE))
        .args(["benchmark", "extrinsic", "--chain", runtime])
        .args(["--pallet", pallet, "--extrinsic", extrinsic])
        // Run with low repeats for faster execution.
        .args(["--repeat=1", "--warmup=1", "--max-ext-per-block=1"])
        .status()
        .map_err(|e| format!("command failed: {e:?}"))?;

    if !status.success() {
        return Err("Command failed".into());
    }

    Ok(())
}
