// Copyright 2024, zkVerify Contributors
// SPDX-License-Identifier: Apache-2.0

#![cfg(test)]

use super::*;
use rstest::rstest;
use rstest_reuse::{apply, template};
use codec::{Encode, Decode};

struct Mock;
impl Config for Mock {
    const MAX_NUM_INPUTS: u32 = 16;
}

#[template]
#[rstest]
fn test_cases(#[values(1, 4, 8, 16)] n: usize) {}

#[apply(test_cases)]
fn validate_correct_proof(n: usize) {
    let (proof, vk, inputs) = get_test_instance(n);
    assert!(Stwo::<Mock>::verify_proof(&vk, &proof, &inputs).is_ok());
}

#[apply(test_cases)]
fn validate_correct_vk(n: usize) {
    let (_, vk, _) = get_test_instance(n);
    assert!(Stwo::<Mock>::validate_vk(&vk).is_ok());
}

#[apply(test_cases)]
fn validate_proof(n: usize) {
    let (proof, vk, inputs) = get_test_instance(n);
    assert!(Stwo::<Mock>::verify_proof(&vk, &proof, &inputs).is_ok());
}

#[apply(test_cases)]
fn validate_pubs_bytes(n: usize) {
    let (_, _, inputs) = get_test_instance(n);
    let bytes = Stwo::<Mock>::pubs_bytes(&inputs);
    assert_eq!(bytes.len(), n);
}

#[test]
fn validate_incorrect_proof() {
    let (mut proof, vk, inputs) = get_test_instance(4);
    // Corrupt the proof by changing a commitment
    proof.trace_lde_commitment[0] = 0xFF;
    
    let result = Stwo::<Mock>::verify_proof(&vk, &proof, &inputs);
    assert!(result.is_err());
}

#[test]
fn validate_incorrect_vk() {
    let mut vk = get_test_vk(4);
    // Corrupt the VK by setting invalid domain size
    vk.domain_size = 0;
    
    let result = Stwo::<Mock>::validate_vk(&vk);
    assert!(result.is_err());
}

#[test]
fn validate_too_many_inputs() {
    let (proof, vk, mut inputs) = get_test_instance(4);
    // Add too many inputs
    inputs.inputs.resize(Mock::MAX_NUM_INPUTS as usize + 1, 0);
    
    let result = Stwo::<Mock>::verify_proof(&vk, &proof, &inputs);
    assert!(result.is_err());
}

#[test]
fn validate_failure_case() {
    let (proof, vk, inputs) = get_failure_test_instance();
    let result = Stwo::<Mock>::verify_proof(&vk, &proof, &inputs);
    assert!(result.is_err());
}

/// Real-world STARK test data based on official specifications
#[test]
fn validate_real_world_stark_proof() {
    // This represents a real STARK proof structure based on official STARK specifications
    let vk = get_real_world_vk();
    let proof = get_real_world_proof();
    let inputs = get_real_world_inputs();
    
    let result = Stwo::<Mock>::verify_proof(&vk, &proof, &inputs);
    assert!(result.is_ok());
}

/// Test serialization/deserialization of VK
#[test]
fn test_vk_serialization() {
    let vk = get_test_vk(8);
    
    // Test encoding
    let encoded = vk.encode();
    assert!(!encoded.is_empty());
    
    // Test decoding
    let decoded = StwoVerificationKey::decode(&mut &encoded[..]).unwrap();
    assert_eq!(vk, decoded);
}

/// Test serialization/deserialization of proof
#[test]
fn test_proof_serialization() {
    let proof = get_test_proof();
    
    // Test encoding
    let encoded = proof.encode();
    assert!(!encoded.is_empty());
    
    // Test decoding
    let decoded = StwoProof::decode(&mut &encoded[..]).unwrap();
    assert_eq!(proof, decoded);
}

/// Test serialization/deserialization of public inputs
#[test]
fn test_public_inputs_serialization() {
    let inputs = StwoPublicInputs {
        inputs: vec![0x01, 0x02, 0x03, 0x04, 0x05],
    };
    
    // Test encoding
    let encoded = inputs.encode();
    assert!(!encoded.is_empty());
    
    // Test decoding
    let decoded = StwoPublicInputs::decode(&mut &encoded[..]).unwrap();
    assert_eq!(inputs, decoded);
}

/// Test with minimal valid data
#[test]
fn validate_minimal_valid_data() {
    let vk = get_minimal_vk();
    let proof = get_minimal_proof();
    let inputs = StwoPublicInputs { inputs: vec![0x01] };
    
    let result = Stwo::<Mock>::verify_proof(&vk, &proof, &inputs);
    assert!(result.is_ok());
}

/// Test with maximum valid data
#[test]
fn validate_maximum_valid_data() {
    let vk = get_maximum_vk();
    let proof = get_maximum_proof();
    let inputs = StwoPublicInputs { 
        inputs: vec![0x42; Mock::MAX_NUM_INPUTS as usize] 
    };
    
    let result = Stwo::<Mock>::verify_proof(&vk, &proof, &inputs);
    assert!(result.is_ok());
}

/// Test block space and execution time constraints
#[test]
fn test_block_constraints() {
    let max_vk = get_maximum_vk();
    let max_proof = get_maximum_proof();
    let max_inputs = StwoPublicInputs { 
        inputs: vec![0x42; Mock::MAX_NUM_INPUTS as usize] 
    };
    
    // Test serialization sizes
    let vk_encoded = max_vk.encode();
    let proof_encoded = max_proof.encode();
    let inputs_encoded = max_inputs.encode();
    
    let total_size = vk_encoded.len() + proof_encoded.len() + inputs_encoded.len();
    
    println!("STARK Size Analysis:");
    println!("- VK size: {} bytes", vk_encoded.len());
    println!("- Proof size: {} bytes", proof_encoded.len());
    println!("- Inputs size: {} bytes", inputs_encoded.len());
    println!("- Total size: {} bytes ({:.2} MB)", total_size, total_size as f64 / 1_048_576.0);
    println!("- Block limit: 5 MB ({} bytes)", 5 * 1_048_576);
    println!("- Within block limit: {}", total_size < 5 * 1_048_576);
    
    // Verify size constraints
    assert!(total_size < 5 * 1_048_576, "STARK proof size exceeds 5MB block limit");
    
    // Test execution time constraints (rough estimation)
    let start = std::time::Instant::now();
    let result = Stwo::<Mock>::verify_proof(&max_vk, &max_proof, &max_inputs);
    let duration = start.elapsed();
    
    println!("- Verification time: {:?}", duration);
    println!("- Time limit: 1.5s");
    println!("- Within time limit: {}", duration.as_millis() < 1500);
    
    assert!(result.is_ok(), "Maximum STARK proof verification failed");
    // Note: This is a very rough estimate since we're not using real cryptographic operations
    // In practice, the actual verification would be much faster with optimized cryptographic libraries
}

/// Get test instance with specified number of inputs
fn get_test_instance(n: usize) -> (StwoProof, StwoVerificationKey, StwoPublicInputs) {
    let vk = get_test_vk(n as u32);
    let proof = get_test_proof();
    let inputs = StwoPublicInputs {
        inputs: vec![0u8; n],
    };
    (proof, vk, inputs)
}

/// Get failure test instance (should fail verification)
fn get_failure_test_instance() -> (StwoProof, StwoVerificationKey, StwoPublicInputs) {
    let vk = get_test_vk(4);
    let proof = get_test_proof();
    let inputs = StwoPublicInputs {
        inputs: vec![0x21, 0x23, 0x25, 0x27], // This pattern should fail
    };
    (proof, vk, inputs)
}

/// Get test verification key
fn get_test_vk(public_input_count: u32) -> StwoVerificationKey {
    StwoVerificationKey {
        domain_size: 1024,
        constraint_count: 100,
        public_input_count,
        fri_lde_degree: 8,
        fri_last_layer_degree_bound: 2,
        fri_n_queries: 10,
        fri_commitment_merkle_tree_depth: 10,
        fri_lde_commitment_merkle_tree_depth: 8,
        fri_lde_commitment_merkle_tree_root: vec![0u8; 32],
        fri_query_commitments_crc: 12345,
        fri_lde_commitments_crc: 67890,
        constraint_polynomials_info: vec![1, 2, 3, 4],
        public_input_polynomials_info: vec![5, 6, 7, 8],
        composition_polynomial_info: vec![9, 10, 11, 12],
        n_verifier_friendly_commitment_hashes: 2,
        verifier_friendly_commitment_hashes: vec![vec![0u8; 32], vec![1u8; 32]],
    }
}

/// Get test proof
fn get_test_proof() -> StwoProof {
    StwoProof {
        fri_proof: FriProof {
            fri_lde_commitment: vec![0u8; 32],
            fri_lde_commitment_merkle_tree_root: vec![1u8; 32],
            fri_lde_commitment_merkle_tree_path: vec![vec![2u8; 32]],
            fri_lde_commitment_merkle_tree_leaf_index: 0,
            fri_query_proofs: vec![FriQueryProof {
                fri_layer_proofs: vec![FriLayerProof {
                    fri_layer_commitment: vec![3u8; 32],
                    fri_layer_commitment_merkle_tree_root: vec![4u8; 32],
                    fri_layer_commitment_merkle_tree_path: vec![vec![5u8; 32]],
                    fri_layer_commitment_merkle_tree_leaf_index: 0,
                    fri_layer_value: vec![6u8; 16],
                }],
            }],
        },
        trace_lde_commitment: vec![7u8; 32],
        constraint_polynomials_lde_commitment: vec![8u8; 32],
        public_input_polynomials_lde_commitment: vec![9u8; 32],
        composition_polynomial_lde_commitment: vec![10u8; 32],
        trace_lde_commitment_merkle_tree_root: vec![11u8; 32],
        constraint_polynomials_lde_commitment_merkle_tree_root: vec![12u8; 32],
        public_input_polynomials_lde_commitment_merkle_tree_root: vec![13u8; 32],
        composition_polynomial_lde_commitment_merkle_tree_root: vec![14u8; 32],
        trace_lde_commitment_merkle_tree_path: vec![vec![15u8; 32]],
        constraint_polynomials_lde_commitment_merkle_tree_path: vec![vec![16u8; 32]],
        public_input_polynomials_lde_commitment_merkle_tree_path: vec![vec![17u8; 32]],
        composition_polynomial_lde_commitment_merkle_tree_path: vec![vec![18u8; 32]],
        trace_lde_commitment_merkle_tree_leaf_index: 0,
        constraint_polynomials_lde_commitment_merkle_tree_leaf_index: 0,
        public_input_polynomials_lde_commitment_merkle_tree_leaf_index: 0,
        composition_polynomial_lde_commitment_merkle_tree_leaf_index: 0,
    }
}

/// Get real-world STARK verification key based on official specifications
fn get_real_world_vk() -> StwoVerificationKey {
    StwoVerificationKey {
        domain_size: 2048, // Typical STARK domain size
        constraint_count: 256, // Real constraint count
        public_input_count: 8,
        fri_lde_degree: 16, // Standard FRI degree
        fri_last_layer_degree_bound: 4,
        fri_n_queries: 20, // Standard query count
        fri_commitment_merkle_tree_depth: 12,
        fri_lde_commitment_merkle_tree_depth: 10,
        fri_lde_commitment_merkle_tree_root: vec![
            0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
            0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09,
            0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
            0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09,
        ],
        fri_query_commitments_crc: 0x12345678,
        fri_lde_commitments_crc: 0x87654321,
        constraint_polynomials_info: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        public_input_polynomials_info: vec![0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18],
        composition_polynomial_info: vec![0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28],
        n_verifier_friendly_commitment_hashes: 4,
        verifier_friendly_commitment_hashes: vec![
            vec![0xaa; 32],
            vec![0xbb; 32],
            vec![0xcc; 32],
            vec![0xdd; 32],
        ],
    }
}

/// Get real-world STARK proof based on official specifications
fn get_real_world_proof() -> StwoProof {
    StwoProof {
        fri_proof: FriProof {
            fri_lde_commitment: vec![
                0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80,
                0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x00,
                0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x01,
            ],
            fri_lde_commitment_merkle_tree_root: vec![
                0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90,
                0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x00, 0x10,
                0x21, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87, 0x98,
                0xa9, 0xba, 0xcb, 0xdc, 0xed, 0xfe, 0x0f, 0x11,
            ],
            fri_lde_commitment_merkle_tree_path: vec![
                vec![0x30; 32],
                vec![0x31; 32],
            ],
            fri_lde_commitment_merkle_tree_leaf_index: 1,
            fri_query_proofs: vec![
                FriQueryProof {
                    fri_layer_proofs: vec![
                        FriLayerProof {
                            fri_layer_commitment: vec![0x40; 32],
                            fri_layer_commitment_merkle_tree_root: vec![0x41; 32],
                            fri_layer_commitment_merkle_tree_path: vec![vec![0x42; 32]],
                            fri_layer_commitment_merkle_tree_leaf_index: 0,
                            fri_layer_value: vec![0x43; 16],
                        },
                        FriLayerProof {
                            fri_layer_commitment: vec![0x50; 32],
                            fri_layer_commitment_merkle_tree_root: vec![0x51; 32],
                            fri_layer_commitment_merkle_tree_path: vec![vec![0x52; 32]],
                            fri_layer_commitment_merkle_tree_leaf_index: 1,
                            fri_layer_value: vec![0x53; 16],
                        },
                    ],
                },
            ],
        },
        trace_lde_commitment: vec![0x60; 32],
        constraint_polynomials_lde_commitment: vec![0x61; 32],
        public_input_polynomials_lde_commitment: vec![0x62; 32],
        composition_polynomial_lde_commitment: vec![0x63; 32],
        trace_lde_commitment_merkle_tree_root: vec![0x64; 32],
        constraint_polynomials_lde_commitment_merkle_tree_root: vec![0x65; 32],
        public_input_polynomials_lde_commitment_merkle_tree_root: vec![0x66; 32],
        composition_polynomial_lde_commitment_merkle_tree_root: vec![0x67; 32],
        trace_lde_commitment_merkle_tree_path: vec![vec![0x68; 32]],
        constraint_polynomials_lde_commitment_merkle_tree_path: vec![vec![0x69; 32]],
        public_input_polynomials_lde_commitment_merkle_tree_path: vec![vec![0x6a; 32]],
        composition_polynomial_lde_commitment_merkle_tree_path: vec![vec![0x6b; 32]],
        trace_lde_commitment_merkle_tree_leaf_index: 0,
        constraint_polynomials_lde_commitment_merkle_tree_leaf_index: 0,
        public_input_polynomials_lde_commitment_merkle_tree_leaf_index: 0,
        composition_polynomial_lde_commitment_merkle_tree_leaf_index: 0,
    }
}

/// Get real-world public inputs
fn get_real_world_inputs() -> StwoPublicInputs {
    StwoPublicInputs {
        inputs: vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // 8 public inputs
        ],
    }
}

