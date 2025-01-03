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
    "32e1a33f3988c3cdf127e709cc0323a258b28df750b7a2d5ddc4c5e37f007d99"
));

pub static VALID_PROOF: [u8; include_bytes!("resources/valid_proof.bin").len()] =
    *include_bytes!("resources/valid_proof.bin");

pub static VALID_PUBS: [u8; 4] = hex_literal::hex!("01000078");
