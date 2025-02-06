#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[macro_use]
extern crate alloc;

use core::marker::PhantomData;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

mod benchmarking;
mod verifier_should;
mod weight;

use codec::{Decode, Encode, MaxEncodedLen};
use educe::Educe;
use frame_support::weights::Weight;
use halo2_verifier::halo2curves::bn256;
use halo2_verifier::halo2curves::bn256::{Bn256, G1Affine};
use halo2_verifier::halo2curves::ff::PrimeField;
use halo2_verifier::halo2curves::serde::SerdeObject;
use halo2_verifier::helpers::SerdeFormat;
use halo2_verifier::plonk::{Any, Column};
use halo2_verifier::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
use halo2_verifier::poly::kzg::multiopen::VerifierSHPLONK;
use halo2_verifier::poly::kzg::strategy::SingleStrategy;
use halo2_verifier::transcript::{Blake2bRead, Challenge255, TranscriptReadBuffer};
use halo2_verifier::{verify_proof, VerifyingKey};
use hp_verifiers::{Cow, Verifier, VerifyError};
use scale_info::TypeInfo;
use sp_core::{Get, U256};

#[pallet_verifiers::verifier]
pub struct Halo2<T>;

// Here educe is used for Clone, Debug, and PartialEq to work around
// a long-standing compiler bug https://github.com/rust-lang/rust/issues/26925
#[derive(Educe, Encode, Decode, TypeInfo)]
#[educe(Clone, Debug, PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct ParamsAndVk<T> {
    pub params_bytes: Vec<u8>,
    pub vk_bytes: Vec<u8>,
    _t: PhantomData<T>,
}


impl<T: Config> Verifier for Halo2<T> {
    type Proof = Vec<u8>;

    type Pubs = Vec<U256>;

    type Vk = ParamsAndVk<T>;

