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

#![allow(dead_code)]

use polkadot_core_primitives::{Block, Hash, Header};
use std::{
    future::Future,
    io::{BufRead, BufReader, Read},
    time::Duration,
};
use substrate_rpc_client::{ws_client, ChainApi};

pub const NODE: &str = "zkv-relay";

/// Run the given `future` and panic if the `timeout` is hit.
pub async fn run_with_timeout(timeout: Duration, future: impl Future<Output = ()>) {
    tokio::time::timeout(timeout, future)
        .await
        .expect("Hit timeout");
}

/// Wait for at least `n` blocks to be finalized from a specified node.
pub async fn wait_n_finalized_blocks(n: usize, url: &str) {
    let mut built_blocks = std::collections::HashSet::new();
    let mut interval = tokio::time::interval(Duration::from_secs(6));

    loop {
        let Ok(rpc) = ws_client(url).await else {
            continue;
        };

        if let Ok(block) = ChainApi::<(), Hash, Header, Block>::finalized_head(&rpc).await {
            built_blocks.insert(block);
            if built_blocks.len() > n {
                break;
            }
        };

        interval.tick().await;
    }
}

/// Read the WS address from the output.
///
/// This is hack to get the actual bound sockaddr because
/// polkadot assigns a random port if the specified port was already bound.
///
/// You must call
/// `Command::new("cmd").stdout(process::Stdio::piped()).stderr(process::Stdio::piped())`
/// for this to work.
pub fn find_ws_url_from_output(read: impl Read + Send) -> (String, String) {
    let mut data = String::new();

    let ws_url = BufReader::new(read)
        .lines()
        .find_map(|line| {
            let line = line.expect("failed to obtain next line from stdout for port discovery");

            data.push_str(&line);

            // does the line contain our port (we expect this specific output from substrate).
            let sock_addr = match line.split_once("Running JSON-RPC server: addr=") {
                None => return None,
                Some((_, after)) => after.split_once(',').unwrap().0,
            };

            Some(format!("ws://{sock_addr}"))
        })
        .unwrap_or_else(|| panic!("Could not find address in process output:\n{}", &data));
    (ws_url, data)
}
