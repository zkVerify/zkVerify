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

#![no_std]

use sp_core::H256;

/// Trait used by proof verifier pallets to signal that a successful proof verification happened.
/// This must be implemented by proof storage pallets (e.g. pallet-poe) to subscribe to proof verification events.
#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait OnProofVerified<A> {
    fn on_proof_verified(account: Option<A>, domain_id: Option<u32>, pubs_hash: H256);
}
