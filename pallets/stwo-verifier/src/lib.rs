#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
pub mod weights;
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, weights::Weight};
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use sp_core::H256;
    use hp_verifiers::WeightInfo;
    
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        type MaxProofSize: Get<u32>;
        #[pallet::constant]
        type MaxKeySize: Get<u32>;
        type WeightInfo: WeightInfo<Self>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::storage]
    #[pallet::getter(fn verifying_keys)]
    pub type VerifyingKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxKeySize>,
        BoundedVec<u8, T::MaxKeySize>,
        OptionQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProofVerified { 
            who: T::AccountId,
            statement: H256,
        },
        VerifyingKeyStored { 
            key_id: BoundedVec<u8, T::MaxKeySize>,
            hash: H256,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofTooLarge,
        KeyTooLarge,
        InvalidProof,
        InvalidVerificationKey,
        VerifyingKeyNotFound,
        VerificationFailed,
    }

    impl<T: Config> Pallet<T> {
       pub fn compute_statement_hash(
            vk: &[u8],
            proof: &[u8],
            public_inputs: &[u8],
        ) -> H256 {
            let ctx = b"stwo";
            let vk_hash = sp_io::hashing::keccak_256(vk);
            let proof_hash = sp_io::hashing::keccak_256(proof);
            let inputs_hash = sp_io::hashing::keccak_256(public_inputs);

            let mut data_to_hash = ctx.to_vec();
            data_to_hash.extend_from_slice(&vk_hash);
            data_to_hash.extend_from_slice(&proof_hash);
            data_to_hash.extend_from_slice(&inputs_hash);
            
            H256(sp_io::hashing::keccak_256(&data_to_hash))
        }

        fn internal_verify_proof(
            _vk: &[u8],
            _proof: &[u8],
            _public_inputs: &[u8],
        ) -> Result<(bool, Weight), Error<T>> {
            // Simplified verification for now
            Ok((true, Weight::from_parts(10_000, 0)))
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::store_verifying_key())]
        pub fn store_verifying_key(
            origin: OriginFor<T>,
            key_id: Vec<u8>,
            key: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let bounded_key_id: BoundedVec<u8, T::MaxKeySize> = key_id
                .try_into()
                .map_err(|_| Error::<T>::KeyTooLarge)?;

            // Calculate hash before converting key to BoundedVec
            let hash = H256(sp_io::hashing::keccak_256(&key));
            
            let bounded_key: BoundedVec<u8, T::MaxKeySize> = key
                .try_into()
                .map_err(|_| Error::<T>::KeyTooLarge)?;
            
            <VerifyingKeys<T>>::insert(&bounded_key_id, bounded_key);
            Self::deposit_event(Event::VerifyingKeyStored { 
                key_id: bounded_key_id,
                hash,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::verify_proof(&proof, &public_inputs))]
        pub fn verify_proof(
            origin: OriginFor<T>,
            key_id: Vec<u8>,
            proof: Vec<u8>,
            public_inputs: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(proof.len() <= T::MaxProofSize::get() as usize, Error::<T>::ProofTooLarge);
            
            let bounded_key_id: BoundedVec<u8, T::MaxKeySize> = key_id
                .try_into()
                .map_err(|_| Error::<T>::KeyTooLarge)?;

            let vk = <VerifyingKeys<T>>::get(&bounded_key_id)
                .ok_or(Error::<T>::VerifyingKeyNotFound)?;
            
            let (verification_result, actual_weight) = Self::internal_verify_proof(&vk, &proof, &public_inputs)?;
            ensure!(verification_result, Error::<T>::InvalidProof);
            
            let statement = Self::compute_statement_hash(&vk, &proof, &public_inputs);
            
            Self::deposit_event(Event::ProofVerified { 
                who,
                statement,
            });
            
            Ok(Some(actual_weight).into())
        }
    }
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
