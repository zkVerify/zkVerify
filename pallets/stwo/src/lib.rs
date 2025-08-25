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
mod mock {
    use super::*;
    use frame_support::construct_runtime;

    construct_runtime!(
        pub enum Test where
            Block = frame_system::mocking::MockBlock<Test>,
            NodeBlock = frame_system::mocking::MockBlock<Test>,
            UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>,
        {
            System: frame_system,
            StwoPallet: Pallet,
        }
    );

    impl frame_system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type Nonce = u64;
        type Hash = sp_core::H256;
        type Hashing = sp_runtime::traits::BlakeTwo256;
        type AccountId = u64;
        type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
        type Block = Block;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = ();
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        t.into()
    }
}
