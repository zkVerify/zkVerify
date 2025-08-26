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

#![cfg(any(test, feature = "runtime-benchmarks"))]

pub struct TestData<T: crate::Config> {
    pub vk: crate::Vk<T>,
    pub proof: crate::Proof<T>,
    pub pubs: crate::Pubs,
}

pub fn get_parameterized_test_data<T: crate::Config>(
    degree: usize,
    config: crate::vk::Plonky2Config,
) -> TestData<T> {
    let data = match config {
        crate::vk::Plonky2Config::Poseidon => DATA_POSEIDON,
        crate::vk::Plonky2Config::Keccak => DATA_KECCAK,
    };

    TestData {
        vk: crate::VkWithConfig::new(config, data[degree - MIN_DEGREE].vk.to_vec()),
        proof: crate::Proof::new(data[degree - MIN_DEGREE].proof.to_vec()),
        pubs: data[degree - MIN_DEGREE].pubs.to_vec(),
    }
}

struct Data {
    vk: &'static [u8],
    proof: &'static [u8],
    pubs: &'static [u8],
}

const MIN_DEGREE: usize = 2;
static DATA_POSEIDON: &[Data] = &[
    Data {
        vk: include_bytes!("resources/degree_2/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_2/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_2/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_3/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_3/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_3/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_4/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_4/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_4/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_5/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_5/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_5/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_6/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_6/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_6/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_7/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_7/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_7/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_8/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_8/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_8/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_9/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_9/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_9/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_10/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_10/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_10/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_11/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_11/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_11/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_12/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_12/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_12/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_13/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_13/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_13/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_14/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_14/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_14/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_15/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_15/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_15/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_16/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_16/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_16/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_17/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_17/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_17/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_18/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_18/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_18/uncompressed/poseidon/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_19/uncompressed/poseidon/vk.bin"),
        proof: include_bytes!("resources/degree_19/uncompressed/poseidon/proof.bin"),
        pubs: include_bytes!("resources/degree_19/uncompressed/poseidon/pubs.bin"),
    },
];

static DATA_KECCAK: &[Data] = &[
    Data {
        vk: include_bytes!("resources/degree_2/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_2/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_2/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_3/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_3/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_3/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_4/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_4/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_4/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_5/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_5/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_5/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_6/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_6/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_6/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_7/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_7/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_7/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_8/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_8/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_8/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_9/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_9/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_9/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_10/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_10/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_10/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_11/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_11/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_11/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_12/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_12/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_12/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_13/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_13/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_13/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_14/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_14/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_14/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_15/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_15/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_15/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_16/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_16/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_16/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_17/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_17/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_17/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_18/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_18/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_18/uncompressed/keccak/pubs.bin"),
    },
    Data {
        vk: include_bytes!("resources/degree_19/uncompressed/keccak/vk.bin"),
        proof: include_bytes!("resources/degree_19/uncompressed/keccak/proof.bin"),
        pubs: include_bytes!("resources/degree_19/uncompressed/keccak/pubs.bin"),
    },
];
