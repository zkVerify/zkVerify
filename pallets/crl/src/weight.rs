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

//! Weight functions for `pallet_crl`.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_crl`.
pub trait WeightInfo {
    /// Weight for registering a new CA.
    fn register_ca() -> Weight;
    /// Weight for unregistering a CA.
    fn unregister_ca() -> Weight;
    /// Weight for updating the CRL for a CA.
    fn update_crl(n: u32) -> Weight;
    /// Weight for clearing the CRL for a CA.
    fn clear_crl() -> Weight;
}

/// Weights for pallet_crl using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: `Crl::CertificateAuthorities` (r:1 w:1)
    /// Storage: `Crl::CaList` (r:1 w:1)
    fn register_ca() -> Weight {
        Weight::from_parts(20_000_000, 3500)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    /// Storage: `Crl::CertificateAuthorities` (r:1 w:1)
    /// Storage: `Crl::CaList` (r:1 w:1)
    /// Storage: `Crl::RevokedIssuers` (r:0 w:n)
    /// Storage: `Crl::RevokedSerialNumbers` (r:0 w:n)
    fn unregister_ca() -> Weight {
        Weight::from_parts(30_000_000, 3500)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    /// Storage: `Crl::CertificateAuthorities` (r:1 w:1)
    /// Storage: `Crl::RevokedIssuers` (r:0 w:n)
    /// Storage: `Crl::RevokedSerialNumbers` (r:0 w:n)
    /// The range of component `n` is `[0, 65536]` (CRL PEM size in bytes).
    fn update_crl(n: u32) -> Weight {
        // Base weight for parsing and verification
        Weight::from_parts(50_000_000, 3500)
            // Per-byte weight for CRL parsing
            .saturating_add(Weight::from_parts(1_000, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }

    /// Storage: `Crl::CertificateAuthorities` (r:1 w:1)
    /// Storage: `Crl::RevokedIssuers` (r:0 w:n)
    /// Storage: `Crl::RevokedSerialNumbers` (r:0 w:n)
    fn clear_crl() -> Weight {
        Weight::from_parts(15_000_000, 3500)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}

// For backwards compatibility and tests.
impl WeightInfo for () {
    fn register_ca() -> Weight {
        Weight::from_parts(20_000_000, 3500)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }

    fn unregister_ca() -> Weight {
        Weight::from_parts(30_000_000, 3500)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }

    fn update_crl(n: u32) -> Weight {
        Weight::from_parts(50_000_000, 3500)
            .saturating_add(Weight::from_parts(1_000, 0).saturating_mul(n.into()))
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    fn clear_crl() -> Weight {
        Weight::from_parts(15_000_000, 3500)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
}
