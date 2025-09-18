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

pub mod any_mock {
    pub static VK: sp_core::H256 = sp_core::H256([0xad; 32]);
    pub static PUBS: &[u8] = &hex_literal::hex!("0000000000000000");
    pub static PROOF: &[u8] = include_bytes!("resources/v_2_1_poseidon2_16.bin");
}
pub mod v2_1 {
    pub static VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "8e3794e8255e7810de2be7710fe19f79e538de060f038a21b24529e28d0b744c"
    ));
    pub static PUBS_22: &[u8] = &hex_literal::hex!("00003c0000000000");
    pub static PUBS_16: &[u8] = &hex_literal::hex!("1c40000000000000");

    pub static PROOF_SUCCINCT: &[u8] = include_bytes!("resources/v_2_1_succinct_22.bin");
    pub static PUBS_SUCCINCT: &[u8] = PUBS_22;

    pub static PROOF_POSEIDON2_22: &[u8] = include_bytes!("resources/v_2_1_poseidon2_22.bin");
    pub static PROOF_POSEIDON2_16: &[u8] = include_bytes!("resources/v_2_1_poseidon2_16.bin");

    pub static VALID_PROOF: &[u8] = PROOF_POSEIDON2_16;
    pub static PUBS: &[u8] = PUBS_16;

    pub static PUBS_COMPOSITE_3_SLOTS: &[u8] = &hex_literal::hex!("1a00200000000000");
    pub static PROOF_COMPOSITE_3_SLOTS: &[u8] =
        include_bytes!("resources/v_2_1_poseidon2_composite_3_slots.bin");

    pub static PROOF_UPPER_BOUND: &[u8] = include_bytes!("resources/v_2_1_poseidon2_22.bin");
    pub static PUBS_UPPER_BOUND: &[u8] = &hex_literal::hex!("00003c0000000000");
}

pub mod v2_2 {
    pub static VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "096adc2485db3717d21133bcc66068f1924b172b555be45fa9a3a68d7988d5c4"
    ));
    pub static PUBS_22: &[u8] = &hex_literal::hex!("00003c0000000000");
    pub static PUBS_16: &[u8] = &hex_literal::hex!("1c40000000000000");

    pub static PROOF_SUCCINCT: &[u8] = include_bytes!("resources/v_2_2_succinct_22.bin");
    pub static PUBS_SUCCINCT: &[u8] = PUBS_22;

    pub static PROOF_POSEIDON2_22: &[u8] = include_bytes!("resources/v_2_2_poseidon2_22.bin");
    pub static PROOF_POSEIDON2_16: &[u8] = include_bytes!("resources/v_2_2_poseidon2_16.bin");
}

pub mod v2_3 {
    pub static VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "23aa8e69b6fe32677982830a2a9e43b2d16163365fcbf8d191a819f3274dec6b"
    ));
    pub static PUBS_22: &[u8] = &hex_literal::hex!("19003c0000000000");
    pub static PUBS_16: &[u8] = &hex_literal::hex!("0f40000000000000");

    pub static PROOF_SUCCINCT: &[u8] = include_bytes!("resources/v_2_3_succinct_22.bin");
    pub static PUBS_SUCCINCT: &[u8] = PUBS_22;

    pub static PROOF_POSEIDON2_22: &[u8] = include_bytes!("resources/v_2_3_poseidon2_22.bin");
    pub static PROOF_POSEIDON2_16: &[u8] = include_bytes!("resources/v_2_3_poseidon2_16.bin");
}

pub mod v3_0 {
    pub static VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "7f423b2f1095359fe3305c5328e9f5cc3eba01c9f9559bfbf6a3a59d73cc77ae"
    ));
    pub static PUBS_22: &[u8] = &hex_literal::hex!("22003c0000000000");
    pub static PUBS_16: &[u8] = &hex_literal::hex!("3a40000000000000");

    pub static PROOF_SUCCINCT: &[u8] = include_bytes!("resources/v_3_0_succinct_22.bin");
    pub static PUBS_SUCCINCT: &[u8] = PUBS_22;

    pub static PROOF_POSEIDON2_22: &[u8] = include_bytes!("resources/v_3_0_poseidon2_22.bin");
    pub static PROOF_POSEIDON2_16: &[u8] = include_bytes!("resources/v_3_0_poseidon2_16.bin");
}
