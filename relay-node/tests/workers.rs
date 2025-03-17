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

use std::process::Command;
use zkv_cli::NODE_VERSION;

const PREPARE_WORKER_EXE: &str = env!("CARGO_BIN_EXE_zkv-relay-prepare-worker");
const EXECUTE_WORKER_EXE: &str = env!("CARGO_BIN_EXE_zkv-relay-execute-worker");

#[test]
fn worker_binaries_have_same_version_as_node() {
    let prep_worker_version = Command::new(PREPARE_WORKER_EXE)
        .args(["--version"])
        .output()
        .unwrap()
        .stdout;
    let prep_worker_version = std::str::from_utf8(&prep_worker_version)
        .expect("version is printed as a string; qed")
        .trim();
    assert_eq!(prep_worker_version, NODE_VERSION);

    let exec_worker_version = Command::new(EXECUTE_WORKER_EXE)
        .args(["--version"])
        .output()
        .unwrap()
        .stdout;
    let exec_worker_version = std::str::from_utf8(&exec_worker_version)
        .expect("version is printed as a string; qed")
        .trim();
    assert_eq!(exec_worker_version, NODE_VERSION);
}
