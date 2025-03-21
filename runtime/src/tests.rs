// Copyright 2024, Horizen Labs, Inc.

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

use super::*;
/// Reexport `test` runner
pub use testsfixtures::{test, BABE_AUTHOR_ID, BLOCK_NUMBER, SLOT_ID};

mod availability;
mod misc;
mod pallets_interact;
mod payout;
mod testsfixtures;
mod use_correct_weights;
#![cfg(test)]

use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn store_verifying_key_works() {
    new_test_ext().execute_with(|| {
        let key_id = vec![1, 2, 3];
        let key = vec![4, 5, 6];
        
        assert_ok!(StwoVerifier::store_verifying_key(RuntimeOrigin::root(), key_id.clone(), key.clone()));
        
        assert_eq!(StwoVerifier::verifying_keys(&key_id), Some(key));
    });
}

#[test]
fn verify_proof_works() {
    new_test_ext().execute_with(|| {
        let key_id = vec![1, 2, 3];
        let key = vec![4, 5, 6];
        let proof = vec![7, 8, 9];
        let public_inputs = vec![10, 11, 12];
        
        assert_ok!(StwoVerifier::store_verifying_key(RuntimeOrigin::root(), key_id.clone(), key));
        
        assert_ok!(StwoVerifier::verify_proof(
            RuntimeOrigin::signed(1),
            key_id,
            proof,
            public_inputs
        ));
    });
}
