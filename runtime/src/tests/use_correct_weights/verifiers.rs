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

//! Here vwe implement just the test about verifiers weights linking.

use super::*;

#[test]
fn pallet_groth16_verifier() {
    use pallet_groth16_verifier::Groth16;
    use pallet_groth16_verifier::WeightInfo;

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<Groth16<Runtime>>>::WeightInfo as
            pallet_verifiers::WeightInfo<Groth16<Runtime>>>
            ::verify_proof(
            &pallet_groth16_verifier::Proof::default(),
            &Vec::new()
        ),
        crate::weights::pallet_groth16_verifier::ZKVWeight::<Runtime>::verify_proof_bn254(0)
    );
}

#[test]
fn pallet_settlement_risc0() {
    use pallet_risc0_verifier::Risc0;
    use pallet_risc0_verifier::WeightInfo;

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<Risc0<Runtime>>>::WeightInfo as
            pallet_verifiers::WeightInfo<Risc0<Runtime>>>
            ::verify_proof(
            &pallet_risc0_verifier::Proof::V1_1(Vec::new()),
            &Vec::new()
        ),
        crate::weights::pallet_risc0_verifier::ZKVWeight::<Runtime>::verify_proof_cycle_2_pow_13()
    );
}

#[test]
fn pallet_settlement_ultraplonk() {
    use pallet_ultraplonk_verifier::{Ultraplonk, WeightInfo};

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<Ultraplonk<Runtime>>>::WeightInfo as
            pallet_verifiers::WeightInfo<Ultraplonk<Runtime>>>
            ::verify_proof(
            &vec![0; pallet_ultraplonk_verifier::PROOF_SIZE],
            &Vec::new()
        ),
        crate::weights::pallet_ultraplonk_verifier::ZKVWeight::<Runtime>::verify_proof()
    );
}

#[test]
fn pallet_settlement_proofofsql() {
    use pallet_proofofsql_verifier::{ProofOfSql, WeightInfo};

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<ProofOfSql<Runtime>>>::WeightInfo as
            pallet_verifiers::WeightInfo<ProofOfSql<Runtime>>>
            ::verify_proof(
            &Vec::new(),
            &Vec::new()
        ),
        crate::weights::pallet_proofofsql_verifier::ZKVWeight::<Runtime>::verify_proof()
    );
}
