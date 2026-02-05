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

/// Migrations for the polkadot-stable2512 upgrade.
///
/// These migrations handle:
/// 1. pallet-staking v15 → v16: Migrates `DisabledValidators` from `Vec<u32>` to
///    `Vec<(u32, OffenceSeverity)>` to track offense severity for re-enabling purposes.
/// 2. pallet-session v0 → v1: Migrates disabled validators storage to the new format.
///    Must run AFTER staking v16 migration.
///
/// Note: The staking locks-to-holds migration is handled internally by the pallet
/// when processing ledger updates, not via explicit storage migration.
pub type PolkadotSdk2512 = (
    // Staking v15 → v16: DisabledValidators format change
    pallet_staking::migrations::v16::MigrateV15ToV16<crate::Runtime>,
    // Session v0 → v1: Must run after staking v16 migration
    pallet_session::migrations::v1::MigrateV0ToV1<
        crate::Runtime,
        pallet_staking::migrations::v17::MigrateDisabledToSession<crate::Runtime>,
    >,
);

pub type Unreleased = (
    pallet_aggregate::migrations::v4::MigrateV3ToV4<crate::Runtime>,
    PolkadotSdk2512,
);
