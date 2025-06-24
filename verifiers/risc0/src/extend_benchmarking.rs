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

#[benchmarks(where T: pallet_verifiers::Config<Risc0<T>>)]
mod benchmarks {
    use crate::{InjectNativePoseidon2IfNeeded, Proof, R0Proof};
    use risc0_verifier::Verifier;

    use super::*;

    impl TryFrom<crate::Proof> for R0Proof {
        type Error = crate::ConvertProofError;

        fn try_from(value: Proof) -> Result<Self, Self::Error> {
            (&value).try_into()
        }
    }

    pub static VALID_VK_V2_1: &[u8; 32] =
        &hex_literal::hex!("8e3794e8255e7810de2be7710fe19f79e538de060f038a21b24529e28d0b744c");

    static PROOF_SUCCINCT_V2_1: &[u8] = include_bytes!("resources/v_2_1_succinct_22.bin");

    static PUBS_22: [u8; 8] = hex_literal::hex!("00003c0000000000");

    static PUBS_16: [u8; 8] = hex_literal::hex!("1c40000000000000");

    static PROOF_POSEIDON2_22_V2_1: &[u8] = include_bytes!("resources/v_2_1_poseidon2_22.bin");

    static PROOF_POSEIDON2_16_V2_1: &[u8] = include_bytes!("resources/v_2_1_poseidon2_16.bin");

    #[benchmark]
    fn verify_poseidon2_succinct_not_accelerated() {
        let vk = (*VALID_VK_V2_1).into();
        let journal = risc0_verifier::Journal::new(PUBS_22.to_vec());

        #[block]
        {
            let proof: crate::R0Proof = Proof::V2_1(PROOF_SUCCINCT_V2_1.to_vec())
                .try_into()
                .unwrap();
            risc0_verifier::v2_1()
                .verify(vk, proof.take_proof(), journal)
                .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_succinct_accelerated() {
        let vk = (*VALID_VK_V2_1).into();
        let journal = risc0_verifier::Journal::new(PUBS_22.to_vec());

        #[block]
        {
            let proof: crate::R0Proof = Proof::V2_1(PROOF_SUCCINCT_V2_1.to_vec())
                .try_into()
                .unwrap();
            risc0_verifier::v2_1()
                .inject_native_poseidon2_if_needed()
                .verify(vk, proof.take_proof(), journal)
                .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_16_not_accelerated() {
        let vk = (*VALID_VK_V2_1).into();
        let journal = risc0_verifier::Journal::new(PUBS_16.to_vec());

        #[block]
        {
            let proof: crate::R0Proof = Proof::V2_1(PROOF_POSEIDON2_16_V2_1.to_vec())
                .try_into()
                .unwrap();
            risc0_verifier::v2_1()
                .verify(vk, proof.take_proof(), journal)
                .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_16_accelerated() {
        let vk = (*VALID_VK_V2_1).into();
        let journal = risc0_verifier::Journal::new(PUBS_16.to_vec());

        #[block]
        {
            let proof: crate::R0Proof = Proof::V2_1(PROOF_POSEIDON2_16_V2_1.to_vec())
                .try_into()
                .unwrap();
            risc0_verifier::v2_1()
                .inject_native_poseidon2_if_needed()
                .verify(vk, proof.take_proof(), journal)
                .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_22_not_accelerated() {
        let vk = (*VALID_VK_V2_1).into();
        let journal = risc0_verifier::Journal::new(PUBS_22.to_vec());

        #[block]
        {
            let proof: crate::R0Proof = Proof::V2_1(PROOF_POSEIDON2_22_V2_1.to_vec())
                .try_into()
                .unwrap();
            risc0_verifier::v2_1()
                .verify(vk, proof.take_proof(), journal)
                .unwrap()
        }
    }

    #[benchmark]
    fn verify_poseidon2_22_accelerated() {
        let vk = (*VALID_VK_V2_1).into();
        let journal = risc0_verifier::Journal::new(PUBS_22.to_vec());

        #[block]
        {
            let proof: crate::R0Proof = Proof::V2_1(PROOF_POSEIDON2_22_V2_1.to_vec())
                .try_into()
                .unwrap();
            risc0_verifier::v2_1()
                .inject_native_poseidon2_if_needed()
                .verify(vk, proof.take_proof(), journal)
                .unwrap()
        }
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::benchmarking::mock::test_ext(),
        crate::benchmarking::mock::Test
    );
}