    fn hash_context_data() -> &'static [u8] {
        b"halo2"
    }

    fn verify_proof(
        vk_and_params: &Self::Vk,
        raw_proof: &Self::Proof,
        pubs: &Self::Pubs,
    ) -> Result<(), VerifyError> {
        let (params, vk) = vk_and_params.decode()?;

        let pubs = pubs
            .clone()
            .iter()
            .map(|x| {
                let mut bytes = [0; 32];
                x.to_little_endian(&mut bytes);
                bn256::Fr::from_bytes(&bytes).into_option()
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(VerifyError::InvalidInput)?;

        let instance = if vk.cs().num_instance_columns > 0 {
            vec![&pubs[..]]
        } else {
            vec![]
        };

        let mut transcript = Blake2bRead::init(raw_proof.as_slice());

        let strategy = SingleStrategy::new(&params);

        verify_proof::<
            KZGCommitmentScheme<Bn256>,
            VerifierSHPLONK<'_, Bn256>,
            Challenge255<G1Affine>,
            Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
            SingleStrategy<'_, Bn256>,
        >(&params, &vk, strategy, &[&instance[..]], &mut transcript)
        .map(|_| ())
        .map_err(|e| log::debug!("Verification failed: {:?}", e))
        .map_err(|_| VerifyError::VerifyError)
    }

    fn validate_vk(vk: &Self::Vk) -> Result<(), hp_verifiers::VerifyError> {
        vk.validate_size()?;
        let _ = vk.decode()?;

        Ok(())
    }

    fn pubs_bytes(pubs: &Self::Pubs) -> Cow<[u8]> {
        Cow::Owned(pubs.encode())
    }
}

impl<T: Config> ParamsAndVk<T> {
    pub fn decode(
        &self,
    ) -> Result<(ParamsKZG<bn256::Bn256>, VerifyingKey<bn256::G1Affine>), VerifyError> {
        let params = ParamsKZG::<bn256::Bn256>::read(&mut &self.params_bytes[..])
            .map_err(|e| {
            log::debug!("Invalid params: {:?}", e);
            VerifyError::InvalidVerificationKey
        })?;
        let vk =
            VerifyingKey::<bn256::G1Affine>::read(&mut &self.vk_bytes[..], SerdeFormat::RawBytes)
                .map_err(|e| {
                log::debug!("Invalid verifying key: {:?}", e);
                VerifyError::InvalidVerificationKey
            })?;
        Ok((params, vk))
    }

    pub fn validate_size(&self) -> Result<(), VerifyError> {
        log::debug!(
            "Validating sizes: params_bytes.len() = {}, vk_bytes.len() = {}",
            self.params_bytes.len(),
            self.vk_bytes.len()
        );

        if self.params_bytes.len() != ParamsKZG::<bn256::Bn256>::bytes_length() {
            log::debug!("Validation failed: Invalid params size.");
            return Err(VerifyError::InvalidVerificationKey);
        }

        if self.vk_bytes.len() > ParamsAndVk::<T>::max_encoded_len() {
            log::debug!("Validation failed: vk_bytes size exceeds max allowed.");
            return Err(VerifyError::InvalidVerificationKey);
        }

        log::debug!("Validation succeeded.");
        Ok(())
    }
}

impl<T: Config> ParamsAndVk<T> {
    pub fn flatten(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::with_capacity(self.params_bytes.len() + self.vk_bytes.len());
        bytes.extend_from_slice(&self.params_bytes);
        bytes.extend_from_slice(&self.vk_bytes);
        bytes
    }

    pub fn from_flattened(bytes: Vec<u8>) -> Result<Self, VerifyError> {
        let params_len = ParamsKZG::<bn256::Bn256>::bytes_length();
        let (left, right) = bytes.split_at(params_len);

        Ok(Self {
            params_bytes: left.to_vec(),
            vk_bytes: right.to_vec(),
            _t: PhantomData,
        })
    }
}

impl<T: Config> TryFrom<(ParamsKZG<bn256::Bn256>, VerifyingKey<bn256::G1Affine>)>
    for ParamsAndVk<T>
{
    type Error = VerifyError;

    fn try_from(
        value: (ParamsKZG<bn256::Bn256>, VerifyingKey<bn256::G1Affine>),
    ) -> Result<Self, Self::Error> {
        let (params, vk) = value;

        if vk.fixed_commitments().len() > T::max_fixed() {
            return Err(log::debug!(
                "too many fixed commitments: {:?}",
                vk.fixed_commitments().len()
            ))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;
        }

        if vk.permutation().commitments.len() > T::max_permutation() {
            return Err(log::debug!(
                "too many permutations: {:?}",
                vk.permutation().commitments.len()
            ))
            .map_err(|_| VerifyError::InvalidVerificationKey)?;
        }

        if vk.selectors.len() > T::max_selectors() {
            return Err(log::debug!("too many selectors: {:?}", vk.selectors.len()))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
        }

        let num_columns =
            vk.cs().num_advice_columns + vk.cs().num_instance_columns + vk.cs().num_fixed_columns;

        if num_columns > T::max_columns() {
            return Err(log::debug!("too many columns: {:?}", num_columns))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
        }

        let num_queries = vk.cs().advice_queries.len()
            + vk.cs().instance_queries.len()
            + vk.cs().fixed_queries.len();

        if num_queries > T::max_queries() {
            return Err(log::debug!("too many queries: {:?}", num_queries))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
        }

        if vk.cs().gates.len() > T::max_gates() {
            return Err(log::debug!("too many gates: {:?}", vk.cs().gates.len()))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
        }

        if vk.cs().lookups.len() > T::max_lookups() {
            return Err(log::debug!("too many lookups: {:?}", vk.cs().lookups.len()))
                .map_err(|_| VerifyError::InvalidVerificationKey)?;
        }

        let mut params_bytes = Vec::<u8>::with_capacity(ParamsKZG::<bn256::Bn256>::bytes_length());
        let mut vk_bytes = Vec::<u8>::with_capacity(vk.bytes_length());

        params.write(&mut params_bytes).unwrap();
        vk.write(&mut vk_bytes, SerdeFormat::RawBytes).unwrap();
        Ok(Self {
            params_bytes,
            vk_bytes,
            _t: PhantomData,
        })
    }
}

impl<T: Config> From<Vec<u8>> for ParamsAndVk<T> {
    fn from(value: Vec<u8>) -> Self {
        Self::from_flattened(value).unwrap()
    }
}

impl<T: Config> MaxEncodedLen for ParamsAndVk<T> {
    fn max_encoded_len() -> usize {
        let g1_bytes = bn256::G1Affine::default().to_raw_bytes().len();

        let expression_max = 8 + T::max_expression_degree() * (T::max_expression_vars() * 6 + 16);

        let cs_max = 24
            + T::max_columns() * 8
            + T::max_selectors() * 8
            + T::max_queries() * 6
            + T::max_permutation() * Column::<Any>::bytes_length()
            + T::max_gates() * expression_max
            + T::max_lookups() * expression_max * 2
            + T::max_shuffles() * expression_max * 2;

        40 + (T::max_fixed() * g1_bytes)
            + (T::max_permutation() * g1_bytes)
            + T::max_selectors() * ((1 << T::largest_k()) / 8 + 1)
            + (bn256::Fr::NUM_BITS as usize / 8)
            + cs_max
    }
}

/// The struct to use in runtime pallet configuration to map the weight computed by this crate
/// benchmarks to the weight needed by the `pallet-verifiers`.
/// In this case the implementation doesn't depends from the kind of proof or public input and
/// the crate's benchmarks are mapped 1-1 to the `pallet-verifiers`'s one.
pub struct Halo2Weight<W: weight::WeightInfo>(PhantomData<W>);

impl<T: Config, W: weight::WeightInfo> pallet_verifiers::WeightInfo<Halo2<T>> for Halo2Weight<W> {
    fn submit_proof(
        _proof: &<Halo2<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Halo2<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::submit_proof()
    }

    fn submit_proof_with_vk_hash(
        _proof: &<Halo2<T> as hp_verifiers::Verifier>::Proof,
        _pubs: &<Halo2<T> as hp_verifiers::Verifier>::Pubs,
    ) -> Weight {
        W::submit_proof_with_vk_hash()
    }

    fn register_vk(_vk: &<Halo2<T> as hp_verifiers::Verifier>::Vk) -> Weight {
        W::register_vk()
    }

    fn unregister_vk() -> Weight {
        W::unregister_vk()
    }
}

pub trait Config: 'static {
    type FixedMax: Get<u32>;
    type ColumnsMax: Get<u32>;
    type PermutationMax: Get<u32>;
    type SelectorMax: Get<u32>;
    type LargestK: Get<u32>;
    type QueriesMax: Get<u32>;
    type ExpressionDegreeMax: Get<u32>;
    type ExpressionVarsMax: Get<u32>;
    type GatesMax: Get<u32>;
    type LookupsMax: Get<u32>;
    type ShufflesMax: Get<u32>;
}

