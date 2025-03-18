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

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use frame_support::{ensure, fail, pallet_prelude::*, weights::Weight};
use hp_verifiers::{Verifier, VerifyError};
use log::debug;
use risc0_verifier::poseidon2_injection::Boxed as _;
use risc0_verifier::{Journal, SegmentInfo, Verifier as _, VerifierContext, Vk as Risc0Vk};
use sp_core::{Get, H256};
use sp_std::vec::Vec;

pub mod benchmarking;
pub mod benchmarking_verify_proof;
pub mod extend_benchmarking;
pub mod fake_extend_benchmarking;

#[cfg(all(feature = "runtime-benchmarks", not(feature = "extend-benchmarks")))]
pub use fake_extend_benchmarking as extend_benchmarking;

mod verifier_should;
mod weight;
mod weight_verify_proof;

pub use crate::weight_verify_proof::WeightInfo as WeightInfoVerifyProof;
pub use weight::WeightInfo;

pub trait Config {
    /// Maximum number of 2^20 segments proof in a composite -> this, combined with
    /// `Segment20MaxSize`, defines also the maximum proof size (proof bigger than
    /// this value will be rejected).
    type MaxNSegment: Get<u32>;

    /// Maximum number of bytes contained in a 2^20 segment size
    type Segment20MaxSize: Get<u32>;

    /// Maximum number of bytes contained in the public inputs (otherwise rejected)
    type MaxPubsSize: Get<u32>;

    /// Weight info used to compute the verify proof weight
    type WeightInfo: WeightInfoVerifyProof;

    fn max_proof_size() -> u32 {
        Self::MaxNSegment::get() * Self::Segment20MaxSize::get()
    }

    fn max_pubs_size() -> u32 {
        Self::MaxPubsSize::get()
    }

    fn max_verify_proof_weight() -> Weight {
        Self::WeightInfo::verify_proof_segment_poseidon2_20() * Self::MaxNSegment::get() as u64
    }
}

#[pallet_verifiers::verifier]
pub struct Risc0<T>;

#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
pub enum Proof {
    V1_0(Vec<u8>),
    V1_1(Vec<u8>),
    V1_2(Vec<u8>),
}

pub type Pubs = Vec<u8>;
pub type Vk = H256;

enum R0Proof {
    V1_0(risc0_verifier::Proof),
    V1_1(risc0_verifier::Proof),
    V1_2(risc0_verifier::Proof),
}

enum ProofStructure {
    Succinct,
    Composite(Vec<SegmentInfo>),
}

impl R0Proof {
    fn verify(self, vk: Risc0Vk, journal: Journal) -> Result<(), hp_verifiers::VerifyError> {
        self.verifier()
            .verify(vk.into(), self.take_proof(), journal)
            .inspect_err(|e| log::debug!("Cannot verify proof: {:?}", e))
            .map_err(|_| hp_verifiers::VerifyError::VerifyError)
    }

    fn proof_structure(&self) -> Result<ProofStructure, ()> {
        let r0_proof = self.proof();
        let structure = match r0_proof.inner.composite().ok() {
            Some(c) => self
                .verifier()
                .extract_composite_segments_info(c)
                .map(ProofStructure::Composite)
                .map_err(|_| ())?,
            None => ProofStructure::Succinct,
        };
        Ok(structure)
    }

    fn proof(&self) -> &risc0_verifier::Proof {
        match self {
            R0Proof::V1_0(p) => p,
            R0Proof::V1_1(p) => p,
            R0Proof::V1_2(p) => p,
        }
    }

    fn take_proof(self) -> risc0_verifier::Proof {
        match self {
            R0Proof::V1_0(p) => p,
            R0Proof::V1_1(p) => p,
            R0Proof::V1_2(p) => p,
        }
    }

