// Copyright 2024, Horizen Labs, Inc.

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

#![cfg(feature = "runtime-benchmarks")]
#![cfg(feature = "extend-benchmarks")]

//! An extended benchmarks that not produce an interface for the weights but
//! you can just use it to benchmarks the native some verifications against
//! WASM and native implementation.

use super::Risc0;
use frame_benchmarking::v2::*;

pub struct Pallet<T: Config>(crate::Pallet<T>);

pub use crate::benchmarking::{Call, Config};

#[benchmarks(where T: pallet_verifiers::Config<Risc0<T>> + pallet_aggregate::Config)]
mod benchmarks {

    use crate::benchmarking::VALID_VK;
    use crate::InjectNativePoseidon2IfNeeded;

    use super::*;

    static VK_RISC0_VERIFIER_VM_1_2_0: &[u8; 32] =
        &hex_literal::hex!("9db9988d9fbcacadf2bd29fc7c60b98bc4234342fe536eb983169eb6cc248009");
    static PROOF_SUCCINCT: &[u8; include_bytes!("resources/v_1_2_succinct_22.bin").len()] =
        include_bytes!("resources/v_1_2_succinct_22.bin");

    static PUBS_RISC0_VERIFIER_22: [u8; 8] = hex_literal::hex!("1d64010000000000");
    static PROOF_RISC0_VERIFIER_POSEIDON2_22_VM_1_2: &[u8; include_bytes!(
        "resources/v_1_2_poseidon2_22.bin"
    )
     .len()] = include_bytes!("resources/v_1_2_poseidon2_22.bin");

    static VK_RISC0_VERIFIER_VM_1_1_3: &[u8; 32] =
        &hex_literal::hex!("2addbbeb4ddb2f2ec2b4a0a8a21c03f7d3bf42cfd2ee9f4a69d2ebd9974218b6");
    static PROOF_RISC0_VERIFIER_POSEIDON2_16_VM_1_1: &[u8; include_bytes!(
        "resources/v_1_1_poseidon2_16.bin"
    )
     .len()] = include_bytes!("resources/v_1_1_poseidon2_16.bin");

    static PUBS_RISC0_VERIFIER_16: [u8; 8] = hex_literal::hex!("8105000000000000");

    #[benchmark]
    fn verify_sha2_22() {
        let vk = (*VK_RISC0_VERIFIER_VM_1_2_0).into();
        let journal = risc0_verifier::Journal::new(PUBS_RISC0_VERIFIER_22.to_vec());
        let proof = include_bytes!("resources_benchmarking/RISC0_SHA2_22_VM_1_2_0.bin");

        #[block]
        {
            crate::deserialize_and_verify_proof(
                &risc0_verifier::VerifierContext::v1_2(),
                vk,
                proof,
                journal,
            )
            .unwrap()
        }
    }

    #[benchmark]
    fn verify_legacy_22() {
        let inner_proof =
            include_bytes!("resources_benchmarking/LEGACY_VALID_PROOF_CYCLE_2_POW_22.bin").to_vec();
        let pubs = hex_literal::hex!("0400000000000000d4850100");
        #[block]
        {
            native::risc_0_verify::verify(VALID_VK.0, &inner_proof, &pubs)
                .map_err(|e| match e {
                    native::VerifyError::InvalidProofData => "Invalid proof",
                    native::VerifyError::InvalidInput => "Invalid public inputs",
                    native::VerifyError::InvalidVerificationKey => "Invalid Vk",
                    native::VerifyError::VerifyError => "Verify Error",
                })
                .unwrap();
        }
    }

    #[benchmark]
    fn verify_poseidon2_succinct_not_accelerated() {
        let vk = (*VK_RISC0_VERIFIER_VM_1_2_0).into();
        let journal = risc0_verifier::Journal::new(PUBS_RISC0_VERIFIER_22.to_vec());

        #[block]
        {
            crate::deserialize_and_verify_proof(
                &risc0_verifier::VerifierContext::v1_2(),
                vk,
                PROOF_SUCCINCT,
                journal,
            )
            .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_succinct_accelerated() {
        let vk = (*VK_RISC0_VERIFIER_VM_1_2_0).into();
        let journal = risc0_verifier::Journal::new(PUBS_RISC0_VERIFIER_22.to_vec());

        #[block]
        {
            crate::deserialize_and_verify_proof(
                &risc0_verifier::VerifierContext::v1_2().inject_native_poseidon2_if_needed(),
                vk,
                PROOF_SUCCINCT,
                journal,
            )
            .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_16_not_accelerated() {
        let vk = (*VK_RISC0_VERIFIER_VM_1_1_3).into();
        let journal = risc0_verifier::Journal::new(PUBS_RISC0_VERIFIER_16.to_vec());

        #[block]
        {
            crate::deserialize_and_verify_proof(
                &risc0_verifier::VerifierContext::v1_1(),
                vk,
                PROOF_RISC0_VERIFIER_POSEIDON2_16_VM_1_1,
                journal,
            )
            .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_16_accelerated() {
        let vk = (*VK_RISC0_VERIFIER_VM_1_1_3).into();
        let journal = risc0_verifier::Journal::new(PUBS_RISC0_VERIFIER_16.to_vec());

        #[block]
        {
            crate::deserialize_and_verify_proof(
                &risc0_verifier::VerifierContext::v1_1().inject_native_poseidon2_if_needed(),
                vk,
                PROOF_RISC0_VERIFIER_POSEIDON2_16_VM_1_1,
                journal,
            )
            .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_22_not_accelerated() {
        let vk = (*VK_RISC0_VERIFIER_VM_1_2_0).into();
        let journal = risc0_verifier::Journal::new(PUBS_RISC0_VERIFIER_22.to_vec());

        #[block]
        {
            crate::deserialize_and_verify_proof(
                &risc0_verifier::VerifierContext::v1_2(),
                vk,
                PROOF_RISC0_VERIFIER_POSEIDON2_22_VM_1_2,
                journal,
            )
            .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_22_accelerated() {
        let vk = (*VK_RISC0_VERIFIER_VM_1_2_0).into();
        let journal = risc0_verifier::Journal::new(PUBS_RISC0_VERIFIER_22.to_vec());

        #[block]
        {
            crate::deserialize_and_verify_proof(
                &risc0_verifier::VerifierContext::v1_2().inject_native_poseidon2_if_needed(),
                vk,
                PROOF_RISC0_VERIFIER_POSEIDON2_22_VM_1_2,
                journal,
            )
            .unwrap()
        }
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::benchmarking::mock::test_ext(),
        crate::benchmarking::mock::Test
    );
}
