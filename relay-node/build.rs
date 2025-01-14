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

use std::{env, path::PathBuf};

use native_cache::rocksdb;

fn main() {
    substrate_build_script_utils::generate_cargo_keys();
    // For the node/worker version check, make sure we always rebuild the node and binary workers
    // when the version changes.

    let profile = env::var("PROFILE").expect("Should have a PROFILE environment variable");
    let cache_root: PathBuf = PathBuf::from(env::var("CARGO_WORKSPACE_DIR").unwrap()).join("deps");
    native_cache::handle_dependencies("../target", &rocksdb(&cache_root, &profile), &profile)
        .expect("Cannot handle cache");
}