    fn verifier(&self) -> sp_std::boxed::Box<dyn risc0_verifier::Verifier> {
        match self {
            R0Proof::V1_0(_r0_proof) => VerifierContext::v1_0()
                .inject_native_poseidon2_if_needed()
                .boxed(),
            R0Proof::V1_1(_r0_proof) => VerifierContext::v1_1()
                .inject_native_poseidon2_if_needed()
                .boxed(),
            R0Proof::V1_2(_r0_proof) => VerifierContext::v1_2()
                .inject_native_poseidon2_if_needed()
                .boxed(),
        }
    }
}

impl TryFrom<&Proof> for R0Proof {
    type Error = ();

    fn try_from(proof: &Proof) -> Result<Self, Self::Error> {
        let risc0_proof = ciborium::from_reader(proof.bytes()).map_err(|_| ())?;
        Ok(match proof {
            Proof::V1_0(_) => Self::V1_0(risc0_proof),
            Proof::V1_1(_) => Self::V1_1(risc0_proof),
            Proof::V1_2(_) => Self::V1_2(risc0_proof),
        })
    }
}

impl Proof {
    fn len(&self) -> usize {
        match self {
            Proof::V1_0(proof_bytes) => proof_bytes.len(),
            Proof::V1_1(proof_bytes) => proof_bytes.len(),
            Proof::V1_2(proof_bytes) => proof_bytes.len(),
        }
    }

    fn bytes(&self) -> &[u8] {
        match self {
            Proof::V1_0(proof_bytes) => proof_bytes.as_slice(),
            Proof::V1_1(proof_bytes) => proof_bytes.as_slice(),
            Proof::V1_2(proof_bytes) => proof_bytes.as_slice(),
        }
    }
}

mod native_poseidon2 {
    #![cfg(feature = "inject-native-poseidon2")]
    //! This module provide [`NativePoseidon2Mix`] that can handle the poseidon2 mix native
    //! implementation.

    use risc0_verifier::poseidon2_injection::{BabyBearElem, Poseidon2Mix, POSEIDON2_CELLS};

    /// Implement Poseidon2 mix native implementation.
    pub struct NativePoseidon2Mix;

    impl Poseidon2Mix for NativePoseidon2Mix {
        fn poseidon2_mix(&self, cells: &mut [BabyBearElem; POSEIDON2_CELLS]) {
            native::Poseidon2Mix::new(cells).poseidon2_mix();
        }
    }
}

/// Inject native poseidon2 mix into VerifierContext if it's needed: if and only if
/// the `"inject-native-poseidon2"` is enabled.
pub trait InjectNativePoseidon2IfNeeded {
    fn inject_native_poseidon2_if_needed(self) -> Self;
}

impl<C: risc0_verifier::Verifier> InjectNativePoseidon2IfNeeded for C {
    #[cfg(feature = "inject-native-poseidon2")]
    fn inject_native_poseidon2_if_needed(mut self) -> Self {
        self.set_poseidon2_mix_impl(native_poseidon2::NativePoseidon2Mix.boxed());
        self
    }
    #[cfg(not(feature = "inject-native-poseidon2"))]
    fn inject_native_poseidon2_if_needed(self) -> Self {
        self
    }
}

impl<T: Config> Verifier for Risc0<T> {
    type Proof = Proof;

    type Pubs = Pubs;

    type Vk = Vk;

