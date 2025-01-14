// Copyright 2024, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! This module contains the code for all the current and past runtime migrations.

#[allow(unused_imports)]
use super::*;

mod migrate_change_vk_format {

    use super::*;

    #[allow(dead_code)]
    pub struct RemoveUltraplonkVks;

    #[cfg(feature = "try-runtime")]
    type Balances = sp_std::collections::btree_map::BTreeMap<crate::AccountId, crate::Balance>;

    impl RemoveUltraplonkVks {
        #[cfg(feature = "try-runtime")]
        fn get_all_tickets_balances() -> Balances {
            use frame_support::traits::fungible::Inspect;

            pallet_ultraplonk_verifier::Tickets::<Runtime>::iter()
                .map(|((id, _), _)| {
                    let balance = crate::Balances::balance(&id);
                    (id, balance)
                })
                .collect::<sp_std::collections::btree_map::BTreeMap<_, _>>()
        }
    }

    impl frame_support::traits::OnRuntimeUpgrade for RemoveUltraplonkVks {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
            use codec::Encode;

            frame_support::ensure!(
                crate::System::last_runtime_upgrade_spec_version() < 10_000,
                "must upgrade linearly"
            );

            Ok(Self::get_all_tickets_balances().encode())
        }

        /// Migrates validators and nominators to bags list for the staking pallet.
        fn on_runtime_upgrade() -> Weight {
            use frame_support::traits::UncheckedOnRuntimeUpgrade;
            if crate::System::last_runtime_upgrade_spec_version() >= 10_000 {
                log::warn!("Remove old migration");
                return Default::default();
            }
            let w = pallet_verifiers::migrations::RemoveAllVks::<
                Runtime,
                pallet_ultraplonk_verifier::Ultraplonk<Runtime>,
            >::on_runtime_upgrade();

            log::info!("Removed all vk from ultraplonk-verifier");

            w
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            use codec::Decode;

            let old = Balances::decode(&mut &state[..]).unwrap();

            let mut ok = true;

            for (id, old_balance) in old.iter() {
                use frame_support::traits::fungible::Inspect;
                let new_balance = crate::Balances::balance(&id);
                if old_balance >= &new_balance {
                    log::warn!("User {id:?} balance has not refunded [old] {old_balance} [new] {new_balance}");
                    ok = false;
                }
            }
            frame_support::ensure!(ok, "Some users not refunded");
            Ok(())
        }
    }
}

pub type Unreleased = ();
