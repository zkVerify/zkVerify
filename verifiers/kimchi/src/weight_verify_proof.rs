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

use frame_support::weights::Weight;

/// Weight functions needed for `pallet_kimchi_verifier_verify_proof`.
pub trait WeightInfo {
    fn verify_proof_domain_4096() -> Weight;
    fn verify_proof_domain_65536_pubs_64() -> Weight;
}

// For backwards compatibility and tests.
impl WeightInfo for () {
    fn verify_proof_domain_4096() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 15_495_000_000 picoseconds.
        Weight::from_parts(19_798_000_000, 0)
    }

    fn verify_proof_domain_65536_pubs_64() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 56_800_000_000 picoseconds.
        Weight::from_parts(74_082_000_000, 0)
    }
}
