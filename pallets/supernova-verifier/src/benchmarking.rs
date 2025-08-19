#![cfg(feature = "runtime-benchmarks")]

use super::verifier;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

benchmarks! {
    verify {
        let input: Vec<u8> = vec![]; // dummy proof
    }: {
        verifier::verify_proof(&input).ok();
    }
}
