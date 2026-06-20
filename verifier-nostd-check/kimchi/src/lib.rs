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

#![no_std]

extern crate alloc;

use alloc::{vec, vec::Vec};
use frame_support::traits::ConstU32;
use pallet_kimchi_verifier::{Kimchi, KimchiProfileId, Vk, PUB_SIZE};
use pallet_verifiers::traits::Verifier;

struct DummyConfig;

impl pallet_kimchi_verifier::Config for DummyConfig {
    type MaxProofSize = ConstU32<1024>;
    type MaxPubs = ConstU32<4>;
    type MaxVkSize = ConstU32<1024>;
    type WeightInfo = ();
}

pub fn verify() {
    let vk = Vk::<DummyConfig>::new(vec![1_u8, 2, 3], KimchiProfileId::Vesta16);
    let proof = Vec::new();
    let pubs = vec![[0_u8; PUB_SIZE]];

    let _ = Kimchi::<DummyConfig>::hash_context_data();
    let _ = Kimchi::<DummyConfig>::validate_vk(&vk);
    let _ = Kimchi::<DummyConfig>::verify_proof(&vk, &proof, &pubs);
}
