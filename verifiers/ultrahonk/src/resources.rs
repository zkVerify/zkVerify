// Copyright 2025-2026, Horizen Labs, Inc.

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

use crate::{
    ProofType, ProtocolVersion, VersionedProof, VersionedVk, MIN_BENCHMARKED_LOG_CIRCUIT_SIZE,
    VK_SIZE_V0_84, VK_SIZE_V3_0, VK_SIZE_V5_0,
};

/// Largest log_circuit_size with a fixture. TODO: restore to `MAX_BENCHMARKED_LOG_CIRCUIT_SIZE`
/// once the log_n = 25 fixtures are generated (they exhaust the memory of the machine used so
/// far); until then the benchmarks fit the weight on [7, 24] and extrapolate it to 25.
pub const MAX_FIXTURE_LOG_CIRCUIT_SIZE: u64 = 24;

pub struct TestParams {
    log_circuit_size: Option<u64>, // only V5_0 is benchmarked based on log_n
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

    // For the deprecated versions, of which a single sample is kept to test their rejection.
    pub fn new_deprecated(proof_type: ProofType, protocol_version: ProtocolVersion) -> Self {
        Self {
            log_circuit_size: None,
            proof_type,
            protocol_version,
        }
    }
}

pub struct TestData {
    pub versioned_vk: crate::VersionedVk,
    pub versioned_proof: crate::VersionedProof,
    pub pubs: crate::Pubs,
}

fn pubs(raw_pubs: &[u8]) -> crate::Pubs {
    raw_pubs.as_chunks::<{ crate::PUB_SIZE }>().0.to_vec()
}

pub fn get_parameterized_test_data(test_params: TestParams) -> Result<TestData, &'static str> {
    let proof_type = test_params.proof_type;
    match test_params.protocol_version {
        ProtocolVersion::V5_0 => {
            let log_circuit_size = test_params
                .log_circuit_size
                .ok_or("log_circuit_size must be specified for ProtocolVersion::V5_0")?;
            if !(MIN_BENCHMARKED_LOG_CIRCUIT_SIZE..=MAX_FIXTURE_LOG_CIRCUIT_SIZE)
                .contains(&log_circuit_size)
            {
                return Err("no fixture for the requested log_circuit_size");
            }

            let data = match proof_type {
                ProofType::ZK => DATA_ZK,
                ProofType::Plain => DATA_PLAIN,
            };
            let raw_test_data =
                &data[(log_circuit_size - MIN_BENCHMARKED_LOG_CIRCUIT_SIZE) as usize];

            let raw_vk: [u8; VK_SIZE_V5_0] = raw_test_data
                .vk
                .try_into()
                .expect("Benchmark file should always have the correct vk size");
            let proof = crate::Proof::new(proof_type, raw_test_data.proof.to_vec());

            Ok(TestData {
                versioned_vk: VersionedVk::V5_0(raw_vk),
                versioned_proof: VersionedProof::V5_0(proof),
                pubs: pubs(raw_test_data.pubs),
            })
        }
        ProtocolVersion::V3_0 => {
            let raw_test_data = match proof_type {
                ProofType::ZK => &DATA_V3_0_ZK,
                ProofType::Plain => &DATA_V3_0_PLAIN,
            };
            let raw_vk: [u8; VK_SIZE_V3_0] = raw_test_data
                .vk
                .try_into()
                .expect("Test file should always have the correct vk size");
            let proof = crate::Proof::new(proof_type, raw_test_data.proof.to_vec());

            Ok(TestData {
                versioned_vk: VersionedVk::V3_0(raw_vk),
                versioned_proof: VersionedProof::V3_0(proof),
                pubs: pubs(raw_test_data.pubs),
            })
        }
        ProtocolVersion::V0_84 | ProtocolVersion::Legacy => {
            let raw_test_data = match proof_type {
                ProofType::ZK => &DATA_V0_84_ZK,
                ProofType::Plain => &DATA_V0_84_PLAIN,
            };
            let raw_vk: [u8; VK_SIZE_V0_84] = raw_test_data
                .vk
                .try_into()
                .expect("Test file should always have the correct vk size");
            let proof = crate::Proof::new(proof_type, raw_test_data.proof.to_vec());
            let (versioned_vk, versioned_proof) = match test_params.protocol_version {
                ProtocolVersion::Legacy => {
                    (VersionedVk::Legacy(raw_vk), VersionedProof::Legacy(proof))
                }
                _ => (VersionedVk::V0_84(raw_vk), VersionedProof::V0_84(proof)),
            };

            Ok(TestData {
                versioned_vk,
                versioned_proof,
                pubs: pubs(raw_test_data.pubs),
            })
        }
    }
}

