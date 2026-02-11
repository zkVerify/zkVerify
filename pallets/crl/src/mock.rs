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

#![cfg(test)]

use frame_support::{
    derive_impl, parameter_types,
    sp_runtime::{traits::IdentityLookup, BuildStorage},
};
use frame_system::RawOrigin;

pub type AccountId = u64;
pub type Origin = RawOrigin<AccountId>;

pub const ALICE: AccountId = 1;
pub const CA_NAME: &[u8] = b"Test_CA";

/// Feb 11 2026 12:00:00 UTC, in milliseconds.
/// Falls within the validity period of the test certificates.
pub const PRESENT_MS: u64 = 1_770_811_200_000;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        CrlPallet: crate,
    }
);

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type Block = frame_system::mocking::MockBlockU32<Test>;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
}

parameter_types! {
    pub const MaxCaNameLength: u32 = 64;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type ManagerOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = ();
    type MaxCaNameLength = MaxCaNameLength;
    type UnixTime = Timestamp;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = sp_core::ConstU64<5>;
    type WeightInfo = ();
}

// Test CA hierarchy (all P-256 ECDSA):
//
//   Test Root CA (root_ca.der)
//   ├── Test Intermediate CA 1 (in chain1.pem)
//   │   ├── crl_inter1_v1.pem — CRL v2, 3 revoked certs, issued 2026-02-11 11:48:15 UTC
//   │   └── crl_inter1_v2.pem — CRL v2, 5 revoked certs, issued 2026-02-11 11:48:42 UTC
//   └── Test Intermediate CA 2 (in chain2.pem)
//       └── crl_inter2.pem   — CRL v2, 2 revoked certs, issued 2026-02-11 11:48:42 UTC
//
// chain PEM order: [intermediate cert, root cert]

/// Root CA certificate (DER encoded).
pub fn root_cert() -> Vec<u8> {
    include_bytes!("resources/test/root_ca.der").to_vec()
}

/// Certificate chain for Intermediate CA 1 (PEM: intermediate + root).
pub fn chain1() -> Vec<u8> {
    include_bytes!("resources/test/chain1.pem").to_vec()
}

/// Certificate chain for Intermediate CA 2 (PEM: intermediate + root).
pub fn chain2() -> Vec<u8> {
    include_bytes!("resources/test/chain2.pem").to_vec()
}

/// CRL from Intermediate CA 1, version 1 — 3 revoked certs, issued 2026-02-11 11:48:15 UTC.
pub fn crl_inter1_v1() -> Vec<u8> {
    include_bytes!("resources/test/crl_inter1_v1.pem").to_vec()
}

/// CRL from Intermediate CA 1, version 2 — 5 revoked certs, issued 2026-02-11 11:48:42 UTC.
pub fn crl_inter1_v2() -> Vec<u8> {
    include_bytes!("resources/test/crl_inter1_v2.pem").to_vec()
}

/// CRL from Intermediate CA 2 — 2 revoked certs, issued 2026-02-11 11:48:42 UTC.
pub fn crl_inter2() -> Vec<u8> {
    include_bytes!("resources/test/crl_inter2.pem").to_vec()
}

pub fn test() -> sp_io::TestExternalities {
    let mut ext = sp_io::TestExternalities::from(
        frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap(),
    );
    ext.execute_with(|| {
        System::set_block_number(1);
        pallet_timestamp::Now::<Test>::put(PRESENT_MS);
    });
    ext
}
