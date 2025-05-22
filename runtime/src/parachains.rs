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

//! Parachain modules configurations.
//! FIXME: this configuration is meant for testing only, and MUST not deployed to a production
//! network without proper assessment.

use frame_support::{
    traits::{ProcessMessage, ProcessMessageError},
    weights::WeightMeter,
};
use frame_system::EnsureRoot;
use polkadot_primitives::ValidatorId;
use xcm::latest::Junction;

pub use polkadot_runtime_parachains::{
    assigner_coretime as parachains_assigner_coretime, configuration,
    configuration::ActiveConfigHrmpChannelSizeAndCapacityRatio,
    disputes,
    disputes::slashing,
    dmp as parachains_dmp, hrmp, inclusion, initializer, on_demand, origin as parachains_origin,
    paras, paras_inherent, reward_points as parachains_reward_points,
    runtime_api_impl::{
        v11 as parachains_runtime_api_impl, vstaging as parachains_staging_runtime_api_impl,
    },
    scheduler as parachains_scheduler, session_info as parachains_session_info,
    shared as parachains_shared,
};

pub use polkadot_runtime_common::{paras_registrar, paras_sudo_wrapper, slots, traits::Registrar};

use super::{
    currency::Balance, weights, xcm_config, AccountId, Babe, Balances, BlockNumber, BlockWeights,
    Historical, KeyOwnerProofSystem, KeyTypeId, MaxAuthorities, MessageQueue, Offences,
    ParaInclusion, ParachainsAssignmentProvider, ParasDisputes, ParasSlashing, Perbill,
    ReportLongevity, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, Session, Weight,
};
use sp_runtime::transaction_validity::TransactionPriority;

use crate::XcmPallet;
use inclusion::AggregateMessageOrigin;
use sp_core::parameter_types;
use sp_runtime::{FixedU128, Percent};

#[cfg(feature = "fast-runtime")]
pub const TIMESLICE_PERIOD: u32 = 20;
#[cfg(not(feature = "fast-runtime"))]
pub const TIMESLICE_PERIOD: u32 = 80;

parameter_types! {
    pub const OnDemandTrafficDefaultValue: FixedU128 = FixedU128::from_u32(1);
    // Keep 2 timeslices worth of revenue information.
    pub const MaxHistoricalRevenue: BlockNumber = 2 * TIMESLICE_PERIOD;
    pub const OnDemandPalletId: crate::PalletId = crate::PalletId(*b"zk/ondmd");
}

impl on_demand::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type TrafficDefaultValue = OnDemandTrafficDefaultValue;
    type WeightInfo = weights::parachains::on_demand::ZKVWeight<Runtime>;
    type MaxHistoricalRevenue = MaxHistoricalRevenue;
    type PalletId = OnDemandPalletId;
}

impl parachains_assigner_coretime::Config for Runtime {}

impl initializer::Config for Runtime {
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
    type ForceOrigin = EnsureRoot<AccountId>;
    type WeightInfo = weights::parachains::initializer::ZKVWeight<Runtime>;

    type CoretimeOnNewSession = ();
}

impl disputes::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RewardValidators = parachains_reward_points::RewardValidatorsWithEraPoints<Runtime>;
    type SlashingHandler = slashing::SlashValidatorsForDisputes<ParasSlashing>;
    type WeightInfo = weights::parachains::disputes::ZKVWeight<Runtime>;
}

impl slashing::Config for Runtime {
    type KeyOwnerProofSystem = Historical;
    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, ValidatorId)>>::Proof;
    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        ValidatorId,
    )>>::IdentificationTuple;
    type HandleReports =
        slashing::SlashingReportHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
    type WeightInfo = weights::parachains::slashing::ZKVWeight<Runtime>;
    type BenchmarkingConfig = slashing::BenchConfig<{ crate::MAX_ACTIVE_VALIDATORS }>;
}

impl parachains_dmp::Config for Runtime {}

parameter_types! {
        pub const HrmpChannelSizeAndCapacityWithSystemRatio: Percent = Percent::from_percent(100);
}

impl hrmp::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type ChannelManager = EnsureRoot<AccountId>;
    type Currency = Balances;
    type WeightInfo = weights::parachains::hrmp::ZKVWeight<Runtime>;
    // Use the `HrmpChannelSizeAndCapacityWithSystemRatio` ratio from the actual active
    // `HostConfiguration` configuration for `hrmp_channel_max_message_size` and
    // `hrmp_channel_max_capacity`.
    type DefaultChannelSizeAndCapacityWithSystem = ActiveConfigHrmpChannelSizeAndCapacityRatio<
        Runtime,
        HrmpChannelSizeAndCapacityWithSystemRatio,
    >;
    type VersionWrapper = XcmPallet;
}

