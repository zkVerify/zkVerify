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

pub struct VoidMigrate<T, I>(core::marker::PhantomData<(T, I)>);

impl<T, I> frame_support::traits::UncheckedOnRuntimeUpgrade for VoidMigrate<T, I>
where
    T: pallet_verifiers::Config<I>,
    I: hp_verifiers::Verifier + 'static,
{
}

pub type VoidMigration<const FROM: u16, const TO: u16, R, V, P> =
    frame_support::migrations::VersionedMigration<
        FROM,
        TO,
        VoidMigrate<R, V>,
        P,
        <R as frame_system::Config>::DbWeight,
    >;

pub type Unreleased = (
    pallet_aggregate::migrations::v4::MigrateV3ToV4<crate::Runtime>,
    pallet_ultrahonk_verifier::migrations::MigrateV1ToV2<crate::Runtime>,
    VoidMigration<
        1,
        2,
        crate::Runtime,
        pallet_groth16_verifier::Groth16<crate::Runtime>,
        crate::SettlementGroth16Pallet,
    >,
    VoidMigration<
        1,
        2,
        crate::Runtime,
        pallet_ultraplonk_verifier::Ultraplonk<crate::Runtime>,
        crate::SettlementUltraplonkPallet,
    >,
    VoidMigration<
        1,
        2,
        crate::Runtime,
        pallet_risc0_verifier::Risc0<crate::Runtime>,
        crate::SettlementRisc0Pallet,
    >,
    VoidMigration<
        1,
        2,
        crate::Runtime,
        pallet_sp1_verifier::Sp1<crate::Runtime>,
        crate::SettlementSp1Pallet,
    >,
    VoidMigration<
        1,
        2,
        crate::Runtime,
        pallet_ezkl_verifier::Ezkl<crate::Runtime>,
        crate::SettlementEzklPallet,
    >,
    VoidMigration<
        1,
        2,
        crate::Runtime,
        pallet_tee_verifier::Tee<crate::Runtime>,
        crate::SettlementTeePallet,
    >,
    VoidMigration<
        1,
        2,
        crate::Runtime,
        pallet_fflonk_verifier::Fflonk,
        crate::SettlementFFlonkPallet,
    >,
    VoidMigration<
        1,
        2,
        crate::Runtime,
        pallet_plonky2_verifier::Plonky2<crate::Runtime>,
        crate::SettlementPlonky2Pallet,
    >,
    (),
);
