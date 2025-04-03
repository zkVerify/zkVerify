#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::proof::Proof as MorphProof;
pub use crate::vk::{Plonky2Config, VkWithConfig};

use frame_support::ensure;
use frame_support::pallet_prelude::Get;
use frame_support::weights::Weight;
use hp_verifiers::{Cow, Verifier, VerifyError};
use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, KeccakGoldilocksConfig, PoseidonGoldilocksConfig};
use plonky2_verifier::validate::ValidateError;
use plonky2_verifier::verify;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

pub mod benchmarking;
mod proof;
pub(crate) mod verifier_should;
mod vk;
mod weight;

pub use weight::WeightInfo;

pub type Pubs = Vec<u8>;
pub type Proof<T> = MorphProof<T>;
pub type Vk<T> = VkWithConfig<T>;

const MAX_DEGREE_BITS: u32 = 19;

impl<T: Config> Vk<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        if self.bytes.len() > T::max_vk_size() as usize {
            return Err(VerifyError::InvalidVerificationKey);
        }
        Ok(())
    }
}

impl<T: Config> Proof<T> {
    pub fn validate_size(&self) -> Result<(), VerifyError> {
        if self.bytes.len() > T::max_proof_size() as usize {
            return Err(VerifyError::InvalidProofData);
        }
        Ok(())
    }
}

pub trait Config: 'static {
    /// Maximum number of bytes contained in the proof (otherwise rejected)
    type MaxProofSize: Get<u32>;
    /// Maximum number of bytes contained in the public inputs (otherwise rejected)
    type MaxPubsSize: Get<u32>;
    /// Maximum number of bytes contained in the verification key (otherwise rejected)
    type MaxVkSize: Get<u32>;

    fn max_proof_size() -> u32 {
        Self::MaxProofSize::get()
    }

    fn max_pubs_size() -> u32 {
        Self::MaxPubsSize::get()
    }

    fn max_vk_size() -> u32 {
        Self::MaxVkSize::get()
    }
}

#[pallet_verifiers::verifier]
pub struct Plonky2<T>;

impl<T: Config> Verifier for Plonky2<T> {
    type Proof = Proof<T>;

    type Pubs = Pubs;

    type Vk = Vk<T>;

    fn hash_context_data() -> &'static [u8] {
        b"plonky2"
    }

    fn verify_proof(
        vk: &Self::Vk,
        raw_proof: &Self::Proof,
        raw_pubs: &Self::Pubs,
    ) -> Result<Option<Weight>, VerifyError> {
        vk.validate_size()?;
        raw_proof.validate_size()?;
        ensure!(
            raw_pubs.len() <= T::MaxPubsSize::get() as usize,
            hp_verifiers::VerifyError::InvalidInput
        );

        let vk = plonky2_verifier::Vk::from(vk.clone());
        let proof = plonky2_verifier::Proof::from(raw_proof.clone());

        verify(&vk, &proof, raw_pubs)
            .inspect_err(|e| log::debug!("Proof verification failed: {:?}", e))
            .map_err(|_| VerifyError::VerifyError)
            .map(|_| None)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), VerifyError> {
        vk.validate_size()?;

        let vk = plonky2_verifier::Vk::from(vk.clone());

        // validate_vk(&vk)
        //     .inspect_err(|e| log::debug!("VK validation failed: {:?}", e))
        //     .map_err(|_| VerifyError::InvalidVerificationKey)

        match vk.config {
            plonky2_verifier::Plonky2Config::Keccak => {
                const D: usize = 2;
                type C = KeccakGoldilocksConfig;
                type F = <C as GenericConfig<D>>::F;

                validate_vk_inner::<F, C, D>(&vk.bytes)
            }
            plonky2_verifier::Plonky2Config::Poseidon => {
                const D: usize = 2;
                type C = PoseidonGoldilocksConfig;
                type F = <C as GenericConfig<D>>::F;

                validate_vk_inner::<F, C, D>(&vk.bytes)
            }
        }
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]> {
        Cow::Borrowed(pubs)
    }
}

fn validate_vk_inner<F, C, const D: usize>(vk: &[u8]) -> plonky2_verifier::ValidateResult
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    let vk = plonky2_verifier::deserializer::deserialize_vk::<F, C, D>(vk)
        .map_err(ValidateError::from)
        .and_then(|vk| {
            (vk.common.config == CircuitConfig::standard_recursion_config()
                && vk.common.fri_params.degree_bits <= MAX_DEGREE_BITS)
                .then_some(())
                .ok_or(ValidateError::UnsupportedCircuitConfig)
        });
}

pub struct Plonky2Weight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Plonky2<T>>
    for Plonky2Weight<W>
{
    fn verify_proof(
        _proof: &<Plonky2<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Plonky2<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::verify_proof()
    }

    fn register_vk(_vk: &<Plonky2<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> frame_support::weights::Weight {
        W::unregister_vk()
    }

    fn get_vk() -> Weight {
        W::get_vk()
    }

    fn validate_vk(_vk: &<Plonky2<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::validate_vk()
    }

    fn compute_statement_hash(
        _proof: &<Plonky2<T> as Verifier>::Proof,
        _pubs: &<Plonky2<T> as Verifier>::Pubs,
    ) -> Weight {
        W::compute_statement_hash()
    }
}