pub(crate) trait MaxFieldSizes {
    fn max_fixed() -> usize;
    fn max_columns() -> usize;

    fn max_permutation() -> usize;

    fn max_selectors() -> usize;

    fn largest_k() -> usize;

    fn max_queries() -> usize;

    fn max_expression_degree() -> usize;

    fn max_expression_vars() -> usize;

    fn max_gates() -> usize;

    fn max_lookups() -> usize;

    fn max_shuffles() -> usize;
}

impl<T: Config> MaxFieldSizes for T {
    fn max_fixed() -> usize {
        <Self as Config>::FixedMax::get() as usize
    }

    fn max_columns() -> usize {
        <Self as Config>::ColumnsMax::get() as usize
    }

    fn max_permutation() -> usize {
        <Self as Config>::PermutationMax::get() as usize
    }

    fn max_selectors() -> usize {
        <Self as Config>::SelectorMax::get() as usize
    }

    fn largest_k() -> usize {
        <Self as Config>::LargestK::get() as usize
    }

    fn max_queries() -> usize {
        <Self as Config>::QueriesMax::get() as usize
    }

    fn max_expression_degree() -> usize {
        <Self as Config>::ExpressionDegreeMax::get() as usize
    }

    fn max_expression_vars() -> usize {
        <Self as Config>::ExpressionVarsMax::get() as usize
    }

    fn max_gates() -> usize {
        <Self as Config>::GatesMax::get() as usize
    }

    fn max_lookups() -> usize {
        <Self as Config>::LookupsMax::get() as usize
    }

    fn max_shuffles() -> usize {
        <Self as Config>::ShufflesMax::get() as usize
    }
}
