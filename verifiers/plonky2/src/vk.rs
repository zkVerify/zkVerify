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

//! Verification Key to be morphed into `plonky2` variant.

use crate::Config;
use alloc::vec::Vec;
use core::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use educe::Educe;
use frame_support::pallet_prelude::TypeInfo;

#[derive(Copy, Clone, Debug, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub enum Plonky2Config {
    Keccak,
    #[default]
    Poseidon,
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

impl<T> VkWithConfig<T> {
    pub fn new(config: Plonky2Config, bytes: Vec<u8>) -> Self {
        Self {
            config,
            bytes,
            _marker: PhantomData,
        }
    }
}

impl<T: Config> MaxEncodedLen for VkWithConfig<T> {
    fn max_encoded_len() -> usize {
        Plonky2Config::max_encoded_len()
            + codec::Compact(T::max_vk_size()).encoded_size()
            + T::max_vk_size() as usize
    }
}

impl From<Plonky2Config> for plonky2_verifier::Plonky2Config {
    fn from(config: Plonky2Config) -> Self {
        match config {
            Plonky2Config::Keccak => plonky2_verifier::Plonky2Config::Keccak,
            Plonky2Config::Poseidon => plonky2_verifier::Plonky2Config::Poseidon,
        }
    }
}

impl<T: Config> From<VkWithConfig<T>> for plonky2_verifier::Vk {
    fn from(vk: VkWithConfig<T>) -> Self {
        Self {
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
