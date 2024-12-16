#![cfg_attr(not(feature = "std"), no_std)]

mod vk;
mod circuit_info;
mod params;

use codec::Encode;
use frame_benchmarking::BenchmarkParameter::r;
use frame_support::weights::Weight;
use halo2_proofs::halo2curves::bn256::G1Affine;
use hp_verifiers::{Cow, Verifier, VerifyError};
use halo2_proofs::plonk::verify_proof;
use halo2_proofs::transcript::{Blake2bRead, TranscriptReadBuffer};
use crate::vk::Fr;
use halo2_proofs::halo2curves::bn256;

#[pallet_verifiers::verifier]
pub struct Halo2;

struct Public(Vec<Vec<Vec<Fr>>>);

impl Verifier for Halo2 {
    type Proof = Vec<u8>;

    type Pubs = Vec<u8>;

    type Vk = (vk::Vk, params::ParamsKZG);

    fn hash_context_data() -> &'static [u8] {
        b"fflonk"
    }

    fn verify_proof(
        (vk, params): &(Self::Vk, params::ParamsKZG),
        raw_proof: &Self::Proof,
        raw_pubs: &Self::Pubs,
    ) -> Result<(), VerifyError> {
        let vk = vk
            .clone()
            .try_into()
            .map_err(|e| log::debug!("Invalid Vk: {:?}", e))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;
        let params = params
            .clone()
            .try_into()
            .map_err(|e| log::debug!("Invalid Params: {:?}", e))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;
        let pubs = raw_pubs.iter().map(|x| *x.map(|x| *x.map(|x| *x.into()))).collect();
        let mut transcript = Blake2bRead::init(raw_proof);

        // log::trace!(
        //     "Extracted public inputs [{:?}...{:?}] and proof data [{:?}...{:?}]",
        //     &raw_pubs[0],
        //     &raw_pubs[PUBS_SIZE - 1],
        //     &raw_proof[0],
        //     &raw_proof[PROOF_SIZE - 1]
        // );

        let strategy = halo2_proofs::poly::kzg::strategy::SingleStrategy::new(&params);

        verify_proof(&params, &vk, strategy, pubs, &mut transcript)
            .map(|_| ())
            .map_err(|e| log::debug!x("Verification failed: {:?}", e))
            .map_err(|_| VerifyError::VerifyError)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), hp_verifiers::VerifyError> {
        let _: halo2_proofs::plonk::VerifyingKey<bn256::G1Affine> = vk
            .clone()
            .try_into()
            .map_err(|e| log::debug!("Invalid Vk: {:?}", e))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;
        Ok(())
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]> {
        Cow::Borrowed(pubs)
    }
}

impl TryFrom<&[u8]> for Public {
    type Error = core::array::TryFromSliceError;

    fn try_from(inner: &[u8]) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}