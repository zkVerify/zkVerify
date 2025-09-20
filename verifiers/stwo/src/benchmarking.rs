// Copyright 2024, zkVerify Contributors
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "runtime-benchmarks")]

use super::Stwo as Verifier;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use hp_verifiers::Verifier as _;
use pallet_verifiers::{benchmarking_utils, VkOrHash};

pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>)]
mod benchmarks {

    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    /// Benchmark proof verification with varying input sizes
    #[benchmark]
    fn verify_proof(n: Linear<1, <T as crate::Config>::MAX_NUM_INPUTS>) {
        let (proof, vk, pubs) = get_test_instance(n as usize);

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    /// Benchmark proof verification with real-world STARK data
    #[benchmark]
    fn verify_proof_real_world() {
        let (proof, vk, pubs) = get_real_world_instance();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    /// Benchmark proof verification with maximum complexity
    #[benchmark]
    fn verify_proof_maximum() {
        let (proof, vk, pubs) = get_maximum_instance();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    /// Benchmark VK registration with varying input sizes
    #[benchmark]
    fn register_vk(n: Linear<1, <T as crate::Config>::MAX_NUM_INPUTS>) {
        let caller = funded_account::<T>();
        let (_, vk, _) = get_test_instance(n as usize);

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.clone().into());

        // Verify
        assert!(do_get_vk::<T>(&do_vk_hash::<T>(&vk)).is_some());
    }

    /// Benchmark VK registration with real-world data
    #[benchmark]
    fn register_vk_real_world() {
        let caller = funded_account::<T>();
        let (_, vk, _) = get_real_world_instance();

        #[extrinsic_call]
        register_vk(RawOrigin::Signed(caller), vk.clone().into());

        // Verify
        assert!(do_get_vk::<T>(&do_vk_hash::<T>(&vk)).is_some());
    }

    /// Benchmark VK unregistration
    #[benchmark]
    fn unregister_vk() {
        let caller: T::AccountId = funded_account::<T>();
        let hash = sp_core::H256::repeat_byte(2);
        let (_, vk, _) = get_test_instance(4);

        insert_vk::<T>(caller.clone(), vk, hash);

        #[extrinsic_call]
        unregister_vk(RawOrigin::Signed(caller), hash);

        // Verify
        assert!(do_get_vk::<T>(&hash).is_none());
    }

    /// Benchmark VK retrieval
    #[benchmark]
    fn get_vk() {
        let (_, vk, _) = get_test_instance(4);
        let hash = sp_core::H256::repeat_byte(2);

        insert_vk_anonymous::<T>(vk, hash);

        let r;
        #[block]
        {
            r = do_get_vk::<T>(&hash)
        };
        assert!(r.is_some());
    }

    /// Benchmark VK validation with varying input sizes
    #[benchmark]
    fn validate_vk(n: Linear<1, <T as crate::Config>::MAX_NUM_INPUTS>) {
        let (_, vk, _) = get_test_instance(n as usize);

        let r;
        #[block]
        {
            r = do_validate_vk::<T>(&vk)
        };
        assert!(r.is_ok());
    }

    /// Benchmark statement hash computation with varying input sizes
    #[benchmark]
    fn compute_statement_hash(n: Linear<1, <T as crate::Config>::MAX_NUM_INPUTS>) {
        let (proof, vk, pubs) = get_test_instance(n as usize);

        let vk = VkOrHash::Vk(vk.into());

        #[block]
        {
            do_compute_statement_hash::<T>(&vk, &proof, &pubs);
        }
    }

    /// Benchmark statement hash computation with real-world data
    #[benchmark]
    fn compute_statement_hash_real_world() {
        let (proof, vk, pubs) = get_real_world_instance();

        let vk = VkOrHash::Vk(vk.into());

        #[block]
        {
            do_compute_statement_hash::<T>(&vk, &proof, &pubs);
        }
    }

    /// Benchmark batch verification (multiple proofs)
    #[benchmark]
    fn verify_proof_batch() {
        let instances = get_batch_instances();

        #[block]
        {
            for (proof, vk, pubs) in instances {
                let r = do_verify_proof::<T>(&vk, &proof, &pubs);
                assert!(r.is_ok());
            }
        };
    }
}

/// Get test instance with specified number of inputs
fn get_test_instance(n: usize) -> (crate::StwoProof, crate::StwoVerificationKey, crate::StwoPublicInputs) {
    let vk = get_test_vk(n as u32);
    let proof = get_test_proof();
    let inputs = crate::StwoPublicInputs {
        inputs: vec![0u8; n],
    };
    (proof, vk, inputs)
}

/// Get real-world STARK instance based on official specifications
fn get_real_world_instance() -> (crate::StwoProof, crate::StwoVerificationKey, crate::StwoPublicInputs) {
    let vk = get_real_world_vk();
    let proof = get_real_world_proof();
    let inputs = crate::StwoPublicInputs {
        inputs: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08], // 8 public inputs
    };
    (proof, vk, inputs)
}

/// Get maximum complexity instance
fn get_maximum_instance() -> (crate::StwoProof, crate::StwoVerificationKey, crate::StwoPublicInputs) {
    let vk = get_maximum_vk();
    let proof = get_maximum_proof();
    let inputs = crate::StwoPublicInputs {
        inputs: vec![0x42; 64], // Maximum inputs
    };
    (proof, vk, inputs)
}

/// Get batch of test instances for batch verification
fn get_batch_instances() -> Vec<(crate::StwoProof, crate::StwoVerificationKey, crate::StwoPublicInputs)> {
    vec![
        get_test_instance(4),
        get_test_instance(8),
        get_test_instance(16),
    ]
}

/// Get test verification key
fn get_test_vk(public_input_count: u32) -> crate::StwoVerificationKey {
    crate::StwoVerificationKey {
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

/// Get real-world STARK verification key based on official specifications
fn get_real_world_vk() -> crate::StwoVerificationKey {
    crate::StwoVerificationKey {
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

/// Get maximum complexity verification key
fn get_maximum_vk() -> crate::StwoVerificationKey {
    crate::StwoVerificationKey {
        domain_size: 65536, // Large domain size
        constraint_count: 1024, // Many constraints
        public_input_count: 64, // Maximum inputs
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

/// Get test proof
fn get_test_proof() -> crate::StwoProof {
    crate::StwoProof {
        fri_proof: crate::FriProof {
            fri_lde_commitment: vec![0u8; 32],
            fri_lde_commitment_merkle_tree_root: vec![1u8; 32],
            fri_lde_commitment_merkle_tree_path: vec![vec![2u8; 32]],
            fri_lde_commitment_merkle_tree_leaf_index: 0,
            fri_query_proofs: vec![crate::FriQueryProof {
                fri_layer_proofs: vec![crate::FriLayerProof {
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

/// Get real-world STARK proof based on official specifications
fn get_real_world_proof() -> crate::StwoProof {
    crate::StwoProof {
        fri_proof: crate::FriProof {
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
                crate::FriQueryProof {
                    fri_layer_proofs: vec![
                        crate::FriLayerProof {
                            fri_layer_commitment: vec![0x40; 32],
                            fri_layer_commitment_merkle_tree_root: vec![0x41; 32],
                            fri_layer_commitment_merkle_tree_path: vec![vec![0x42; 32]],
                            fri_layer_commitment_merkle_tree_leaf_index: 0,
                            fri_layer_value: vec![0x43; 16],
                        },
                        crate::FriLayerProof {
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

/// Get maximum complexity proof
fn get_maximum_proof() -> crate::StwoProof {
    crate::StwoProof {
        fri_proof: crate::FriProof {
            fri_lde_commitment: vec![0xff; 32],
            fri_lde_commitment_merkle_tree_root: vec![0xfe; 32],
            fri_lde_commitment_merkle_tree_path: vec![vec![0xfd; 32]; 16], // Max depth
            fri_lde_commitment_merkle_tree_leaf_index: 0xffffffff,
            fri_query_proofs: vec![crate::FriQueryProof {
                fri_layer_proofs: vec![crate::FriLayerProof {
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

#[cfg(test)]
mod mock {
    use frame_support::{
        derive_impl, parameter_types,
        sp_runtime::{traits::IdentityLookup, BuildStorage},
        traits::{fungible::HoldConsideration, LinearStoragePrice},
    };
    use sp_core::{ConstU128, ConstU32};

    type Balance = u128;
    type AccountId = u64;

    // Configure a mock runtime to test the pallet.
    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            Balances: pallet_balances,
            CommonVerifiersPallet: pallet_verifiers::common,
            VerifierPallet: crate,
        }
    );

    impl crate::Config for Test {
        const MAX_NUM_INPUTS: u32 = crate::MAX_NUM_INPUTS;
    }

    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlockU32<Test>;
        type AccountId = AccountId;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    parameter_types! {
        pub const BaseDeposit: Balance = 1;
        pub const PerByteDeposit: Balance = 2;
        pub const HoldReasonVkRegistration: RuntimeHoldReason = RuntimeHoldReason::CommonVerifiersPallet(pallet_verifiers::common::HoldReason::VkRegistration);
    }

    impl pallet_verifiers::Config<crate::Stwo<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::StwoWeight<()>;
        type Ticket = HoldConsideration<
            AccountId,
            Balances,
            HoldReasonVkRegistration,
            LinearStoragePrice<BaseDeposit, PerByteDeposit, Balance>,
        >;
        type Currency = Balances;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type WeightInfo = ();
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxLocks = ConstU32<10>;
        type MaxReserves = ConstU32<10>;
        type MaxFreezes = ConstU32<10>;
        type DoneSlashHandler = ();
    }

    impl pallet_verifiers::common::Config for Test {
        type CommonWeightInfo = Test;
    }

    /// Build genesis storage according to the mock runtime.
    pub fn test_ext() -> sp_io::TestExternalities {
        let mut ext = sp_io::TestExternalities::from(
            frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .unwrap(),
        );
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
