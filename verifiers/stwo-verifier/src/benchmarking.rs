#![cfg(feature = "runtime-benchmarks")]

use super::Stwo;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use hp_verifiers::Verifier;
use pallet_verifiers::{VkOrHash, Vks};

pub struct Pallet<T: Config>(crate::Pallet<T>);

pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Stwo<T>>;

include!("resources.rs");

#[benchmarks(where T: pallet_verifiers::Config<Stwo<T>>)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_proof() {
        let caller = whitelisted_caller();
        let vk = VALID_VK;
        let proof = VALID_PROOF;
        let pubs = VALID_PUBS;

        #[extrinsic_call]
        submit_proof(
            RawOrigin::Signed(caller),
            VkOrHash::from_vk(vk),
            proof.into(),
            pubs.into(),
        );
    }
}
