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

//! Proof to be morphed into `plonky2` variant.
use crate::Config;
use alloc::vec::Vec;

use core::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use educe::Educe;
use frame_support::pallet_prelude::TypeInfo;

// Here educe is used for Clone, Debug, and PartialEq to work around
// a long-standing compiler bug https://github.com/rust-lang/rust/issues/26925
#[derive(Educe, Encode, Decode, TypeInfo)]
#[educe(Clone, Debug, PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct Proof<T> {
    pub bytes: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<T: Config> Proof<T> {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            _marker: PhantomData,
        }
    }
}

impl<T: Config> MaxEncodedLen for Proof<T> {
    fn max_encoded_len() -> usize {
        bool::max_encoded_len()
            + codec::Compact(T::max_proof_size()).encoded_size()
            + T::max_proof_size() as usize
    }
}

impl<T: Config> From<Proof<T>> for plonky2_verifier::Proof {
    fn from(proof: Proof<T>) -> Self {
        Self {
            compressed: false,
            bytes: proof.bytes,
        }
    }
}

impl<T: Config> Default for Proof<T> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
