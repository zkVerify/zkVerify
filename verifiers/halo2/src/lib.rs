#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use frame_support::weights::Weight;
use hp_verifiers::{Cow, Verifier, VerifyError};
use halo2_proofs::plonk::verify_proof;

mod vk;

pub mod benchmarking;
mod verifier_should;
mod weight;
pub use weight::WeightInfo;

#[pallet_verifiers::verifier]
pub struct Halo2;

impl Verifier for Halo2 {
    type Proof = ();

    type Pubs = ();

    type Vk = ();

    fn hash_context_data() -> &'static [u8] {
        b"fflonk"
    }

    fn verify_proof(
        vk: &Self::Vk,
        raw_proof: &Self::Proof,
        raw_pubs: &Self::Pubs,
    ) -> Result<(), VerifyError> {
        // let vk: fflonk_verifier::VerificationKey = vk
        //     .clone()
        //     .try_into()
        //     .map_err(|e| log::debug!("Invalid Vk: {:?}", e))
        //     .map_err(|_| VerifyError::InvalidVerificationKey)?;
        // let pubs: fflonk_verifier::Public = (*raw_pubs).into();
        // let proof = fflonk_verifier::Proof::try_from(raw_proof)
        //     .map_err(|e| log::debug!("Cannot extract raw proof data: {:?}", e))
        //     .map_err(|_| VerifyError::InvalidProofData)?;
        // log::trace!(
        //     "Extracted public inputs [{:?}...{:?}] and proof data [{:?}...{:?}]",
        //     &raw_pubs[0],
        //     &raw_pubs[PUBS_SIZE - 1],
        //     &raw_proof[0],
        //     &raw_proof[PROOF_SIZE - 1]
        // );

        verify_proof()
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), hp_verifiers::VerifyError> {
        let _: fflonk_verifier::VerificationKey = vk
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