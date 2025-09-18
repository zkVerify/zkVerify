// Copyright 2025, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg(test)]

use hex_literal::hex;

use super::*;
include!("resources.rs");

struct MockRuntime;

impl crate::Config for MockRuntime {
    type MaxPubs = sp_core::ConstU32<10>;
}

#[test]
fn verify_valid_proof() {
    let vk = EzklVk::new(VALID_VKA.to_vec());
    let proof = VALID_PROOF.to_vec();
    let pi = valid_instances();

    assert!(Ezkl::<MockRuntime>::verify_proof(&vk, &proof, &pi).is_ok());
}
