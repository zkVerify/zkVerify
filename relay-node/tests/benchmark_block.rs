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

// Unix only since it uses signals.
#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use common::run_with_timeout;
use nix::{
    sys::signal::{kill, Signal::SIGINT},
    unistd::Pid,
};
use std::{
    path::Path,
    process::{self, Command},
    result::Result,
    time::Duration,
};
use tempfile::tempdir;

pub mod common;

/// `benchmark block` works for all dev runtimes using the wasm executor.
#[tokio::test]
async fn benchmark_block_works() {
    run_with_timeout(Duration::from_secs(10 * 60), async move {
        let tmp_dir = tempdir().expect("could not create a temp dir");
        let base_path = tmp_dir.path();
        let chain = "dev";

        // Build a chain with a single block.
        build_chain(chain, base_path).await;
        // Benchmark the one block.
        benchmark_block(chain, base_path, 1).unwrap();
    })
    .await
}

/// Builds a chain with one block for the given runtime and base path.
async fn build_chain(runtime: &str, base_path: &Path) {
    let mut cmd = Command::new(cargo_bin(common::NODE))
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .args([
            "--chain",
            runtime,
            "--force-authoring",
            "--alice",
            "--unsafe-force-node-key-generation",
        ])
        .arg("-d")
        .arg(base_path)
        .arg("--no-hardware-benchmarks")
        .spawn()
        .unwrap();

    let (ws_url, _) = common::find_ws_url_from_output(cmd.stderr.take().unwrap());

    // Wait for the chain to produce one block.
    common::wait_n_finalized_blocks(1, &ws_url).await;
    // Send SIGINT to node.
    kill(Pid::from_raw(cmd.id().try_into().unwrap()), SIGINT).unwrap();
    assert!(cmd.wait().unwrap().success());
}

/// Benchmarks the given block with the wasm executor.
fn benchmark_block(runtime: &str, base_path: &Path, block: u32) -> Result<(), String> {
    // Invoke `benchmark block` with all options to make sure that they are valid.
    let status = Command::new(cargo_bin(common::NODE))
        .args(["benchmark", "block", "--chain", runtime])
        .arg("-d")
        .arg(base_path)
        .args(["--from", &block.to_string(), "--to", &block.to_string()])
        .args(["--repeat", "1"])
        .args(["--wasm-execution", "compiled"])
        .status()
        .map_err(|e| format!("command failed: {e:?}"))?;

    if !status.success() {
        return Err("Command failed".into());
    }

    Ok(())
}
