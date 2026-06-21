// Copyright 2024-2026, Horizen Labs, Inc.
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

/// Decode a byte buffer into a properly aligned `Poseidon2Slice`, run `poseidon2_mix`,
/// and write the result back. Uses `u32::from_le_bytes` / `to_le_bytes` so no alignment
/// or host-endianness assumptions are needed — safe for both the v1 thin-pointer path
/// (stack `[u8; N]` with alignment 1) and the v2 fat-pointer path.
#[cfg(feature = "std")]
fn poseidon2_mix_bytes(bytes: &mut [u8]) {
    assert_eq!(bytes.len(), POSEIDON2_ARG_BYTES_SIZE);
    let mut cells: Poseidon2Slice = core::array::from_fn(|i| {
        let offset = i * 4;
        let w = u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect(
            "POSEIDON2_ARG_BYTES_SIZE is POSEIDON2_CELLS * 4, so each 4-byte chunk is valid; qed",
        ));
        BabyBearElem::new_raw(w)
    });
    risc0_verifier::poseidon2_injection::poseidon2_mix(&mut cells);
    for (i, cell) in cells.iter().enumerate() {
        let offset = i * 4;
        bytes[offset..offset + 4].copy_from_slice(&cell.as_u32_montgomery().to_le_bytes());
    }
}

#[runtime_interface]
pub trait Risc0Accelerate {
    /// Version 1: old ABI (thin pointer, i32). Registered for old on-chain runtimes.
    #[version(1, register_only)]
    fn poseidon2_mix(
        bytes: PassPointerAndReadWrite<&mut Poseidon2ArgBytes, POSEIDON2_ARG_BYTES_SIZE>,
    ) {
        poseidon2_mix_bytes(bytes);
    }

    /// Version 2: new ABI (fat pointer, i64). Used by the current runtime.
    #[version(2)]
    fn poseidon2_mix(bytes: PassFatPointerAndReadWrite<&mut [u8]>) {
        poseidon2_mix_bytes(bytes);
    }
}
