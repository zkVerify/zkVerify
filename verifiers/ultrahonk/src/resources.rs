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

use crate::{ProofType, ProtocolVersion, VersionedProof, VersionedVk};

// Minimum allowed value for the logarithm of the polynomial evaluation domain size.
const MIN_BENCHMARKED_LOG_CIRCUIT_SIZE: u64 = 7;

// Struct containing the parameters pointing to the exact benchmark data that should be used.
pub struct TestParams {
    log_circuit_size: Option<u64>, // optional since for 0.84.0 we do not benchmark based on log_n
    proof_type: ProofType,
    protocol_version: ProtocolVersion,
}

impl TestParams {
    pub fn new(
        log_circuit_size: u64,
        proof_type: ProofType,
        protocol_version: ProtocolVersion,
    ) -> Self {
        Self {
            log_circuit_size: Some(log_circuit_size),
            proof_type,
            protocol_version,
        }
    }

    // For handling V0_84 where there is no log_circuit_size
    pub fn new_v0_84(proof_type: ProofType) -> Self {
        Self {
            log_circuit_size: None,
            proof_type,
            protocol_version: ProtocolVersion::V0_84,
        }
    }
}

pub struct TestData {
    pub versioned_vk: crate::VersionedVk,
    pub versioned_proof: crate::VersionedProof,
    pub pubs: crate::Pubs,
}

pub fn get_parameterized_test_data(test_params: TestParams) -> Result<TestData, &'static str> {
    match test_params.protocol_version {
        ProtocolVersion::V3_0 => {
            if test_params.log_circuit_size.is_none() {
                return Err("log_circuit_size must be specified for ProtocolVersion::V3_0");
            }

            let data = match test_params.proof_type {
                ProofType::ZK => DATA_ZK,
                ProofType::Plain => DATA_PLAIN,
            };
            let raw_test_data_idx = (test_params
                .log_circuit_size
                .expect("Should never fail at this point")
                - MIN_BENCHMARKED_LOG_CIRCUIT_SIZE) as usize;
            let raw_test_data = &data[raw_test_data_idx];

            let raw_vk: [u8; ultrahonk_no_std_v3_0::VK_SIZE] = raw_test_data
                .vk
                .try_into()
                .expect("Benchmark file should always have the correct vk size");
            let versioned_vk = VersionedVk::V3_0(raw_vk);
            let proof = crate::Proof::new(test_params.proof_type, raw_test_data.proof.to_vec());
            let versioned_proof = crate::VersionedProof::V3_0(proof);
            let pubs = raw_test_data
                .pubs
                .chunks_exact(crate::PUB_SIZE)
                .map(|c| c.try_into().unwrap())
                .collect();

            Ok(TestData {
                versioned_vk,
                versioned_proof,
                pubs,
            })
        }
        ProtocolVersion::V0_84 => {
            let raw_test_data = match test_params.proof_type {
                ProofType::ZK => &DATA_V0_84_ZK,
                ProofType::Plain => &DATA_V0_84_PLAIN,
            };

            let raw_vk: [u8; ultrahonk_no_std_v0_84::VK_SIZE] = raw_test_data
                .vk
                .try_into()
                .expect("Benchmark file should always have the correct vk size");
            let versioned_vk = VersionedVk::V0_84(raw_vk);
            let proof = crate::Proof::new(test_params.proof_type, raw_test_data.proof.to_vec());
            let versioned_proof = crate::VersionedProof::V0_84(proof);
            let pubs = raw_test_data
                .pubs
                .chunks_exact(crate::PUB_SIZE)
                .map(|c| c.try_into().unwrap())
                .collect();

            Ok(TestData {
                versioned_vk,
                versioned_proof,
                pubs,
            })
        }
    }
}

struct Data {
    vk: &'static [u8],
    proof: &'static [u8],
    pubs: &'static [u8],
}

static DATA_V0_84_ZK: Data = Data {
    vk: include_bytes!("resources/v0_84/zk/32/vk"),
    proof: include_bytes!("resources/v0_84/zk/32/proof"),
    pubs: include_bytes!("resources/v0_84/zk/32/pubs"),
};

static DATA_V0_84_PLAIN: Data = Data {
    vk: include_bytes!("resources/v0_84/plain/32/vk"),
    proof: include_bytes!("resources/v0_84/plain/32/proof"),
    pubs: include_bytes!("resources/v0_84/plain/32/pubs"),
};

static DATA_PLAIN: &[Data] = &[
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_7/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_7/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_7/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_8/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_8/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_8/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_9/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_9/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_9/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_10/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_10/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_10/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_11/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_11/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_11/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_12/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_12/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_12/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_13/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_13/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_13/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_14/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_14/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_14/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_15/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_15/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_15/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_16/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_16/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_16/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_17/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_17/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_17/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_18/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_18/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_18/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_19/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_19/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_19/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_20/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_20/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_20/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_21/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_21/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_21/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_22/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_22/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_22/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_23/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_23/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_23/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_24/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_24/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_24/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/plain/log_25/vk"),
        proof: include_bytes!("resources/v3_0/plain/log_25/proof"),
        pubs: include_bytes!("resources/v3_0/plain/log_25/pubs"),
    },
];

static DATA_ZK: &[Data] = &[
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_7/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_7/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_7/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_8/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_8/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_8/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_9/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_9/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_9/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_10/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_10/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_10/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_11/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_11/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_11/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_12/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_12/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_12/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_13/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_13/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_13/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_14/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_14/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_14/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_15/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_15/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_15/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_16/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_16/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_16/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_17/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_17/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_17/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_18/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_18/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_18/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_19/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_19/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_19/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_20/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_20/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_20/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_21/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_21/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_21/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_22/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_22/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_22/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_23/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_23/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_23/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_24/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_24/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_24/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v3_0/zk/log_25/vk"),
        proof: include_bytes!("resources/v3_0/zk/log_25/proof"),
        pubs: include_bytes!("resources/v3_0/zk/log_25/pubs"),
    },
];
