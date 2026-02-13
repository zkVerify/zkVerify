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

#![cfg(unix)]

use common::run_with_timeout;
use nix::{
    sys::signal::{kill, Signal::SIGINT},
    unistd::Pid,
};
use std::{
    process::{self, Command},
    time::Duration,
};
use tempfile::tempdir;

pub mod common;

const DB_PATH: &str = "chains/volta_dev";
const ROCKS: &str = "db/full";
const PARITY: &str = "paritydb/full";

#[tokio::test]
async fn purge_chain_rocksdb_works() {
    run_with_timeout(Duration::from_secs(5 * 60), async move {
        let tmpdir = tempdir().expect("could not create temp dir");

        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("zkv-relay"))
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .args(["--dev", "-d"])
            .arg(tmpdir.path())
            .arg("--port")
            .arg("33034")
            .arg("--no-hardware-benchmarks")
            .spawn()
            .unwrap();

        let (ws_url, _) = common::find_ws_url_from_output(cmd.stderr.take().unwrap());

        // Let it produce 1 block.
        common::wait_n_finalized_blocks(1, &ws_url).await;

        // Send SIGINT to node.
        kill(Pid::from_raw(cmd.id().try_into().unwrap()), SIGINT).unwrap();
        // Wait for the node to handle it and exit.
        assert!(cmd.wait().unwrap().success());
        assert!(tmpdir.path().join(DB_PATH).exists());
        assert!(tmpdir.path().join(DB_PATH).join(ROCKS).exists());

        // Purge chain
        let status = Command::new(assert_cmd::cargo::cargo_bin!("zkv-relay"))
            .args(["purge-chain", "--dev", "-d"])
            .arg(tmpdir.path())
            .arg("-y")
            .status()
            .unwrap();
        assert!(status.success());

        // Make sure that the chain folder exists, but `db/full` is deleted.
        assert!(tmpdir.path().join(DB_PATH).exists());
        assert!(!tmpdir.path().join(DB_PATH).join(ROCKS).exists());
    })
    .await;
}

#[tokio::test]
async fn purge_chain_paritydb_works() {
    run_with_timeout(Duration::from_secs(5 * 60), async move {
        let tmpdir = tempdir().expect("could not create temp dir");

        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("zkv-relay"))
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .args(["--dev", "-d"])
            .arg(tmpdir.path())
            .arg("--database")
            .arg("paritydb")
            .arg("--no-hardware-benchmarks")
            .spawn()
            .unwrap();

        let (ws_url, _) = common::find_ws_url_from_output(cmd.stderr.take().unwrap());

        // Let it produce 1 block.
        common::wait_n_finalized_blocks(1, &ws_url).await;

        // Send SIGINT to node.
        kill(Pid::from_raw(cmd.id().try_into().unwrap()), SIGINT).unwrap();
        // Wait for the node to handle it and exit.
        assert!(cmd.wait().unwrap().success());
        assert!(tmpdir.path().join(DB_PATH).exists());
        assert!(tmpdir.path().join(DB_PATH).join(PARITY).exists());

        // Purge chain
        let status = Command::new(assert_cmd::cargo::cargo_bin!("zkv-relay"))
            .args(["purge-chain", "--dev", "-d"])
            .arg(tmpdir.path())
            .arg("--database")
            .arg("paritydb")
            .arg("-y")
            .status()
            .unwrap();
        assert!(status.success());

        // Make sure that the chain folder exists, but `db/full` is deleted.
        assert!(tmpdir.path().join(DB_PATH).exists());
        assert!(!tmpdir.path().join(DB_PATH).join(PARITY).exists());
    })
    .await;
}
