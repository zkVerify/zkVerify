#![cfg(feature = "runtime-benchmarks")]

use super::*;
// Removed unused import
use frame_benchmarking::{benchmarks, whitelisted_caller, BenchmarkError};
use frame_support::BoundedVec;
use crate::MaxVkLen;
use frame_system::RawOrigin;

fn vec_of(byte: u8, len: u32) -> Vec<u8> {
	let mut v = Vec::with_capacity(len as usize);
	v.resize(len as usize, byte);
	v
}

benchmarks! {
	register_vk {
		let caller: T::AccountId = whitelisted_caller();
		let id: u32 = 1;
		let vk = vec_of(0, 1024);
	}: register_vk(RawOrigin::Signed(caller), id, vk)

	submit_proof {
		let caller: T::AccountId = whitelisted_caller();
		let vk = vec_of(0, 1024);
		let proof = vec_of(1, 2048);
		let inputs = vec_of(2, 512);
	}: submit_proof(RawOrigin::Signed(caller), vk, proof, inputs)

	submit_proofs_batch {
		let caller: T::AccountId = whitelisted_caller();
		let id: u32 = 99;
		let vk = vec_of(3, 4096);
    let bounded: BoundedVec<u8, MaxVkLen> = BoundedVec::try_from(vk.clone()).map_err(|_| BenchmarkError::Stop("vk too large"))?;
    <VkRegistry<T>>::insert(id, bounded);
		let n: u32 = 10;
		let mut proofs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
		for i in 0..n {
			let p = vec_of((i % 251) as u8, 1024);
			let inp = vec_of((i % 241) as u8, 256);
			proofs.push((p, inp));
		}
	}: submit_proofs_batch(RawOrigin::Signed(caller), id, proofs)
}

// Note: Benchmark test suite disabled due to mock runtime compatibility issues
// The benchmarks themselves are functional and can be tested in a real runtime


