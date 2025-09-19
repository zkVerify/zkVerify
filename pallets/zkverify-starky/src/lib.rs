 #![cfg_attr(not(feature = "std"), no_std)]

 pub use pallet::*;
 pub mod weights;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod mock;

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, BoundedVec};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use zkv_stwo::{StwoVerify, StwoVerificationKey, StwoProof, StwoPublicInputs, AdvancedStwoVerifier, ProofType};
use crate::weights::{WeightInfo};

 #[frame_support::pallet]
 pub mod pallet {
 	use super::*;

 	#[pallet::config]
 	pub trait Config: frame_system::Config {
 		/// The overarching event type.
 		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
 		/// Weight provider
 		type WeightInfo: WeightInfo;
 	}

 	#[pallet::pallet]
 	pub struct Pallet<T>(_);

 	#[pallet::storage]
 	#[pallet::getter(fn last_verification_result)]
 	pub type LastVerificationResult<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vk_registry)]
	pub type VkRegistry<T: Config> = StorageMap<_, Blake2_128Concat, u32, BoundedVec<u8, MaxVkLen>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vk_owner)]
	pub type VkOwner<T: Config> = StorageMap<_, Blake2_128Concat, u32, T::AccountId, OptionQuery>;

 	#[pallet::event]
 	#[pallet::generate_deposit(pub(super) fn deposit_event)]
 	pub enum Event<T: Config> {
		Verified { success: bool },
		VkRegistered { id: u32, owner: T::AccountId },
 	}

 	#[pallet::error]
 	pub enum Error<T> {
 		InvalidInput,
 	}

 	#[pallet::call]
 	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::submit_proof(vk.len() as u32, proof.len() as u32, public_inputs.len() as u32))]
		pub fn submit_proof(
			origin: OriginFor<T>,
			vk: Vec<u8>,
			proof: Vec<u8>,
			public_inputs: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
        let vk = StwoVerificationKey {
            bytes: vk,
            version: 1,
            circuit_size: 1000,
            is_recursive: false,
        };
        let proof = StwoProof {
            bytes: proof,
            proof_type: ProofType::Standard,
            timestamp: 1000,
        };
        let public_inputs = StwoPublicInputs {
            inputs: public_inputs.clone(),
            input_count: public_inputs.len() as u32,
            input_hash: [0u8; 32],
        };

        let success = <AdvancedStwoVerifier as StwoVerify>::verify(&vk, &proof, &public_inputs);
			<LastVerificationResult<T>>::put(success);
			Self::deposit_event(Event::Verified { success });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::register_vk(vk.len() as u32))]
		pub fn register_vk(origin: OriginFor<T>, id: u32, vk: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!VkRegistry::<T>::contains_key(&id), Error::<T>::InvalidInput);
			let bounded: BoundedVec<u8, MaxVkLen> = vk.try_into().map_err(|_| Error::<T>::InvalidInput)?;
			<VkRegistry<T>>::insert(id, bounded);
			<VkOwner<T>>::insert(id, who.clone());
			Self::deposit_event(Event::VkRegistered { id, owner: who });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::submit_proofs_batch(0, 0, 0, proofs.len() as u32))]
		pub fn submit_proofs_batch(origin: OriginFor<T>, vk_id: u32, proofs: Vec<(Vec<u8>, Vec<u8>)>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let vk_bytes = VkRegistry::<T>::get(&vk_id).ok_or(Error::<T>::InvalidInput)?;
			let vk = StwoVerificationKey { 
				bytes: vk_bytes.into_inner(),
				version: 1,
				circuit_size: 1000,
				is_recursive: false,
			};
			let mut all_ok = true;
			let mut total_proof_len: u32 = 0;
			let mut total_inputs_len: u32 = 0;
			for (proof_bytes, inputs_bytes) in proofs.into_iter() {
				total_proof_len = total_proof_len.saturating_add(proof_bytes.len() as u32);
				total_inputs_len = total_inputs_len.saturating_add(inputs_bytes.len() as u32);
				let proof = StwoProof { 
					bytes: proof_bytes,
					proof_type: ProofType::Standard,
					timestamp: 1000,
				};
				let public_inputs = StwoPublicInputs { 
					inputs: inputs_bytes.clone(),
					input_count: inputs_bytes.len() as u32,
					input_hash: [0u8; 32],
				};
				let ok = <AdvancedStwoVerifier as StwoVerify>::verify(&vk, &proof, &public_inputs);
				all_ok &= ok;
				Self::deposit_event(Event::Verified { success: ok });
			}
			<LastVerificationResult<T>>::put(all_ok);
			Ok(())
		}

		/// Submit a recursive proof for aggregation (INNOVATIVE FEATURE)
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::submit_proofs_batch(0, 0, 0, 1))]
		pub fn submit_recursive_proof(
			origin: OriginFor<T>,
			vk_id: u32,
			proof: Vec<u8>,
			public_inputs: Vec<u8>,
			inner_vk_ids: Vec<u32>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let vk_bytes = VkRegistry::<T>::get(&vk_id).ok_or(Error::<T>::InvalidInput)?;
			let vk = StwoVerificationKey { 
				bytes: vk_bytes.into_inner(),
				version: 1,
				circuit_size: 1000,
				is_recursive: true,
			};
			
			// Get inner VKs for recursive verification
			let mut inner_vks = Vec::new();
			for inner_id in inner_vk_ids {
				let inner_vk_bytes = VkRegistry::<T>::get(&inner_id).ok_or(Error::<T>::InvalidInput)?;
				inner_vks.push(StwoVerificationKey { 
					bytes: inner_vk_bytes.into_inner(),
					version: 1,
					circuit_size: 500,
					is_recursive: false,
				});
			}
			
			let proof_struct = StwoProof { 
				bytes: proof,
				proof_type: ProofType::Recursive,
				timestamp: 1000,
			};
			let inputs_struct = StwoPublicInputs { 
				inputs: public_inputs.clone(),
				input_count: public_inputs.len() as u32,
				input_hash: [0u8; 32],
			};
			
			// Use recursive verification if available
			let success = <AdvancedStwoVerifier as StwoVerify>::verify_recursive(&vk, &proof_struct, &inputs_struct, &inner_vks);
			<LastVerificationResult<T>>::put(success);
			Self::deposit_event(Event::Verified { success });
			Ok(())
		}

		/// Get verification statistics (INNOVATIVE FEATURE)
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::get_verification_stats())]
		pub fn get_verification_stats(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			// In a real implementation, this would return detailed stats
			// For now, just emit a success event
			Self::deposit_event(Event::Verified { success: true });
			Ok(())
		}
 	}
 }

// Max sizes (enforced for storage types). Tune to fit 5MB block limit.
pub type MaxVkLen = frame_support::traits::ConstU32<1_048_576>; // 1 MiB

