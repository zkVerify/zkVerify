// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub static VALID_VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
    "6a54c0dd1af095e69c626811b3fc9f86807cbfb29754fdf1dfa31bee0f5226a7"
));

pub static VALID_VK_SUCCINCT: sp_core::H256 = sp_core::H256(hex_literal::hex!(
    "9db9988d9fbcacadf2bd29fc7c60b98bc4234342fe536eb983169eb6cc248009"
));

pub static VALID_VK_BLOCKS: sp_core::H256 = sp_core::H256(hex_literal::hex!(
    "1392c64d4677b7054b5c666c99dde629cd6aacb4a2abe440fb85b889a509b2e0"
));

pub static VALID_PUBS_SUCCINCT: &[u8] = &hex_literal::hex!("1d64010000000000");

pub static VALID_PUBS_BLOCK_16: &[u8] = &hex_literal::hex!("0e20000000000000");

pub static VALID_PUBS_BLOCK_17: &[u8] = &hex_literal::hex!("0880000000000000");

pub static VALID_PUBS_BLOCK_18: &[u8] = &hex_literal::hex!("0200020000000000");

pub static VALID_PUBS_BLOCK_19: &[u8] = &hex_literal::hex!("0600040000000000");

pub static VALID_PUBS_BLOCK_20: &[u8] = &hex_literal::hex!("04000c0000000000");

pub static VALID_PUBS_BLOCK_21: &[u8] = &hex_literal::hex!("00001c0000000000");
