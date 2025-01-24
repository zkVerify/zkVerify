//! Storage migrations.

#![cfg(not(doc))]

use frame_support::traits::UncheckedOnRuntimeUpgrade;
use hp_verifiers::Verifier;
use sp_core::Get;

use crate::Config;
use frame_support::traits::Consideration;

/// Migration from v0 to v1
pub mod v1;

/// Implements [`UncheckedOnRuntimeUpgrade`], for clean all vk storage and ticket and refound all held founds.
pub struct RemoveAllVks<T, I>(core::marker::PhantomData<(T, I)>);

impl<T: Config<I>, I: 'static> UncheckedOnRuntimeUpgrade for RemoveAllVks<T, I>
where
    I: Verifier,
{
    /// Migrate the storage from V0 to V1.
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let n = crate::Vks::<T, I>::drain().count() as u64;
        let m = crate::Tickets::<T, I>::drain()
            .map(|((account_id, vk), t)| {
                if let Some(ticket) = t {
                    if let Err(e) = ticket.drop(&account_id) {
                        log::error!(
                            "Error dropping ticket on migration ({account_id:?},{vk}): {e:?}"
                        );
                    }
                }
            })
            .count() as u64;
        T::DbWeight::get().reads_writes(n + m, n + m)
    }
}
