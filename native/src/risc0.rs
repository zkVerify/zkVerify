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

use crate::compat::PassPointerAndReadWrite;
use sp_runtime_interface::pass_by::PassFatPointerAndReadWrite;
use sp_runtime_interface::runtime_interface;

use risc0_verifier::poseidon2_injection::{BabyBearElem, POSEIDON2_CELLS};

/// Define the byte slice for poseidon2 mix call argument type.
pub type Poseidon2ArgBytes = [u8; (u32::BITS as usize / u8::BITS as usize) * POSEIDON2_CELLS];
/// The expected size of the Poseidon2ArgBytes array.
pub const POSEIDON2_ARG_BYTES_SIZE: usize =
    (u32::BITS as usize / u8::BITS as usize) * POSEIDON2_CELLS;
/// Define the `BabyBearElem` slice for poseidon2 mix call argument type.
type Poseidon2Slice = [BabyBearElem; POSEIDON2_CELLS];

/// A struct that hide the `poseidon2_mix()` function native call. Use this struct call the native
/// implementation gives the possibility to be sure of don't pass some invalid data that is different from
/// a `BabyBearElem` `POSEIDON2_CELLS` array.
pub struct Poseidon2Mix<'a> {
    inner: &'a mut Poseidon2Slice,
}

impl<'a> Poseidon2Mix<'a> {
    #[inline]
    /// Create a new `Poseidon2Mix`.
    pub fn new(cells: &'a mut Poseidon2Slice) -> Self {
        Self { inner: cells }
    }

    /// Consume `self` and call the native `poseidon2_mix()` function on the inner
    /// `BabyBearElem` array.
    pub fn poseidon2_mix(self) {
        risc_0_accelerate::poseidon2_mix(self.into_mut_bytes())
    }

    #[inline]
    #[cfg(feature = "std")]
    /// SAFETY: BabyBearElem is always u32 and use `repr(transparent)`. Moreover
    /// this method is private and it's just used by `poseidon2_mix` that cannot be
    /// accessed outside of this module: only `Self::poseidon2_mix()` call it
    /// that can be just called from a `Poseidon2Mix` struct.
    /// The `Poseidon2Mix` struct can be built just from a mutable slice of `BabyBearElem`
    /// with the correct size.
    fn from_mut_bytes(bytes: &mut [u8]) -> Self {
        assert_eq!(bytes.len(), POSEIDON2_ARG_BYTES_SIZE);
        Self::new(unsafe {
            &mut *(bytes.as_mut_ptr() as *mut Poseidon2Slice)
        })
    }

    /// SAFETY: BabyBearElem is always u32 and use `repr(transparent)`. The inner
    /// mut slice can just be built from a mutable slice of `BabyBearElem`
    /// with the correct size. Moreover, all invariants of the `BabyBearElem` will
    /// be maintained from the only operation that can change them: `poseidon2_mix()`
    /// that get them again as `BabyBearElem`'s before call the `poseidon2_mix()` from
    /// risc0_verifier.
    fn into_mut_bytes(self) -> &'a mut Poseidon2ArgBytes {
        unsafe { core::mem::transmute::<&mut Poseidon2Slice, &mut Poseidon2ArgBytes>(self.inner) }
    }
}

#[runtime_interface]
pub trait Risc0Accelerate {
    /// Version 1: old ABI (thin pointer, i32). Registered for old on-chain runtimes.
    #[version(1, register_only)]
    fn poseidon2_mix(
        bytes: PassPointerAndReadWrite<&mut Poseidon2ArgBytes, POSEIDON2_ARG_BYTES_SIZE>,
    ) {
        let cells = Poseidon2Mix::from_mut_bytes(bytes);
        risc0_verifier::poseidon2_injection::poseidon2_mix(cells.inner);
    }

    /// Version 2: new ABI (fat pointer, i64). Used by the current runtime.
    #[version(2)]
    fn poseidon2_mix(bytes: PassFatPointerAndReadWrite<&mut [u8]>) {
        let cells = Poseidon2Mix::from_mut_bytes(bytes);
        risc0_verifier::poseidon2_injection::poseidon2_mix(cells.inner);
    }
}
