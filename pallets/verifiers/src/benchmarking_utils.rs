#![cfg(feature = "runtime-benchmarks")]

//! Some useful utils for benchmarking the verifier pallet.

use crate::{Config, VkEntry};
use frame_benchmarking::whitelisted_caller;
use frame_support::traits::{Consideration, Footprint};
pub use hp_verifiers::{Verifier, VerifyError};
use sp_core::H256;

pub use pallet_verifiers_macros::benchmarking_utils;

/// Return a whitelisted account with enough founds to do anything.
pub fn funded_account<T, I>() -> T::AccountId
where
    T: Config<I>,
    I: 'static + Verifier,
{
    use frame_support::sp_runtime::traits::Bounded;
    use frame_support::traits::fungible::Inspect;
    type BalanceOf<T, I> =
        <<T as Config<I>>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    use frame_support::traits::fungible::Mutate;
    let caller: T::AccountId = whitelisted_caller();
    T::Currency::set_balance(&caller, BalanceOf::<T, I>::max_value() / 2u32.into());
    caller
}

/// Insert a valid vk into the Vks storage and related ticket.
pub fn insert_vk<T, I>(owner: T::AccountId, vk: I::Vk, hash: H256)
where
    T: Config<I>,
    I: 'static + Verifier,
{
    let vk_entry = VkEntry::new(vk);
    let footprint = Footprint::from_encodable(&vk_entry);
    let ticket = T::Ticket::new(&owner, footprint).unwrap();

    crate::Vks::<T, I>::insert(hash, vk_entry);
    crate::Tickets::<T, I>::insert((owner, hash), ticket);
}

/// Insert a valid vk into the Vks storage and related ticket.
pub fn insert_vk_anonymous<T, I>(vk: I::Vk, hash: H256)
where
    T: Config<I>,
    I: 'static + Verifier,
{
    insert_vk::<T, I>(funded_account::<T, I>(), vk, hash);
}
