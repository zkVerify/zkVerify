// Copyright 2025, Horizen Labs, Inc.

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

#![cfg(any(test, feature = "runtime-benchmarks"))]
#![allow(unused)]

use crate::ProofType;

// Minimum allowed value for the logarithm of the polynomial evaluation domain size.
const MIN_BENCHMARKED_LOG_CIRCUIT_SIZE: u64 = 7;

// Struct containing the parameters pointing to the exact benchmark data that should be used.
struct TestParams {
    log_circuit_size: u64,
    proof_type: ProofType,
}

impl TestParams {
    fn new(log_circuit_size: u64, proof_type: ProofType) -> Self {
        Self {
            log_circuit_size,
            proof_type,
        }
    }
}

pub struct TestData {
    pub vk: crate::Vk,
    pub proof: crate::VersionedProof,
    pub pubs: crate::Pubs,
}

pub fn get_parameterized_test_data(test_params: TestParams) -> TestData {
    let data = match test_params.proof_type {
        ProofType::ZK => DATA_ZK,
        ProofType::Plain => DATA_PLAIN,
    };
    let raw_test_data_idx =
        (test_params.log_circuit_size - MIN_BENCHMARKED_LOG_CIRCUIT_SIZE) as usize;
    let raw_test_data = &data[raw_test_data_idx];

    let vk: [u8; crate::VK_SIZE] = raw_test_data
        .vk
        .try_into()
        .expect("Benchmark file should always have the correct vk size");
    let proof = crate::VersionedProof::V3_0(crate::Proof::new(
        test_params.proof_type,
        raw_test_data.proof.to_vec(),
    ));
    let pubs = raw_test_data
        .pubs
        .chunks_exact(crate::PUB_SIZE)
        .map(|c| c.try_into().unwrap())
        .collect();

    TestData { vk, proof, pubs }
}

struct Data {
    vk: &'static [u8],
    proof: &'static [u8],
    pubs: &'static [u8],
}

static DATA_PLAIN: &[Data] = &[
    Data {
        vk: include_bytes!("resources/plain/log_7/vk"),
        proof: include_bytes!("resources/plain/log_7/proof"),
        pubs: include_bytes!("resources/plain/log_7/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_8/vk"),
        proof: include_bytes!("resources/plain/log_8/proof"),
        pubs: include_bytes!("resources/plain/log_8/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_9/vk"),
        proof: include_bytes!("resources/plain/log_9/proof"),
        pubs: include_bytes!("resources/plain/log_9/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_10/vk"),
        proof: include_bytes!("resources/plain/log_10/proof"),
        pubs: include_bytes!("resources/plain/log_10/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_11/vk"),
        proof: include_bytes!("resources/plain/log_11/proof"),
        pubs: include_bytes!("resources/plain/log_11/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_12/vk"),
        proof: include_bytes!("resources/plain/log_12/proof"),
        pubs: include_bytes!("resources/plain/log_12/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_13/vk"),
        proof: include_bytes!("resources/plain/log_13/proof"),
        pubs: include_bytes!("resources/plain/log_13/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_14/vk"),
        proof: include_bytes!("resources/plain/log_14/proof"),
        pubs: include_bytes!("resources/plain/log_14/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_15/vk"),
        proof: include_bytes!("resources/plain/log_15/proof"),
        pubs: include_bytes!("resources/plain/log_15/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_16/vk"),
        proof: include_bytes!("resources/plain/log_16/proof"),
        pubs: include_bytes!("resources/plain/log_16/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_17/vk"),
        proof: include_bytes!("resources/plain/log_17/proof"),
        pubs: include_bytes!("resources/plain/log_17/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_18/vk"),
        proof: include_bytes!("resources/plain/log_18/proof"),
        pubs: include_bytes!("resources/plain/log_18/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_19/vk"),
        proof: include_bytes!("resources/plain/log_19/proof"),
        pubs: include_bytes!("resources/plain/log_19/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_20/vk"),
        proof: include_bytes!("resources/plain/log_20/proof"),
        pubs: include_bytes!("resources/plain/log_20/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_21/vk"),
        proof: include_bytes!("resources/plain/log_21/proof"),
        pubs: include_bytes!("resources/plain/log_21/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_22/vk"),
        proof: include_bytes!("resources/plain/log_22/proof"),
        pubs: include_bytes!("resources/plain/log_22/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_23/vk"),
        proof: include_bytes!("resources/plain/log_23/proof"),
        pubs: include_bytes!("resources/plain/log_23/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_24/vk"),
        proof: include_bytes!("resources/plain/log_24/proof"),
        pubs: include_bytes!("resources/plain/log_24/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_25/vk"),
        proof: include_bytes!("resources/plain/log_25/proof"),
        pubs: include_bytes!("resources/plain/log_25/pubs"),
    },
];

static DATA_ZK: &[Data] = &[
    Data {
        vk: include_bytes!("resources/zk/log_7/vk"),
        proof: include_bytes!("resources/zk/log_7/proof"),
        pubs: include_bytes!("resources/zk/log_7/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_8/vk"),
        proof: include_bytes!("resources/zk/log_8/proof"),
        pubs: include_bytes!("resources/zk/log_8/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_9/vk"),
        proof: include_bytes!("resources/zk/log_9/proof"),
        pubs: include_bytes!("resources/zk/log_9/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_10/vk"),
        proof: include_bytes!("resources/zk/log_10/proof"),
        pubs: include_bytes!("resources/zk/log_10/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_11/vk"),
        proof: include_bytes!("resources/zk/log_11/proof"),
        pubs: include_bytes!("resources/zk/log_11/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_12/vk"),
        proof: include_bytes!("resources/zk/log_12/proof"),
        pubs: include_bytes!("resources/zk/log_12/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_13/vk"),
        proof: include_bytes!("resources/zk/log_13/proof"),
        pubs: include_bytes!("resources/zk/log_13/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_14/vk"),
        proof: include_bytes!("resources/zk/log_14/proof"),
        pubs: include_bytes!("resources/zk/log_14/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_15/vk"),
        proof: include_bytes!("resources/zk/log_15/proof"),
        pubs: include_bytes!("resources/zk/log_15/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_16/vk"),
        proof: include_bytes!("resources/zk/log_16/proof"),
        pubs: include_bytes!("resources/zk/log_16/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_17/vk"),
        proof: include_bytes!("resources/zk/log_17/proof"),
        pubs: include_bytes!("resources/zk/log_17/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_18/vk"),
        proof: include_bytes!("resources/zk/log_18/proof"),
        pubs: include_bytes!("resources/zk/log_18/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_19/vk"),
        proof: include_bytes!("resources/zk/log_19/proof"),
        pubs: include_bytes!("resources/zk/log_19/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_20/vk"),
        proof: include_bytes!("resources/zk/log_20/proof"),
        pubs: include_bytes!("resources/zk/log_20/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_21/vk"),
        proof: include_bytes!("resources/zk/log_21/proof"),
        pubs: include_bytes!("resources/zk/log_21/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_22/vk"),
        proof: include_bytes!("resources/zk/log_22/proof"),
        pubs: include_bytes!("resources/zk/log_22/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_23/vk"),
        proof: include_bytes!("resources/zk/log_23/proof"),
        pubs: include_bytes!("resources/zk/log_23/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_24/vk"),
        proof: include_bytes!("resources/zk/log_24/proof"),
        pubs: include_bytes!("resources/zk/log_24/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_25/vk"),
        proof: include_bytes!("resources/zk/log_25/proof"),
        pubs: include_bytes!("resources/zk/log_25/pubs"),
    },
];
