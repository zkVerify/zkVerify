#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub use pallet::*;

pub mod verifier;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, dispatch::DispatchResult};
    use frame_system::pallet_prelude::*;
    use alloc::vec::Vec;
    use crate::verifier::{CairoProof, VerificationKey, RealStarkVerifier, StarkVerifier};
    use codec::Decode;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProofVerified { is_valid: bool },
        DebugParsing { parsed: bool },
        DebugPublicInputs { public_inputs: Vec<u64> },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn verify_proof(
            origin: OriginFor<T>,
            proof: Vec<u8>,
            _public_inputs: Vec<u8>,
            vk: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Decode proof and vk using SCALE
            let proof_result = CairoProof::decode(&mut &proof[..]);
            let vk_result = VerificationKey::decode(&mut &vk[..]);
            let parsed = proof_result.is_ok() && vk_result.is_ok();
            Self::deposit_event(Event::DebugParsing { parsed });
            let is_valid = if let (Ok(ref proof), Ok(ref vk)) = (&proof_result, &vk_result) {
                Self::deposit_event(Event::DebugPublicInputs { public_inputs: proof.public_inputs.clone() });
                // Use the real verifier (swap to StubStarkVerifier for mock/testing)
                match RealStarkVerifier::verify(proof, vk, &proof.public_inputs) {
                    Ok(valid) => valid,
                    Err(e) => {
                        // Emit an error event or log if desired
                        log::warn!("Verification error: {}", e);
                        false
                    }
                }
            } else {
                false
            };
            Self::deposit_event(Event::ProofVerified { is_valid });

            Ok(())
        }
    }
}

// --- Note ---
// All STARK/Cairo verification logic and data structures are now in verifier.rs.

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_mock_stark_proof_data() {
        let proof_data = fs::read_to_string("test_data/proof.json").unwrap();
        let public_inputs_data = fs::read_to_string("test_data/public_inputs.json").unwrap();
        let vk_data = fs::read_to_string("test_data/verification_key.json").unwrap();

        // For now, just verify the files can be read
        assert!(!proof_data.is_empty());
        assert!(!public_inputs_data.is_empty());
        assert!(!vk_data.is_empty());
    }
}