struct Data {
    vk: &'static [u8],
    proof: &'static [u8],
    pubs: &'static [u8],
}

// Samples of the deprecated versions, generated by bb 0.84 and bb 3.0.
static DATA_V0_84_ZK: Data = Data {
    vk: include_bytes!("resources/v0_84/zk/vk"),
    proof: include_bytes!("resources/v0_84/zk/proof"),
    pubs: include_bytes!("resources/v0_84/zk/pubs"),
};

static DATA_V0_84_PLAIN: Data = Data {
    vk: include_bytes!("resources/v0_84/plain/vk"),
    proof: include_bytes!("resources/v0_84/plain/proof"),
    pubs: include_bytes!("resources/v0_84/plain/pubs"),
};

static DATA_V3_0_ZK: Data = Data {
    vk: include_bytes!("resources/v3_0/zk/vk"),
    proof: include_bytes!("resources/v3_0/zk/proof"),
    pubs: include_bytes!("resources/v3_0/zk/pubs"),
};

static DATA_V3_0_PLAIN: Data = Data {
    vk: include_bytes!("resources/v3_0/plain/vk"),
    proof: include_bytes!("resources/v3_0/plain/proof"),
    pubs: include_bytes!("resources/v3_0/plain/pubs"),
};

// One fixture per log_circuit_size in [MIN_BENCHMARKED_LOG_CIRCUIT_SIZE, MAX_FIXTURE_LOG_CIRCUIT_SIZE],
// generated by ultrahonk_verifier's `scripts/generate_pallet_fixtures.sh` with bb v5.0.0.
static DATA_PLAIN: &[Data] = &[
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_7/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_7/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_7/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_8/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_8/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_8/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_9/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_9/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_9/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_10/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_10/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_10/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_11/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_11/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_11/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_12/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_12/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_12/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_13/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_13/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_13/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_14/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_14/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_14/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_15/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_15/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_15/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_16/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_16/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_16/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_17/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_17/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_17/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_18/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_18/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_18/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_19/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_19/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_19/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_20/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_20/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_20/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_21/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_21/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_21/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_22/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_22/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_22/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_23/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_23/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_23/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/plain/log_24/vk"),
        proof: include_bytes!("resources/v5_0/plain/log_24/proof"),
        pubs: include_bytes!("resources/v5_0/plain/log_24/pubs"),
    },
];

static DATA_ZK: &[Data] = &[
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_7/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_7/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_7/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_8/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_8/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_8/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_9/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_9/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_9/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_10/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_10/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_10/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_11/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_11/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_11/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_12/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_12/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_12/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_13/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_13/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_13/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_14/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_14/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_14/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_15/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_15/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_15/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_16/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_16/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_16/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_17/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_17/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_17/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_18/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_18/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_18/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_19/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_19/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_19/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_20/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_20/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_20/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_21/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_21/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_21/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_22/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_22/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_22/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_23/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_23/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_23/pubs"),
    },
    Data {
        vk: include_bytes!("resources/v5_0/zk/log_24/vk"),
        proof: include_bytes!("resources/v5_0/zk/log_24/proof"),
        pubs: include_bytes!("resources/v5_0/zk/log_24/pubs"),
    },
];
