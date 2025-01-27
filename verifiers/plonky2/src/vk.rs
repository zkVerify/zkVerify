///! Verification Key to be morphed into `plonky2` variant.

use crate::Config;
use alloc::vec::Vec;

use core::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use educe::Educe;
use frame_support::pallet_prelude::TypeInfo;

#[derive(Copy, Clone, Debug, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Plonky2Config {
    Keccak,
    Poseidon,
}

impl Default for Plonky2Config {
    fn default() -> Self {
        Plonky2Config::Poseidon
    }
}

// Here educe is used for Clone, Debug, and PartialEq to work around
// a long-standing compiler bug https://github.com/rust-lang/rust/issues/26925
#[derive(Educe, Encode, Decode, TypeInfo)]
#[educe(Clone, Debug, PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct VkWithConfig<T> {
    pub config: Plonky2Config,
    pub bytes: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<T: Config> MaxEncodedLen for VkWithConfig<T> {
    fn max_encoded_len() -> usize {
        Plonky2Config::max_encoded_len() + T::max_vk_size() as usize
    }
}

impl From<Plonky2Config> for plonky2_verifier::Plonky2Config{
    fn from(config: Plonky2Config) -> Self {
        match config{
            Plonky2Config::Keccak => plonky2_verifier::Plonky2Config::Keccak,
            Plonky2Config::Poseidon => plonky2_verifier::Plonky2Config::Poseidon,
        }
    }
}

impl<T: Config> From<VkWithConfig<T>> for plonky2_verifier::Vk{
    fn from(vk: VkWithConfig<T>) -> Self {
        Self{
            config: vk.config.into(),
            bytes: vk.bytes,
        }
    }
}

impl<T: Config> Default for VkWithConfig<T> {
    fn default() -> Self {
        Self {
            config: Plonky2Config::default(),
            bytes: Vec::default(),
            _marker: PhantomData,
        }
    }
}

impl<T: Config> VkWithConfig<T> {
    #[allow(dead_code)] // used in resources.rs
    pub(crate) fn from_default_with_bytes(bytes: Vec<u8>) -> Self {
        Self {
             bytes,
            ..Default::default()
        }
    }
}