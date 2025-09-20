 #![cfg_attr(not(feature = "std"), no_std)]

use frame_support::weights::Weight;

pub trait WeightInfo {
	fn submit_proof(len_vk: u32, len_proof: u32, len_inputs: u32) -> Weight;
	fn register_vk(len_vk: u32) -> Weight;
	fn submit_proofs_batch(len_vk: u32, total_len_proofs: u32, total_len_inputs: u32, count: u32) -> Weight;
	fn submit_recursive_proof(len_vk: u32, len_proof: u32, len_inputs: u32) -> Weight;
	fn get_verification_stats() -> Weight;
}

 pub struct DefaultWeight;

impl WeightInfo for DefaultWeight {
	fn submit_proof(len_vk: u32, len_proof: u32, len_inputs: u32) -> Weight {
		// Simple linear model placeholder: base + per-byte cost
		let base = 10_000u64;
		let per_byte = 10u64;
		Weight::from_parts(base + per_byte * (len_vk as u64 + len_proof as u64 + len_inputs as u64), 0)
	}

	fn submit_proofs_batch(len_vk: u32, total_len_proofs: u32, total_len_inputs: u32, count: u32) -> Weight {
		let base = 20_000u64;
		let per_byte = 8u64;
		let per_item = 5_000u64;
		Weight::from_parts(base + per_item * count as u64 + per_byte * (len_vk as u64 + total_len_proofs as u64 + total_len_inputs as u64), 0)
	}

	fn register_vk(len_vk: u32) -> Weight {
		let base = 5_000u64;
		let per_byte = 5u64;
		Weight::from_parts(base + per_byte * len_vk as u64, 0)
	}

	fn submit_recursive_proof(len_vk: u32, len_proof: u32, len_inputs: u32) -> Weight {
		let base = 15_000u64;
		let per_byte = 12u64;
		Weight::from_parts(base + per_byte * (len_vk as u64 + len_proof as u64 + len_inputs as u64), 0)
	}

	fn get_verification_stats() -> Weight {
		Weight::from_parts(1_000u64, 0)
	}
}

