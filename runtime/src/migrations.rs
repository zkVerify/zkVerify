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

pub mod migrate_staking_to_bags_list {
    use super::*;
    use frame_election_provider_support::SortedListProvider;

    pub struct MigrateStakingToBagsList;

    #[allow(dead_code)]
    impl frame_support::traits::OnRuntimeUpgrade for MigrateStakingToBagsList {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
            use codec::Encode;

            frame_support::ensure!(
                crate::System::last_runtime_upgrade_spec_version() == 5_002,
                "must upgrade linearly"
            );

            let to_migrate = pallet_staking::Validators::<Runtime>::count()
                + pallet_staking::Nominators::<Runtime>::count();
            log::info!("👜 staking bags-list migration passes PRE migrate checks ✅");
            Ok(to_migrate.encode())
        }

        /// Migrates validators and nominators to bags list for the staking pallet.
        fn on_runtime_upgrade() -> Weight {
            // Migration intended only for the specific runtime update from version 5_002
            // (which uses pallet_staking::UseNominatorsAndValidatorsMap as VoterList)
            if crate::System::last_runtime_upgrade_spec_version() == 5_002 {
                let migrated = <Runtime as pallet_staking::Config>::VoterList::unsafe_regenerate(
                    pallet_staking::Validators::<Runtime>::iter()
                        .map(|(id, _)| id)
                        .chain(pallet_staking::Nominators::<Runtime>::iter().map(|(id, _)| id)),
                    crate::Staking::weight_of_fn(),
                );

                log::info!(
                    "👜 completed staking migration to bags list with {} voters migrated",
                    migrated,
                );

                BlockWeights::get().max_block
            } else {
                <Runtime as frame_system::Config>::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            use codec::Decode;

            <Runtime as pallet_staking::Config>::VoterList::try_state()
                .map_err(|_| "VoterList is not in a sane state.")?;
            let old_value = u32::decode(&mut &state[..]).unwrap();
            let new_value = <Runtime as pallet_staking::Config>::VoterList::count();
            frame_support::ensure!(old_value == new_value, "The voters count does not match!");
            log::info!("👜 staking bags-list migration passes POST migrate checks ✅");
            Ok(())
        }
    }
}

pub type Unreleased = ();