impl paras_inherent::Config for Runtime {
    type WeightInfo = weights::parachains::paras_inherent::ZKVWeight<Runtime>;
}

impl parachains_scheduler::Config for Runtime {
    type AssignmentProvider = ParachainsAssignmentProvider;
}

impl parachains_origin::Config for Runtime {}

impl configuration::Config for Runtime {
    type WeightInfo = weights::parachains::configuration::ZKVWeight<Runtime>;
}

impl parachains_shared::Config for Runtime {
    type DisabledValidators = Session;
}

impl parachains_session_info::Config for Runtime {
    type ValidatorSet = Historical;
}

impl inclusion::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type DisputesHandler = ParasDisputes;
    type RewardValidators = parachains_reward_points::RewardValidatorsWithEraPoints<Runtime>;
    type MessageQueue = MessageQueue;
    type WeightInfo = weights::parachains::inclusion::ZKVWeight<Runtime>;
}

parameter_types! {
    pub const ParasUnsignedPriority: TransactionPriority = TransactionPriority::MAX;
}

impl paras::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type UnsignedPriority = ParasUnsignedPriority;
    type QueueFootprinter = ParaInclusion;
    type NextSessionRotation = Babe;
    type OnNewHead = crate::Registrar;
    type AssignCoretime = ();
    type WeightInfo = weights::parachains::paras::ZKVWeight<Runtime>;
}

parameter_types! {
    /// Amount of weight that can be spent per block to service messages.
    ///
    /// # WARNING
    ///
    /// This is not a good value for para-chains since the `Scheduler` already uses up to 80% block weight.
    pub MessageQueueServiceWeight: Weight = Perbill::from_percent(20) * BlockWeights::get().max_block;
    pub MessageQueueIdleServiceWeight: Weight = Perbill::from_percent(20) * BlockWeights::get().max_block;
    pub const MessageQueueHeapSize: u32 = 32 * 1024;
    pub const MessageQueueMaxStale: u32 = 96;
}

/// Message processor to handle any messages that were enqueued into the `MessageQueue` pallet.
pub struct MessageProcessor;

impl ProcessMessage for MessageProcessor {
    type Origin = AggregateMessageOrigin;

    fn process_message(
        message: &[u8],
        origin: Self::Origin,
        meter: &mut WeightMeter,
        id: &mut [u8; 32],
    ) -> Result<bool, ProcessMessageError> {
        let para = match origin {
            AggregateMessageOrigin::Ump(inclusion::UmpQueueId::Para(para)) => para,
        };
        xcm_builder::ProcessXcmMessage::<
            Junction,
            xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
            RuntimeCall,
        >::process_message(message, Junction::Parachain(para.into()), meter, id)
    }
}

impl pallet_message_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Size = u32;
    type HeapSize = MessageQueueHeapSize;
    type MaxStale = MessageQueueMaxStale;
    type ServiceWeight = MessageQueueServiceWeight;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type MessageProcessor = MessageProcessor;
    #[cfg(feature = "runtime-benchmarks")]
    type MessageProcessor =
        pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
    type QueueChangeHandler = ParaInclusion;
    type QueuePausedQuery = ();
    type WeightInfo = crate::weights::pallet_message_queue::ZKVWeight<Runtime>;
    type IdleMaxServiceWeight = MessageQueueIdleServiceWeight;
}

impl pallet_authority_discovery::Config for Runtime {
    type MaxAuthorities = MaxAuthorities;
}

impl paras_sudo_wrapper::Config for Runtime {}

parameter_types! {
    pub const ParaDeposit: Balance = Balance::MAX; // deliberately high
    pub const DataDepositPerByte: Balance = Balance::MAX; // deliberately high
}

impl paras_registrar::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type OnSwap = ();
    type ParaDeposit = ParaDeposit;
    type DataDepositPerByte = DataDepositPerByte;
    type WeightInfo = weights::parachains::registrar::ZKVWeight<Runtime>;
}

parameter_types! {
    pub LeasePeriod: BlockNumber = BlockNumber::MAX; // deliberately high
}

impl slots::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Registrar = crate::Registrar;
    type LeasePeriod = LeasePeriod;
    type LeaseOffset = ();
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = weights::parachains::slots::ZKVWeight<Runtime>;
}

/// All migrations that will run on the next runtime upgrade.
///
/// This contains the combined migrations of the last 10 releases. It allows to skip runtime
/// upgrades in case governance decides to do so. THE ORDER IS IMPORTANT.
pub type Migrations = migrations::Unreleased;

pub mod migrations {
    #[allow(unused_imports)]
    use super::*;

    /// Unreleased migrations. Add new ones here:
    pub type Unreleased = ();
}