    fn hash_context_data() -> &'static [u8] {
        b"risc0"
    }

    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        log::trace!("Checking size");
        ensure!(
            proof.len() <= T::max_proof_size() as usize,
            hp_verifiers::VerifyError::InvalidProofData
        );
        ensure!(
            pubs.len() <= T::MaxPubsSize::get() as usize,
            hp_verifiers::VerifyError::InvalidInput
        );
        log::trace!("Verifying (native)");
        let journal = Journal::new(pubs.to_vec());
        let proof_len = proof.len();
        let proof = R0Proof::try_from(proof).map_err(|_| VerifyError::InvalidProofData)?;
        let w = proof
            .proof_structure()
            .and_then(Self::verify_weight)
            .map_err(|_| VerifyError::InvalidProofData)?;
        let max_w = T::max_verify_proof_weight().ref_time();
        if w.ref_time() > max_w {
            debug!(
                "Proof of size {proof_len} need weight {} > {max_w}",
                w.ref_time()
            );
            fail!(VerifyError::InvalidProofData)
        }
        proof.verify(vk.0.into(), journal).map(|_| Some(w))
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> hp_verifiers::Cow<[u8]> {
        hp_verifiers::Cow::Borrowed(pubs)
    }

    fn vk_hash(vk: &Self::Vk) -> H256 {
        *vk
    }

    fn vk_bytes(_vk: &Self::Vk) -> hp_verifiers::Cow<[u8]> {
        panic!("Risc0 vk is already hashed and we cannot know its preimage: use vk_hash() instead")
    }

    fn verifier_version_hash(proof: &Self::Proof) -> H256 {
        let h = match proof {
            Proof::V1_0(_) => hex_literal::hex!(
                "df801e3397d2a8fbb77c2fa30c7f7806ee8a60de44cb536108e7ef272618e2da"
            ),
            Proof::V1_1(_) => hex_literal::hex!(
                "2a06d398245e645477a795d1b707344669459840d154e17fde4df2b40eea5558"
            ),
            Proof::V1_2(_) => hex_literal::hex!(
                "5f39e7751602fc8dbc1055078b61e2704565e3271312744119505ab26605a942"
            ),
        };
        H256(h)
    }
}

impl<T: Config> Risc0<T> {
    fn verify_weight(structure: ProofStructure) -> Result<Weight, ()> {
        let w = match structure {
            ProofStructure::Succinct => T::WeightInfo::verify_proof_succinct(),
            ProofStructure::Composite(powers) => powers
                .into_iter()
                .map(|power| Self::segment_weight(power))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(Weight::default(), |acc, w| acc.add_ref_time(w.ref_time())),
        };
        Ok(w)
    }

    fn segment_weight(info: SegmentInfo) -> Result<Weight, ()> {
        let SegmentInfo { hash, po2 } = info;
        let w = match (hash.as_str(), po2) {
            ("poseidon2", p) if p <= 16 => T::WeightInfo::verify_proof_segment_poseidon2_16(),
            ("poseidon2", 17) => T::WeightInfo::verify_proof_segment_poseidon2_17(),
            ("poseidon2", 18) => T::WeightInfo::verify_proof_segment_poseidon2_18(),
            ("poseidon2", 19) => T::WeightInfo::verify_proof_segment_poseidon2_19(),
            ("poseidon2", 20) => T::WeightInfo::verify_proof_segment_poseidon2_20(),
            ("poseidon2", 21) => T::WeightInfo::verify_proof_segment_poseidon2_21(),
            ("sha-256", p) if p <= 16 => T::WeightInfo::verify_proof_segment_sha_256_16(),
            ("sha-256", 17) => T::WeightInfo::verify_proof_segment_sha_256_17(),
            ("sha-256", 18) => T::WeightInfo::verify_proof_segment_sha_256_18(),
            ("sha-256", 19) => T::WeightInfo::verify_proof_segment_sha_256_19(),
            ("sha-256", 20) => T::WeightInfo::verify_proof_segment_sha_256_20(),
            ("sha-256", 21) => T::WeightInfo::verify_proof_segment_sha_256_21(),
            _ => Err(())?,
        };
        Ok(w)
    }
}

/// The struct to use in runtime pallet configuration to map the weight computed by this crate
/// benchmarks to the weight needed by the `pallet-verifiers`.
pub struct Risc0Weight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Risc0<T>> for Risc0Weight<W> {
    fn verify_proof(
        _proof: &<Risc0<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Risc0<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        <T as Config>::max_verify_proof_weight()
    }

    fn register_vk(_vk: &<Risc0<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> frame_support::weights::Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Risc0<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Risc0<T> as Verifier>::Proof,
        _pubs: &<Risc0<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
