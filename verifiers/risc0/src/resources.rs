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

pub mod v1_0 {
    pub static VALID_VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "32e1a33f3988c3cdf127e709cc0323a258b28df750b7a2d5ddc4c5e37f007d99"
    ));

    pub static VALID_PROOF: &[u8] = include_bytes!("resources/valid_proof_v1_0.bin");

    pub static VALID_PUBS: &[u8] = &hex_literal::hex!("01000078");
}

pub mod v1_1 {
    pub static VALID_VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "2addbbeb4ddb2f2ec2b4a0a8a21c03f7d3bf42cfd2ee9f4a69d2ebd9974218b6"
    ));

    pub static VALID_PROOF: &[u8] = include_bytes!("resources/v_1_1_poseidon2_16.bin");

    pub static VALID_PUBS: &[u8] = &hex_literal::hex!("8105000000000000");
}

pub mod v1_2 {
    pub static VALID_VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "9db9988d9fbcacadf2bd29fc7c60b98bc4234342fe536eb983169eb6cc248009"
    ));

    pub static VALID_PROOF: &[u8] = include_bytes!("resources/v_1_2_succinct_22.bin");

    pub static VALID_PUBS: &[u8] = &hex_literal::hex!("1d64010000000000");

    pub static VALID_PROOF_COMPOSITE_3_SLOTS: &[u8] =
        include_bytes!("resources/v_1_2_poseidon2_22.bin");

    pub static VALID_VK_UPPER_BOUND: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "f9ac6bf258c7e19a574346ff57fec724c087a7dc5f1541a71b4504ebba82cb2f"
    ));
    pub static VALID_PUBS_UPPER_BOUND: &[u8] = &hex_literal::hex!("0d003c0000000000");
    pub static VALID_PROOF_UPPER_BOUND: &[u8] = include_bytes!("resources/upper_bound.bin");
}

pub mod v2_0 {
    pub static VALID_VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "46eb54959aaff64f405fb04575c4a2387630de3a7938d679494b8f31f859cb01"
    ));

    pub static VALID_PROOF: &[u8] = include_bytes!("resources/v_2_0_succinct_22.bin");

    pub static VALID_PUBS: &[u8] = &hex_literal::hex!("08003c0000000000");
}

pub mod v2_1 {
    pub static VALID_VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "8e3794e8255e7810de2be7710fe19f79e538de060f038a21b24529e28d0b744c"
    ));

    pub static VALID_PROOF: &[u8] = include_bytes!("resources/v_2_1_succinct_22.bin");

    pub static VALID_PUBS: &[u8] = &hex_literal::hex!("00003c0000000000");
}
