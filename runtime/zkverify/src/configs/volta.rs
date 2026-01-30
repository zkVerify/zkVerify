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

use crate::{runtime_version, RuntimeVersion, RUNTIME_API_VERSIONS};
use alloc::borrow::Cow;

runtime_version!("tzkv-runtime");

// ASCII for 'Z'+'K'+'V'
pub const SS58_PREFIX: u16 = super::SS58_VOLTA_PREFIX;

pub const ZKV_GENESIS_HASH: [u8; 32] =
    hex_literal::hex!("ff7fe5a610f15fe7a0c52f94f86313fb7db7d3786e7f8acf2b66c11d5be7c242");
