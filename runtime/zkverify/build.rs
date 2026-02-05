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
        // Use legacy wasm target (wasm32-unknown-unknown) instead of wasm32v1-none
        // because some dependencies (bit-vec via risc0-circuit-rv32im) don't properly
        // support the stricter wasm32v1-none target.
        // TODO: Remove this once risc0-verifier upstream fixes the bit-vec dependency.
        std::env::set_var("WASM_BUILD_LEGACY_TARGET", "1");
        // // Also disable building std since we're using the legacy target with Rust >= 1.84
        // std::env::set_var("WASM_BUILD_STD", "0");
        use wasm_builder_ext::WasmBuilderExt;
        substrate_wasm_builder::WasmBuilder::init_with_defaults()
            .handle_metadata_hash()
            .build()
    }
}

#[cfg(feature = "std")]
mod wasm_builder_ext {
    use substrate_wasm_builder::WasmBuilder;

    pub trait WasmBuilderExt: Sized {
        fn handle_metadata_hash(self) -> Self {
            self
        }
    }

    impl WasmBuilderExt for WasmBuilder {
        /// We cannot enable it as default because this option requires building the WASM runtime two
        /// times, one to get the metadata and te recompile it with the metadata hash in an environment
        /// variable.
        #[cfg(feature = "metadata-hash")]
        fn handle_metadata_hash(self) -> Self {
            if std::env::var_os("ZKV_FORCE_DISABLE_METADATA_HASH").is_none() {
                const TOKEN_SYMBOL: &str = if cfg!(not(feature = "volta")) {
                    "VFY"
                } else {
                    "tVFY"
                };
                const TOKEN_DECIMAL: u8 = 18;
                self.enable_metadata_hash(TOKEN_SYMBOL, TOKEN_DECIMAL)
            } else {
                self
            }
        }
    }
}
