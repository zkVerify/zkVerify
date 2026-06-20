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
    fn verify_proof_domain_4096_pubs_0() -> Weight;
    fn verify_proof_domain_65536_pubs_0() -> Weight;
    fn verify_proof_domain_131072_pubs_0() -> Weight;
    fn verify_proof_domain_262144_pubs_0() -> Weight;
    /// Per-public-input formula coefficient derived from the
    /// `#[benchmark(extra)]` public-input scaling probes.
    fn verify_proof_public_input() -> Weight;
}

// For backwards compatibility and tests.
impl WeightInfo for () {
    fn verify_proof_domain_4096_pubs_0() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Measured median execution time: 15_993_000_000 picoseconds.
        // Includes guard margin over a small simple-profile benchmark. Feature-heavy
        // small proofs route to the one-chunk upper tier.
        Weight::from_parts(21_000_000_000, 0)
    }

    fn verify_proof_domain_65536_pubs_0() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Measured median execution time: 62_396_000_000 picoseconds.
        // Includes guard margin over the max-feature one-chunk benchmark.
        Weight::from_parts(80_000_000_000, 0)
    }

    fn verify_proof_domain_131072_pubs_0() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Measured median execution time: 81_968_000_000 picoseconds.
        // Includes guard margin over the two-chunk lookup/runtime-table profile benchmark.
        Weight::from_parts(105_000_000_000, 0)
    }

    fn verify_proof_domain_262144_pubs_0() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Measured median execution time: 139_340_000_000 picoseconds.
        // Includes guard margin over the four-chunk lookup/runtime-table profile benchmark.
        Weight::from_parts(176_000_000_000, 0)
    }

    fn verify_proof_public_input() -> Weight {
        // Flat per-public-input coefficient. Local benchmarks show about
        // 7.5 us/chunk/public input; 50 us/public input covers the supported
        // four-chunk maximum with margin and keeps the production formula
        // chunk-independent.
        Weight::from_parts(50_000_000, 0)
    }
}
