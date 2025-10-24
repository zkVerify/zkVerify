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

//! Here we implement just the test about verifiers weights linking.

use super::*;
use sp_core::H256;

#[test]
fn pallet_settlement_ezkl() {
    use pallet_ezkl_verifier::{Ezkl, WeightInfo};

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<Ezkl<Runtime>>>::WeightInfo as
        pallet_verifiers::WeightInfo<Ezkl<Runtime>>>
        ::verify_proof(
            &Vec::new(),
            &Vec::new()
        ),
        crate::weights::pallet_ezkl_verifier::ZKVWeight::<Runtime>::verify_proof()
    );
}

#[test]
fn pallet_fflonk_verifier() {
    use pallet_fflonk_verifier::Fflonk;
    let dummy_proof = [0; pallet_fflonk_verifier::PROOF_SIZE];
    let dummy_pubs = [0; pallet_fflonk_verifier::PUBS_SIZE];
    use pallet_fflonk_verifier::WeightInfo;

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<Fflonk>>::WeightInfo as pallet_verifiers::WeightInfo<Fflonk>>::verify_proof(
            &dummy_proof,
            &dummy_pubs
        ),
        crate::weights::pallet_fflonk_verifier::ZKVWeight::<Runtime>::verify_proof()
    );
}

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
            ::register_vk(&H256::default())
        ,
        crate::weights::pallet_risc0_verifier::ZKVWeight::<Runtime>::register_vk()
    );
}

#[test]
fn pallet_settlement_risc0_verify_proof() {
    use pallet_risc0_verifier::WeightInfoVerifyProof;

    assert_eq!(
        <Runtime as pallet_risc0_verifier::Config>::WeightInfo::verify_proof_segment_poseidon2_20()
        ,
        crate::weights::pallet_risc0_verifier_verify_proof::ZKVWeight::<Runtime>::verify_proof_segment_poseidon2_20()
    );
}

#[test]
fn pallet_settlement_ultrahonk() {
    use pallet_ultrahonk_verifier::{Ultrahonk, WeightInfo};

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<Ultrahonk<Runtime>>>::WeightInfo as
            pallet_verifiers::WeightInfo<Ultrahonk<Runtime>>>
            ::verify_proof(
            &pallet_ultrahonk_verifier::Proof::new(pallet_ultrahonk_verifier::ProofType::ZK, vec![0; pallet_ultrahonk_verifier::ZK_PROOF_SIZE]),
            &Vec::new()
        ),
        crate::weights::pallet_ultrahonk_verifier::ZKVWeight::<Runtime>::verify_proof_zk_32()
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
fn pallet_settlement_plonky2() {
    use pallet_plonky2_verifier::{Plonky2, Plonky2Config, Vk, WeightInfo};

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<Plonky2<Runtime>>>::WeightInfo as
        pallet_verifiers::WeightInfo<Plonky2<Runtime>>>
        ::register_vk(
            &Vk::new(Plonky2Config::Poseidon, Vec::new())
        ),
        crate::weights::pallet_plonky2_verifier::ZKVWeight::<Runtime>::register_vk()
    );
}

#[test]
fn pallet_settlement_plonky2_verify_proof() {
    use pallet_plonky2_verifier::WeightInfoVerifyProof;

    assert_eq!(
        <Runtime as pallet_plonky2_verifier::Config>::WeightInfo::verify_proof_poseidon_uncompressed_19()
        ,
        crate::weights::pallet_plonky2_verifier_verify_proof::ZKVWeight::<Runtime>::verify_proof_poseidon_uncompressed_19()
    );
}

#[test]
fn pallet_settlement_sp1() {
    use pallet_sp1_verifier::{Sp1, WeightInfo};

    assert_eq!(
        <<Runtime as pallet_verifiers::Config<Sp1<Runtime>>>::WeightInfo as
            pallet_verifiers::WeightInfo<Sp1<Runtime>>>
            ::verify_proof(
            &Vec::new(),
            &Vec::new()
        ),
        crate::weights::pallet_sp1_verifier::ZKVWeight::<Runtime>::verify_proof()
    );
}
