// Copyright 2026, Horizen Labs, Inc.
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

use crate::Config;
use alloc::vec::Vec;
use core::{fmt, marker::PhantomData};

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::TypeInfo;

#[derive(Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct KimchiVk<T> {
    pub verifier_index_bytes: Vec<u8>,
    pub srs_bytes: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<T> KimchiVk<T> {
    pub fn new(verifier_index_bytes: Vec<u8>, srs_bytes: Vec<u8>) -> Self {
        Self {
            verifier_index_bytes,
            srs_bytes,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for KimchiVk<T> {
    fn clone(&self) -> Self {
        Self {
            verifier_index_bytes: self.verifier_index_bytes.clone(),
            srs_bytes: self.srs_bytes.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for KimchiVk<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KimchiVk")
            .field("verifier_index_bytes", &self.verifier_index_bytes)
            .field("srs_bytes", &self.srs_bytes)
            .finish()
    }
}

impl<T> PartialEq for KimchiVk<T> {
    fn eq(&self, other: &Self) -> bool {
        self.verifier_index_bytes == other.verifier_index_bytes && self.srs_bytes == other.srs_bytes
    }
}

impl<T: Config> MaxEncodedLen for KimchiVk<T> {
    fn max_encoded_len() -> usize {
        codec::Compact(T::max_vk_size()).encoded_size()
            + T::max_vk_size() as usize
            + codec::Compact(T::max_srs_size()).encoded_size()
            + T::max_srs_size() as usize
    }
}
