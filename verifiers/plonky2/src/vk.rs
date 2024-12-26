use crate::Config;
use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use educe::Educe;
use frame_support::pallet_prelude::TypeInfo;

#[derive(Copy, Clone, Debug, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Plonky2SystemConfig {
    Keccak,   // preset Keccak over Goldilocks config available in `plonky2`
    Poseidon, // preset Poseidon over Goldilocks config available in `plonky2`
}

impl Default for Plonky2SystemConfig {
    fn default() -> Self {
        Plonky2SystemConfig::Poseidon
    }
}

// Here educe is used for Clone, Debug, and PartialEq to work around
// a long-standing compiler bug https://github.com/rust-lang/rust/issues/26925
#[derive(Educe, Encode, Decode, TypeInfo)]
#[educe(Clone, Debug, PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct VerificationKeyWithSystemConfig<T> {
    pub system_config: Plonky2SystemConfig,
    pub vk_serialized: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<T: Config> MaxEncodedLen for VerificationKeyWithSystemConfig<T> {
    fn max_encoded_len() -> usize {
        Plonky2SystemConfig::max_encoded_len() + T::max_vk_size() as usize
    }
}

impl<T: Config> Default for VerificationKeyWithSystemConfig<T> {
    fn default() -> Self {
        Self {
            system_config: Plonky2SystemConfig::default(),
            vk_serialized: Vec::default(),
            _marker: PhantomData,
        }
    }
}

impl<T: Config> VerificationKeyWithSystemConfig<T> {
    pub(crate) fn from_default_with_bytes(bytes: Vec<u8>) -> Self {
        Self {
            vk_serialized: bytes,
            ..Default::default()
        }
    }
}
