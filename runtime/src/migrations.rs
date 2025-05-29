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

#[allow(unused_imports)]
use super::parachains::*;

use pallet_balances::WeightInfo;

parameter_types! {
        /// Weight for balance unreservations
        pub BalanceTransferAllowDeath: Weight = weights::pallet_balances::ZKVWeight::<Runtime>::transfer_allow_death();
}

pub type Unreleased = (
    pallet_aggregate::migrations::v2::MigrateV1ToV2<Runtime>,
    parachains_shared::migration::MigrateToV1<Runtime>,
    parachains_scheduler::migration::MigrateV2ToV3<Runtime>,
    pallet_child_bounties::migration::MigrateV0ToV1<Runtime, BalanceTransferAllowDeath>,
);