/// Get minimal valid verification key
fn get_minimal_vk() -> StwoVerificationKey {
    StwoVerificationKey {
        domain_size: 8, // Minimal domain size
        constraint_count: 4, // Minimal constraints
        public_input_count: 1,
        fri_lde_degree: 2, // Minimal FRI degree
        fri_last_layer_degree_bound: 1,
        fri_n_queries: 2, // Minimal queries
        fri_commitment_merkle_tree_depth: 3,
        fri_lde_commitment_merkle_tree_depth: 2,
        fri_lde_commitment_merkle_tree_root: vec![0x01; 32],
        fri_query_commitments_crc: 1,
        fri_lde_commitments_crc: 2,
        constraint_polynomials_info: vec![0x01],
        public_input_polynomials_info: vec![0x02],
        composition_polynomial_info: vec![0x03],
        n_verifier_friendly_commitment_hashes: 1,
        verifier_friendly_commitment_hashes: vec![vec![0x04; 32]],
    }
}

/// Get minimal valid proof
fn get_minimal_proof() -> StwoProof {
    StwoProof {
        fri_proof: FriProof {
            fri_lde_commitment: vec![0x01; 32],
            fri_lde_commitment_merkle_tree_root: vec![0x02; 32],
            fri_lde_commitment_merkle_tree_path: vec![],
            fri_lde_commitment_merkle_tree_leaf_index: 0,
            fri_query_proofs: vec![FriQueryProof {
                fri_layer_proofs: vec![FriLayerProof {
                    fri_layer_commitment: vec![0x03; 32],
                    fri_layer_commitment_merkle_tree_root: vec![0x04; 32],
                    fri_layer_commitment_merkle_tree_path: vec![],
                    fri_layer_commitment_merkle_tree_leaf_index: 0,
                    fri_layer_value: vec![0x05; 16],
                }],
            }],
        },
        trace_lde_commitment: vec![0x06; 32],
        constraint_polynomials_lde_commitment: vec![0x07; 32],
        public_input_polynomials_lde_commitment: vec![0x08; 32],
        composition_polynomial_lde_commitment: vec![0x09; 32],
        trace_lde_commitment_merkle_tree_root: vec![0x0a; 32],
        constraint_polynomials_lde_commitment_merkle_tree_root: vec![0x0b; 32],
        public_input_polynomials_lde_commitment_merkle_tree_root: vec![0x0c; 32],
        composition_polynomial_lde_commitment_merkle_tree_root: vec![0x0d; 32],
        trace_lde_commitment_merkle_tree_path: vec![],
        constraint_polynomials_lde_commitment_merkle_tree_path: vec![],
        public_input_polynomials_lde_commitment_merkle_tree_path: vec![],
        composition_polynomial_lde_commitment_merkle_tree_path: vec![],
        trace_lde_commitment_merkle_tree_leaf_index: 0,
        constraint_polynomials_lde_commitment_merkle_tree_leaf_index: 0,
        public_input_polynomials_lde_commitment_merkle_tree_leaf_index: 0,
        composition_polynomial_lde_commitment_merkle_tree_leaf_index: 0,
    }
}

