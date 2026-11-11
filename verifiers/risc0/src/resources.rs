// Copyright 2024-2026, Horizen Labs, Inc.
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

pub mod v3_1 {
    pub static VK: sp_core::H256 = sp_core::H256(hex_literal::hex!(
        "5bfbc39295e3a0c0572640ac729b9223942efc130e1768b1eb142abaa843994a"
    ));
    pub static PUBS_22: &[u8] = &hex_literal::hex!("27003c0000000000");
    pub static PUBS_16: &[u8] = &hex_literal::hex!("3f40000000000000");

    pub static PROOF_SUCCINCT: &[u8] = include_bytes!("resources/v_3_1_succinct_22.bin");
    pub static PUBS_SUCCINCT: &[u8] = PUBS_22;

    pub static PROOF_SUCCINCT_16: &[u8] = include_bytes!("resources/v_3_1_succinct_16.bin");
    pub static PUBS_SUCCINCT_16: &[u8] = PUBS_16;

    pub static PROOF_POSEIDON2_22: &[u8] = include_bytes!("resources/v_3_1_poseidon2_22.bin");
    pub static PROOF_POSEIDON2_16: &[u8] = include_bytes!("resources/v_3_1_poseidon2_16.bin");

    pub static VALID_PROOF: &[u8] = PROOF_POSEIDON2_16;
    pub static PUBS: &[u8] = PUBS_16;

    pub static PUBS_COMPOSITE_3_SLOTS: &[u8] = &hex_literal::hex!("4100200000000000");
    pub static PROOF_COMPOSITE_3_SLOTS: &[u8] =
        include_bytes!("resources/v_3_1_poseidon2_composite_3_slots.bin");

    pub static PROOF_UPPER_BOUND: &[u8] = PROOF_POSEIDON2_22;
    pub static PUBS_UPPER_BOUND: &[u8] = PUBS_22;
}
