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

fn main() {
    #[cfg(feature = "std")]
    {
        std::env::remove_var("CARGO_FEATURE_STD");
        std::env::remove_var("CARGO_FEATURE_DEFAULT");
        let builder = substrate_wasm_builder::WasmBuilder::init_with_defaults();
        // We cannot enable it as default because this option require to build the WASM runtime two
        // time, one to get the metadata and te recompile it with the metadata hash in an environment
        // variable.
        #[cfg(feature = "metadata-hash")]
        let builder = if std::env::var_os("ZKV_FORCE_DISABLE_METADATA_HASH").is_none() {
            builder.enable_metadata_hash("ACME", 18)
        } else {
            builder
        };
        builder.build()
    }
}
