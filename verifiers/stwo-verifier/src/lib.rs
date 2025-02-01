pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use frame_support::weights::Weight;
use hp_verifiers::Verifier;
use sp_core::*;

pub mod benchmarking;
mod verifier_should;
mod weight;
pub use weight::WeightInfo;

pub type Vk = H256;
pub type Proof = [u8; 512];
pub type Pubs = [u8; 32];

#[pallet_verifiers::verifier]
pub struct Stwo<T>;

impl<T: Config> Verifier for Stwo<T> {
    type Vk = Vk;
    type Proof = Proof;
    type Pubs = Pubs;

    fn hash_context_data() -> &'static [u8] {
        b"stwo"
    }

    fn verify_proof(
        vk: &Self::Vk,
        proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<(), hp_verifiers::VerifyError> {
        log::trace!("Verifying proof");
        stwo_verifier::verify((*vk).into(), *proof, *pubs)
            .map_err(|_| log::debug!("Cannot verify stwo proof"))
            .map_err(|_| hp_verifiers::VerifyError::VerifyError)
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> hp_verifiers::Cow<[u8]> {
        hp_verifiers::Cow::Borrowed(pubs)
    }

    fn vk_hash(vk: &Self::Vk) -> H256 {
        *vk
    }
}

pub struct StwoWeight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Stwo<T>> for StwoWeight<W> {
    fn submit_proof(
        _proof: &<Stwo<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Stwo<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::submit_proof()
    }

    fn submit_proof_with_vk_hash(
        _proof: &<Stwo<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Stwo<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::submit_proof_with_vk_hash()
    }

    fn register_vk(_vk: &<Stwo<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }
}
