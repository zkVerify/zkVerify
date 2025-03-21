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
use common::run_with_timeout;
use std::{
    process::{self, Command},
    time::Duration,
};

pub mod common;

#[tokio::test]
async fn check_version() {
    run_with_timeout(Duration::from_secs(5), async move {
        let out = Command::new(cargo_bin(common::NODE))
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .args([
                "--version",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        let v = std::env!("CARGO_PKG_VERSION");
        assert!(stdout.starts_with(&format!("zkv-relay {v}")),
                "Version missmatch. Crate version = {v}, but node version string `{stdout}`. Check NODE_VERSION constant definition in cli crate.");
    })
        .await
}
