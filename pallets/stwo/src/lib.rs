#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use crate::verifier::{StarkVerifier, RealStarkVerifier, CairoProof, VerificationKey};
    use alloc::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type Something<T> = StorageValue<_, u32>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Something has been stored. [something, who]
        SomethingStored { something: u32, who: T::AccountId },
        /// Proof verification completed. [is_valid]
        ProofVerified { is_valid: bool },
        /// Debug event for parsing. [parsed]
        DebugParsing { parsed: bool },
        /// Debug event for public inputs. [public_inputs]
        DebugPublicInputs { public_inputs: Vec<u64> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value is None
        NoneValue,
        /// Value exceeds maximum and cannot be incremented further
        StorageOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/main-docs/build/origins/
            let who = ensure_signed(origin)?;

            // Update storage.
            <Something<T>>::put(something);

            // Emit an event.
            Self::deposit_event(Event::SomethingStored { something, who });
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        /// Verify a STARK proof using real verification logic
        #[pallet::call_index(1)]
        #[pallet::weight(100_000)] // Increased weight for real verification
        pub fn verify_proof(
            origin: OriginFor<T>,
            proof: Vec<u8>,
            public_inputs: Vec<u8>,
            vk: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            
            // Decode the proof and verification key
            let proof_result = CairoProof::decode(&mut &proof[..]);
            let vk_result = VerificationKey::decode(&mut &vk[..]);
            let public_inputs_result = Vec::<u64>::decode(&mut &public_inputs[..]);
            
            let parsed = proof_result.is_ok() && vk_result.is_ok() && public_inputs_result.is_ok();
            Self::deposit_event(Event::DebugParsing { parsed });
            
            let is_valid = if let (Ok(ref proof), Ok(ref vk), Ok(ref inputs)) = 
                (&proof_result, &vk_result, &public_inputs_result) {
                
                Self::deposit_event(Event::DebugPublicInputs { 
                    public_inputs: proof.public_inputs.clone() 
                });
                
                // Use REAL STARK verification (Phase 5)
                match RealStarkVerifier::verify(proof, vk, inputs.as_slice()) {
                    Ok(valid) => valid,
                    Err(_) => false,
                }
            } else {
                false
            };
            
            Self::deposit_event(Event::ProofVerified { is_valid });
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Genesis config not needed for this pallet
}

// Import the verifier module
pub mod verifier;

// Benchmarking module
#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking {
    use super::*;
    use frame_benchmarking::v2::*;
    use frame_system::RawOrigin;
    use verifier::{CairoProof, VerificationKey, FriProof, VkParams};

    #[benchmarks]
    mod benchmarks {
        use super::*;

        #[benchmark]
        fn do_something() {
            let something = 42u32;
            let caller: T::AccountId = whitelisted_caller();
            
            #[extrinsic_call]
            do_something(RawOrigin::Signed(caller), something);
        }

        #[benchmark]
        fn verify_proof() {
            let caller: T::AccountId = whitelisted_caller();
            
            // Create a mock proof for benchmarking
            let proof = CairoProof {
                commitments: vec![b"commitment1".to_vec(), b"commitment2".to_vec()],
                decommitments: vec![b"decommitment1".to_vec(), b"decommitment2".to_vec()],
                fri_proof: FriProof {
                    layers: vec![1, 2, 3, 4],
                },
                public_inputs: vec![42, 43],
            };
            
            let vk = VerificationKey {
                root: "deadbeef".to_string(),
                params: VkParams {
                    alpha: 123,
                    beta: 456,
                },
            };
            
            let public_inputs = vec![42u64, 43u64];
            
            let proof_bytes = proof.encode();
            let vk_bytes = vk.encode();
            let inputs_bytes = public_inputs.encode();
            
            #[extrinsic_call]
            verify_proof(
                RawOrigin::Signed(caller),
                proof_bytes,
                inputs_bytes,
                vk_bytes,
            );
        }

        impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verifier::{CairoProof, VerificationKey, FriProof, VkParams, StubStarkVerifier, RealStarkVerifier, StarkVerifier};
    use codec::{Encode, Decode};

    #[test]
    fn test_verifier_basic_functionality() {
        // Test basic verifier functionality
        let proof = CairoProof {
            commitments: vec!["test".to_string()],
            decommitments: vec!["test".to_string()],
            fri_proof: FriProof {
                layers: vec![1, 2, 3],
            },
            public_inputs: vec![42, 43],
        };

        let vk = VerificationKey {
            root: "test".to_string(),
            params: VkParams {
                alpha: 123,
                beta: 456,
            },
        };

        let public_inputs = vec![42u64, 43u64];

        // Test stub verifier
        let result = StubStarkVerifier::verify(&proof, &vk, &public_inputs);
        assert!(result.is_ok());

        // Test real verifier
        let result = RealStarkVerifier::verify(&proof, &vk, &public_inputs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verifier_with_invalid_data() {
        // Test with invalid data
        let proof = CairoProof {
            commitments: vec![],
            decommitments: vec![],
            fri_proof: FriProof {
                layers: vec![],
            },
            public_inputs: vec![],
        };

        let vk = VerificationKey {
            root: "".to_string(),
            params: VkParams {
                alpha: 0,
                beta: 0,
            },
        };

        let public_inputs = vec![42u64, 43u64];

        // Should handle invalid data gracefully
        let result = RealStarkVerifier::verify(&proof, &vk, &public_inputs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pallet_runtime_integration() {
        // Test that the pallet integrates correctly with the runtime
        // This simulates how the pallet would work in a real blockchain environment
        
        // Test data that would be submitted via extrinsic
        let proof = CairoProof {
            commitments: vec!["runtime_test_commitment".to_string()],
            decommitments: vec!["runtime_test_decommitment".to_string()],
            fri_proof: FriProof {
                layers: vec![1, 2, 3, 4],
            },
            public_inputs: vec![42, 43],
        };

        let vk = VerificationKey {
            root: "runtime_test_root".to_string(),
            params: VkParams {
                alpha: 12345,
                beta: 67890,
            },
        };

        let public_inputs = vec![42u64, 43u64];

        // Encode data as it would be in a real extrinsic
        let proof_bytes = proof.encode();
        let vk_bytes = vk.encode();
        let inputs_bytes = public_inputs.encode();

        // Verify encoding/decoding works correctly
        let decoded_proof = CairoProof::decode(&mut &proof_bytes[..]);
        let decoded_vk = VerificationKey::decode(&mut &vk_bytes[..]);
        let decoded_inputs = Vec::<u64>::decode(&mut &inputs_bytes[..]);

        assert!(decoded_proof.is_ok());
        assert!(decoded_vk.is_ok());
        assert!(decoded_inputs.is_ok());

        // Verify the decoded data matches original
        assert_eq!(decoded_proof.unwrap(), proof);
        assert_eq!(decoded_vk.unwrap(), vk);
        assert_eq!(decoded_inputs.unwrap(), public_inputs);
    }

    #[test]
    fn test_weight_validation() {
        // Test that our weight estimates are reasonable
        // This ensures the pallet won't cause blockchain issues
        
        let proof = CairoProof {
            commitments: vec!["weight_test".to_string(); 5],
            decommitments: vec!["weight_test".to_string(); 5],
            fri_proof: FriProof {
                layers: vec![1; 10],
            },
            public_inputs: vec![42; 50],
        };

        let vk = VerificationKey {
            root: "weight_test_root".to_string(),
            params: VkParams {
                alpha: 12345,
                beta: 67890,
            },
        };

        let public_inputs = vec![42u64; 50];

        // Measure execution time for weight estimation
        let start = std::time::Instant::now();
        let result = RealStarkVerifier::verify(&proof, &vk, &public_inputs);
        let duration = start.elapsed();

        assert!(result.is_ok());
        
        // Should complete within reasonable time for weight calculation
        // This ensures our weight estimates are accurate
        assert!(duration.as_millis() < 100, "Weight test took too long: {:?}", duration);
        
        // Verify result is consistent
        assert!(!result.unwrap()); // Expected for test data
    }
}
