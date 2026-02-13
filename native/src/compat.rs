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

//! ABI compatibility wrapper for old runtimes.
//!
//! Provides [`PassPointerAndReadWrite`], a thin-pointer (u32/i32) FFI wrapper for
//! fixed-size byte arrays. This lets the node expose host functions with the old
//! ABI (thin pointer) alongside the new ABI (fat pointer), so old on-chain WASM
//! runtimes can still be executed during block sync.

use core::marker::PhantomData;
use sp_runtime_interface::RIType;

/// Thin-pointer read-write wrapper for fixed-size byte arrays.
///
/// Reads `N` bytes from a WASM memory pointer (u32), provides them to the host
/// function as `&mut T`, then writes the (possibly modified) bytes back.
pub struct PassPointerAndReadWrite<T, const N: usize>(PhantomData<T>);

impl<T, const N: usize> RIType for PassPointerAndReadWrite<T, N> {
    type FFIType = u32;
    type Inner = T;
}

#[cfg(not(substrate_runtime))]
mod host_impl {
    use super::*;
    use sp_runtime_interface::host::FromFFIValue;
    use sp_runtime_interface::sp_wasm_interface::{FunctionContext, Pointer, Result};

    impl<'a, T, const N: usize> FromFFIValue<'a> for PassPointerAndReadWrite<&'a mut T, N>
    where
        T: From<[u8; N]> + AsRef<[u8]>,
    {
        type Owned = T;

        fn from_ffi_value(context: &mut dyn FunctionContext, arg: u32) -> Result<Self::Owned> {
            let mut buf = [0u8; N];
            context.read_memory_into(Pointer::new(arg), &mut buf)?;
            Ok(T::from(buf))
        }

        fn take_from_owned(owned: &'a mut Self::Owned) -> &'a mut T {
            owned
        }

        fn write_back_into_runtime(
            value: Self::Owned,
            context: &mut dyn FunctionContext,
            arg: u32,
        ) -> Result<()> {
            let bytes = value.as_ref();
            assert_eq!(bytes.len(), N);
            context.write_memory(Pointer::new(arg), bytes)
        }
    }
}

#[cfg(substrate_runtime)]
mod wasm_impl {
    use super::*;
    use sp_runtime_interface::wasm::IntoFFIValue;

    impl<'a, const N: usize> IntoFFIValue for PassPointerAndReadWrite<&'a mut [u8; N], N> {
        type Destructor = ();

        fn into_ffi_value(value: &mut Self::Inner) -> (u32, ()) {
            ((*value).as_ptr() as u32, ())
        }
    }
}
