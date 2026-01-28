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

runtime_version!("zkv-runtime");

pub const SS58_PREFIX: u16 = super::SS58_ZKV_PREFIX;

pub const ZKV_GENESIS_HASH: [u8; 32] =
    hex_literal::hex!("060e3dd3fa2904d031206bb913c954687a2bcc350e5a83d33d9e273ad21460f1");
