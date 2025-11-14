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

#[macro_export]
macro_rules! runtime_version {
    ( $spec_name:tt ) => {
        // To learn more about runtime versioning, see:
        // https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
        #[sp_version::runtime_version]
        pub const VERSION: RuntimeVersion = RuntimeVersion {
            spec_name: Cow::Borrowed($spec_name),
            impl_name: Cow::Borrowed("zkv-node"),
            authoring_version: 1,
            spec_version: 1_003_001,
            impl_version: 1,
            apis: RUNTIME_API_VERSIONS,
            transaction_version: 1,
            system_version: 1,
        };
    };
}

// Set the output address to start with ZK, and sometimes the third is v (since 17%)
pub const SS58_ZKV_PREFIX: u16 = 8741;
// ASCII for 'Z'+'K'+'V'
pub const SS58_VOLTA_PREFIX: u16 = 251;