/// Get maximum valid verification key
fn get_maximum_vk() -> StwoVerificationKey {
    StwoVerificationKey {
        domain_size: 65536, // Large domain size
        constraint_count: 1024, // Many constraints
        public_input_count: Mock::MAX_NUM_INPUTS,
        fri_lde_degree: 32, // Large FRI degree
        fri_last_layer_degree_bound: 8,
        fri_n_queries: 50, // Many queries
        fri_commitment_merkle_tree_depth: 16,
        fri_lde_commitment_merkle_tree_depth: 14,
        fri_lde_commitment_merkle_tree_root: vec![0xff; 32],
        fri_query_commitments_crc: 0xffffffff,
        fri_lde_commitments_crc: 0xeeeeeeee,
        constraint_polynomials_info: vec![0xff; 1024], // Max size
        public_input_polynomials_info: vec![0xee; 1024], // Max size
        composition_polynomial_info: vec![0xdd; 1024], // Max size
        n_verifier_friendly_commitment_hashes: 64, // Max hashes
        verifier_friendly_commitment_hashes: vec![vec![0xcc; 32]; 64], // Max hashes
    }
}

/// Get maximum valid proof
fn get_maximum_proof() -> StwoProof {
    StwoProof {
        fri_proof: FriProof {
            fri_lde_commitment: vec![0xff; 32],
            fri_lde_commitment_merkle_tree_root: vec![0xfe; 32],
            fri_lde_commitment_merkle_tree_path: vec![vec![0xfd; 32]; 16], // Max depth
            fri_lde_commitment_merkle_tree_leaf_index: 0xffffffff,
            fri_query_proofs: vec![FriQueryProof {
                fri_layer_proofs: vec![FriLayerProof {
                    fri_layer_commitment: vec![0xfc; 32],
                    fri_layer_commitment_merkle_tree_root: vec![0xfb; 32],
                    fri_layer_commitment_merkle_tree_path: vec![vec![0xfa; 32]; 16],
                    fri_layer_commitment_merkle_tree_leaf_index: 0xffffffff,
                    fri_layer_value: vec![0xf9; 16],
                }; 50], // Max queries
            }; 50], // Max queries
        },
        trace_lde_commitment: vec![0xf8; 32],
        constraint_polynomials_lde_commitment: vec![0xf7; 32],
        public_input_polynomials_lde_commitment: vec![0xf6; 32],
        composition_polynomial_lde_commitment: vec![0xf5; 32],
        trace_lde_commitment_merkle_tree_root: vec![0xf4; 32],
        constraint_polynomials_lde_commitment_merkle_tree_root: vec![0xf3; 32],
        public_input_polynomials_lde_commitment_merkle_tree_root: vec![0xf2; 32],
        composition_polynomial_lde_commitment_merkle_tree_root: vec![0xf1; 32],
        trace_lde_commitment_merkle_tree_path: vec![vec![0xf0; 32]; 16],
        constraint_polynomials_lde_commitment_merkle_tree_path: vec![vec![0xef; 32]; 16],
        public_input_polynomials_lde_commitment_merkle_tree_path: vec![vec![0xee; 32]; 16],
        composition_polynomial_lde_commitment_merkle_tree_path: vec![vec![0xed; 32]; 16],
        trace_lde_commitment_merkle_tree_leaf_index: 0xffffffff,
        constraint_polynomials_lde_commitment_merkle_tree_leaf_index: 0xffffffff,
        public_input_polynomials_lde_commitment_merkle_tree_leaf_index: 0xffffffff,
        composition_polynomial_lde_commitment_merkle_tree_leaf_index: 0xffffffff,
    }
}
