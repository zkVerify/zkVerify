use frame_support::{
    traits::{ProcessMessage, ProcessMessageError},
    weights::WeightMeter,
};
use frame_system::EnsureRoot;
use polkadot_primitives::ValidatorId;
use xcm::opaque::lts::Junction;

pub use polkadot_runtime_parachains::{
    assigner_parachains as parachains_assigner_parachains, configuration, disputes,
    disputes::slashing,
    dmp as parachains_dmp, hrmp, inclusion, initializer, origin as parachains_origin, paras,
    paras_inherent, reward_points as parachains_reward_points,
    runtime_api_impl::{
        v10 as parachains_runtime_api_impl, vstaging as parachains_staging_runtime_api_impl,
    },
    scheduler as parachains_scheduler, session_info as parachains_session_info,
    shared as parachains_shared,
};

//use polkadot_runtime_common::{paras_registrar, paras_sudo_wrapper, prod_or_fast, slots};
pub use polkadot_runtime_common::paras_sudo_wrapper;

use super::{
    weights, xcm_config, AccountId, Babe, Balances, BlockWeights, Historical, KeyOwnerProofSystem,
    KeyTypeId, MaxAuthorities, MessageQueue, Offences, ParaInclusion, ParachainsAssignmentProvider,
    ParasDisputes, ParasSlashing, Perbill, ReportLongevity, Runtime, RuntimeCall, RuntimeEvent,
    RuntimeOrigin, Session, Weight,
};
use sp_runtime::transaction_validity::TransactionPriority;

use inclusion::AggregateMessageOrigin;
use sp_core::parameter_types;
use sp_runtime::FixedU128;

parameter_types! {
    pub const OnDemandTrafficDefaultValue: FixedU128 = FixedU128::from_u32(1);
}

impl parachains_assigner_parachains::Config for Runtime {}

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
    //type WeightInfo = weights::parachains::slashing::ZKVWeight<Runtime>;
    type WeightInfo = slashing::TestWeightInfo;
    type BenchmarkingConfig = slashing::BenchConfig<200>;
}

impl parachains_dmp::Config for Runtime {}

impl hrmp::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type ChannelManager = EnsureRoot<AccountId>;
    type Currency = Balances;
    type WeightInfo = weights::parachains::hrmp::ZKVWeight<Runtime>;
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
    type WeightInfo = (); //weights::parachains::inclusion::ZKVWeight<Runtime>;
}

parameter_types! {
    pub const ParasUnsignedPriority: TransactionPriority = TransactionPriority::MAX;
}

impl paras::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type UnsignedPriority = ParasUnsignedPriority;
    type QueueFootprinter = ParaInclusion;
    type NextSessionRotation = Babe;
    // type OnNewHead = Registrar;
    type OnNewHead = ();
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
    type WeightInfo = ();
    type IdleMaxServiceWeight = MessageQueueIdleServiceWeight;
}

impl pallet_authority_discovery::Config for Runtime {
    type MaxAuthorities = MaxAuthorities;
}

impl paras_sudo_wrapper::Config for Runtime {}

// parameter_types! {
//     pub const ParaDeposit: Balance = 40 * ACME;
//     pub const DataDepositPerByte: Balance = 1 * CENTS;
// }

// impl paras_registrar::Config for Runtime {
//     type RuntimeOrigin = RuntimeOrigin;
//     type RuntimeEvent = RuntimeEvent;
//     type Currency = Balances;
//     // type OnSwap = (Crowdloan, Slots);
//     type OnSwap = Slots;
//     type ParaDeposit = ParaDeposit;
//     type DataDepositPerByte = DataDepositPerByte;
//     type WeightInfo = paras_registrar::TestWeightInfo;
// }

// parameter_types! {
//     pub LeasePeriod: BlockNumber = prod_or_fast!(1 * DAYS, 1 * DAYS, "ZKV_LEASE_PERIOD");
// }

// impl slots::Config for Runtime {
//     type RuntimeEvent = RuntimeEvent;
//     type Currency = Balances;
//     type Registrar = Registrar;
//     type LeasePeriod = LeasePeriod;
//     type LeaseOffset = ();
//     //type ForceOrigin = EitherOf<EnsureRoot<Self::AccountId>, LeaseAdmin>;
//     type ForceOrigin = EnsureRoot<Self::AccountId>;
//     type WeightInfo = slots::TestWeightInfo;
// }

// /// System Parachains.
// pub mod system_parachain {
//     use xcm::latest::prelude::*;

//     // /// Network's Asset Hub parachain ID.
//     // pub const ASSET_HUB_ID: u32 = 1000;
//     // /// Contracts parachain ID.
//     // pub const CONTRACTS_ID: u32 = 1002;
//     // /// Encointer parachain ID.
//     // pub const ENCOINTER_ID: u32 = 1003;
//     // /// BridgeHub parachain ID.
//     // pub const BRIDGE_HUB_ID: u32 = 1013;

//     frame_support::match_types! {
//         pub type SystemParachains: impl Contains<MultiLocation> = {
//             MultiLocation { parents: 0, interior: X1(Parachain(1000)) }
//             // MultiLocation { parents: 0, interior: X1(Parachain(ASSET_HUB_ID | CONTRACTS_ID | ENCOINTER_ID | BRIDGE_HUB_ID)) }
//         };
//     }
// }

/// All migrations that will run on the next runtime upgrade.
///
/// This contains the combined migrations of the last 10 releases. It allows to skip runtime
/// upgrades in case governance decides to do so. THE ORDER IS IMPORTANT.
pub type Migrations = migrations::Unreleased;

pub mod migrations {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "add-parachain-upgrade")]
    pub mod add_parachain_upgrade {
        use super::*;
        pub struct AddParachainUpgrade;
        const ADD_PARACHAIN_VERSION: u32 = 4_000;
        const PARACHAIN_PARATEST_ID: u32 = 1_599;

        impl frame_support::traits::OnRuntimeUpgrade for AddParachainUpgrade {
            fn on_runtime_upgrade() -> Weight {
                if crate::System::last_runtime_upgrade_spec_version() > ADD_PARACHAIN_VERSION {
                    log::info!("Skipping add paratest parachain upgrade: already applied");
                    return <Runtime as frame_system::Config>::DbWeight::get().reads(1);
                }
                log::info!("Inject paratest parachain");
                let genesis = include_bytes!("paratest_genesis").to_vec();
                let wasm = include_bytes!("paratest_wasm").to_vec();

                let genesis = paras::GenesisConfig::<Runtime> {
                    _config: core::marker::PhantomData,
                    paras: sp_std::vec![(
                        PARACHAIN_PARATEST_ID.into(),
                        paras::ParaGenesisArgs {
                            genesis_head: genesis.into(),
                            validation_code: wasm.into(),
                            para_kind: paras::ParaKind::Parachain,
                        }
                    )],
                };
                use frame_support::traits::BuildGenesisConfig;
                genesis.build();
                sp_runtime::Perbill::from_percent(50) * crate::BlockWeights::get().max_block
            }
        }
    }

    #[cfg(feature = "add-parachain-upgrade")]
    pub type AddParachainUpgrade = add_parachain_upgrade::AddParachainUpgrade;

    #[cfg(not(feature = "add-parachain-upgrade"))]
    pub type AddParachainUpgrade = ();

    /// Unreleased migrations. Add new ones here:
    pub type Unreleased = (AddParachainUpgrade,);
}