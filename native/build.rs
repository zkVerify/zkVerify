// Copyright 2024, Horizen Labs, Inc.

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

use std::env;
use std::path::PathBuf;

use native_cache::ultraplonk;

fn main() {
    let profile = env::var("PROFILE").expect("Should have a PROFILE environment variable");
    let cache_root: PathBuf = PathBuf::from(env::var("CARGO_WORKSPACE_DIR").unwrap()).join("deps");
    native_cache::handle_dependency("../target", &ultraplonk(&cache_root, &profile), &profile)
        .expect("Cannot handle cache");
}
