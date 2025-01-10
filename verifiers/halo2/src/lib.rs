#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

mod circuit_info;
mod params;
mod vk;

use codec::Encode;
use halo2_proofs::halo2curves::bn256;
use halo2_proofs::halo2curves::bn256::{Bn256, G1Affine};
use halo2_proofs::plonk::verify_proof;
use halo2_proofs::poly::kzg::commitment::KZGCommitmentScheme;
use halo2_proofs::poly::kzg::multiopen::VerifierSHPLONK;
use halo2_proofs::poly::kzg::strategy::SingleStrategy;
use halo2_proofs::transcript::{Blake2bRead, Challenge255, TranscriptReadBuffer};
use hp_verifiers::{Cow, Verifier, VerifyError};

#[pallet_verifiers::verifier]
pub struct Halo2;

pub use crate::vk::Fr;
pub use vk::Vk;

impl Verifier for Halo2 {
    type Proof = Vec<u8>;

    type Pubs = Vec<Fr>;

    type Vk = (vk::Vk, params::ParamsKZG);

    fn hash_context_data() -> &'static [u8] {
        b"halo2"
    }

    fn verify_proof(
        (vk, params): &(vk::Vk, params::ParamsKZG),
        raw_proof: &Self::Proof,
        raw_pubs: &Self::Pubs,
    ) -> Result<(), VerifyError> {
        let params = params
            .clone()
            .try_into()
            .map_err(|e| log::debug!("Invalid Params: {:?}", e))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;

        let vk = vk
            .clone()
            .try_into()
            .map_err(|e| log::debug!("Invalid Vk: {:?}", e))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;

        let pubs = raw_pubs
            .iter()
            .map(|x| x.clone().into())
            .collect::<Vec<_>>();
        let mut transcript = Blake2bRead::init(raw_proof.as_slice());

        let strategy = SingleStrategy::new(&params);

        verify_proof::<
            KZGCommitmentScheme<Bn256>,
            VerifierSHPLONK<'_, Bn256>,
            Challenge255<G1Affine>,
            Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
            SingleStrategy<'_, Bn256>,
        >(
            &params,
            &vk,
            strategy,
            &[&[pubs.as_slice()]],
            &mut transcript,
        )
        .map(|_| ())
        .map_err(|e| log::debug!("Verification failed: {:?}", e))
        .map_err(|_| VerifyError::VerifyError)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), hp_verifiers::VerifyError> {
        let _: halo2_proofs::plonk::VerifyingKey<bn256::G1Affine> =
            vk.0.clone()
                .try_into()
                .map_err(|e| log::debug!("Invalid Vk: {:?}", e))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
        Ok(())
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]> {
        Cow::Owned(pubs.encode())
    }
}
