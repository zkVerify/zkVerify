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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 512 (for relay chain).
#![recursion_limit = "512"]
#![allow(clippy::identity_op)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;

use alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    vec::Vec,
};

use authority_discovery_primitives::AuthorityId as AuthorityDiscoveryId;
use codec::MaxEncodedLen;
use pallet_babe::AuthorityId as BabeId;
use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, Get, OpaqueMetadata, H256};
use sp_runtime::{
    generic, impl_opaque_keys,
    traits::{
        AccountIdConversion, BlakeTwo256, Block as BlockT, Bounded, ConvertInto, IdentityLookup,
        NumberFor, One, OpaqueKeys,
    },
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, FixedPointNumber, Perquintill,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_election_provider_support::{
    bounds::{ElectionBounds, ElectionBoundsBuilder},
    onchain,
    onchain::OnChainExecution,
    SequentialPhragmen,
};

use frame_support::genesis_builder_helper::{build_state, get_preset};

use frame_support::dispatch::DispatchResult;
pub use frame_support::{
    construct_runtime, derive_impl,
    dispatch::DispatchClass,
    parameter_types,
    traits::{
        fungible::HoldConsideration,
        tokens::{PayFromAccount, UnityAssetBalanceConversion},
        ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, EitherOfDiverse, EqualPrivilegeOnly,
        KeyOwnerProofSystem, LinearStoragePrice, Randomness, StorageInfo, Time, WithdrawReasons,
    },
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        ConstantMultiplier, IdentityFee, Weight,
    },
    Blake2_128Concat, Identity, PalletId, StorageHasher, StorageValue,
};
pub use frame_system::Call as SystemCall;
use frame_system::EnsureRoot;
use ismp::consensus::{ConsensusClientId, StateMachineHeight, StateMachineId};
use ismp::host::StateMachine;
use ismp::module::IsmpModule;
use ismp::router::{IsmpRouter, Request, Response};
use ismp::Error;
pub use pallet_balances::Call as BalancesCall;
use pallet_hyperbridge::PALLET_HYPERBRIDGE_ID;
use pallet_session::historical as pallet_session_historical;
pub use pallet_timestamp::Call as TimestampCall;
use static_assertions::const_assert;
use weights::block_weights::BlockExecutionWeight;
use weights::extrinsic_weights::ExtrinsicBaseWeight;

use pallet_transaction_payment::{FungibleAdapter, Multiplier, TargetedFeeAdjustment};
use pallet_treasury::TreasuryAccountId;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{MultiSignature, MultiSigner, Perbill, Permill};

pub mod governance;
mod pallet_assets_mock;
use governance::{pallet_custom_origins, TreasurySpender};
pub use zkv_runtime_common::*;

pub mod parachains;
pub mod xcm_config;

mod bag_thresholds;
mod genesis_config_presets;
mod migrations;
mod payout;
mod proxy;
#[cfg(test)]
mod tests;
mod weights;

pub(crate) mod weight_aliases {
    pub mod pallet_plonky2_verifier_verify_proof {
        pub use pallet_plonky2_verifier::WeightInfoVerifyProof as WeightInfo;
    }

    pub mod pallet_risc0_verifier_verify_proof {
        pub use pallet_risc0_verifier::WeightInfoVerifyProof as WeightInfo;
    }

    pub mod frame_system_extensions {
        pub use frame_system::ExtensionsWeightInfo as WeightInfo;
    }
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("tzkv-runtime"),
    impl_name: Cow::Borrowed("zkv-node"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    spec_version: 1_001_000,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

// 1 in 4 blocks will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time, with maximum proof size.
const MAXIMUM_BLOCK_WEIGHT: Weight =
    Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 4096;
    pub const Version: RuntimeVersion = VERSION;

    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();

    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);

    // ASCII for 'Z'+'K'+'V'
    pub const SS58Prefix: u8 = 251;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
    /// The block type for the runtime.
    type Block = Block;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The type for storing how many extrinsics an account has signed.
    type Nonce = Nonce;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = weights::db::constants::RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type SystemWeightInfo = weights::frame_system::ZKVWeight<Runtime>;
    type ExtensionsWeightInfo = weights::frame_system_extensions::ZKVWeight<Runtime>;
}

parameter_types! {
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK; // Should use primitives::Moment
    pub EpochDurationInBlocks: BlockNumber = prod_or_fast!(1 * HOURS, 1 * MINUTES, "ZKV_RELAY_EPOCH_DURATION");

    /// How long (in blocks) an equivocation report is valid for
    pub ReportLongevity: u64 = EpochDurationInBlocks::get() as u64 * 10;
    /// How many authorities BABE and GRANDPA have storage for
    pub const MaxAuthorities: u32 = MaxActiveValidators::get();
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDurationInBlocks;
    type ExpectedBlockTime = ExpectedBlockTime;
    // session module is the trigger
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = Session;
    type WeightInfo = weights::pallet_babe::ZKVWeight<Runtime>;
    type MaxAuthorities = MaxAuthorities;
    type MaxNominators = ConstU32<MAX_VOTERS>;
    type KeyOwnerProof = sp_session::MembershipProof;
    type EquivocationReportSystem =
        pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

parameter_types! {
    pub const BagThresholds: &'static [u64] = &bag_thresholds::THRESHOLDS;
}

type VoterBagsListInstance = pallet_bags_list::Instance1;
impl pallet_bags_list::Config<VoterBagsListInstance> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ScoreProvider = Staking;
    type WeightInfo = weights::pallet_bags_list::ZKVWeight<Runtime>;
    type BagThresholds = BagThresholds;
    type Score = sp_npos_elections::VoteWeight;
}

parameter_types! {
    pub MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type WeightInfo = weights::pallet_grandpa::ZKVWeight<Runtime>;
    type MaxAuthorities = MaxAuthorities;
    type MaxNominators = ConstU32<MAX_VOTERS>;
    type MaxSetIdSessionEntries = MaxSetIdSessionEntries;

    type KeyOwnerProof = sp_session::MembershipProof;
    type EquivocationReportSystem =
        pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = weights::pallet_utility::ZKVWeight<Runtime>;
}

parameter_types! {
    pub const MinVestedTransfer: Balance = 100 * CENTS;
    pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
        WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type BlockNumberToBalance = ConvertInto;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = weights::pallet_vesting::ZKVWeight<Runtime>;
    type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
    type BlockNumberProvider = System;
    const MAX_VESTING_SCHEDULES: u32 = 28;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>; // this is a Babe assumption
    type WeightInfo = weights::pallet_timestamp::ZKVWeight<Runtime>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
    pub const MaxFreezes: u32 = 8;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = weights::pallet_balances::ZKVWeight<Runtime>;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = MaxFreezes;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub babe: Babe,
        pub grandpa: Grandpa,
        pub para_validator: Initializer,
        pub para_assignment: ParaSessionInfo,
        pub authority_discovery: AuthorityDiscovery,
    }
}

parameter_types! {
    pub const TransactionPicosecondFee: Balance = 5000000;
    pub const TransactionByteFee: Balance = 5000000;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(75);
    // AdjustmentVariable computed to result in a desired cost for filling n blocks in a row. See
    // block_cost_after_k_full_blocks test for more info.
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1313646132342424i64, 10000000000000000i64);
    pub MinimumMultiplier: Multiplier = Multiplier::one();
    pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

pub type ZKVFeeUpdate<R> = TargetedFeeAdjustment<
    R,
    TargetBlockFullness,
    AdjustmentVariable,
    MinimumMultiplier,
    MaximumMultiplier,
>;

/// How to handle with fee: Don't burn any fee, give all fee and tip to author.
pub type DealWithFees = zkv_runtime_common::DealWithFees<
    Runtime,
    payout::NoBurnFees,
    payout::AllFeesToAuthor,
    Authorship,
    TreasuryAccountId<Runtime>,
>;

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, DealWithFees>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = ConstantMultiplier<Balance, TransactionPicosecondFee>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = ZKVFeeUpdate<Self>;
    type WeightInfo = weights::pallet_transaction_payment::ZKVWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = weights::pallet_sudo::ZKVWeight<Runtime>;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const MultisigDepositBase: Balance = currency::deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const MultisigDepositFactor: Balance = currency::deposit(0, 32);
    pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = MultisigDepositBase;
    type DepositFactor = MultisigDepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = weights::pallet_multisig::ZKVWeight<Runtime>;
}

parameter_types! {
    pub const PreimageBaseDeposit: Balance = deposit(2, 64);
    pub const PreimageByteDeposit: Balance = deposit(0, 1);
    pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = weights::pallet_preimage::ZKVWeight<Runtime>;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Consideration = frame_support::traits::fungible::HoldConsideration<
        AccountId,
        Balances,
        PreimageHoldReason,
        frame_support::traits::LinearStoragePrice<
            PreimageBaseDeposit,
            PreimageByteDeposit,
            Balance,
        >,
    >;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
    pub MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;

    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = weights::pallet_scheduler::ZKVWeight<Runtime>;
    type Preimages = Preimage;
}

parameter_types! {
    pub const TreasuryPalletId: PalletId = PalletId(*b"zk/trsry");
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 2000 * CENTS;
    pub const ProposalBondMaximum: Balance = THOUSANDS;
    pub const SpendPeriod: BlockNumber = 6 * DAYS;
    pub const Burn: Permill = Permill::from_percent(0);
    pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;
    pub const MaxApprovals: u32 = 100;
    pub ZKVerifyTreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type RejectOrigin = EnsureRoot<AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    type MaxApprovals = MaxApprovals;
    type WeightInfo = weights::pallet_treasury::ZKVWeight<Runtime>;
    type SpendFunds = Bounties;
    type SpendOrigin = TreasurySpender;
    type AssetKind = ();
    type Beneficiary = AccountId;
    type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
    type Paymaster = PayFromAccount<Balances, ZKVerifyTreasuryAccount>;
    type BalanceConverter = UnityAssetBalanceConversion;
    type PayoutPeriod = PayoutSpendPeriod;
    type BlockNumberProvider = System;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

parameter_types! {
    pub const BountyDepositBase: Balance = VFY;
    pub const BountyDepositPayoutDelay: BlockNumber = 8 * DAYS;
    pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
    pub const MaximumReasonLength: u32 = 16384;
    pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
    pub const CuratorDepositMin: Balance = 10 * VFY;
    pub const CuratorDepositMax: Balance = 200 * VFY;
    pub const BountyValueMinimum: Balance = 10 * VFY;
    pub DataDepositPerByte: Balance = deposit(0, 1);
}
impl pallet_bounties::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type CuratorDepositMultiplier = CuratorDepositMultiplier;
    type CuratorDepositMin = CuratorDepositMin;
    type CuratorDepositMax = CuratorDepositMax;
    type BountyValueMinimum = BountyValueMinimum;
    type ChildBountyManager = ChildBounties;
    type DataDepositPerByte = DataDepositPerByte;
    type MaximumReasonLength = MaximumReasonLength;
    type WeightInfo = weights::pallet_bounties::ZKVWeight<Runtime>;
    type OnSlash = Treasury;
}

parameter_types! {
    pub const MaxActiveChildBountyCount: u32 = 100;
    pub const ChildBountyValueMinimum: Balance = BountyValueMinimum::get() / 10;
}

impl pallet_child_bounties::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxActiveChildBountyCount = MaxActiveChildBountyCount;
    type ChildBountyValueMinimum = ChildBountyValueMinimum;
    type WeightInfo = weights::pallet_child_bounties::ZKVWeight<Runtime>;
}

parameter_types! {
    pub const AggregateBaseDeposit: Balance = deposit(2, 64);
    pub const AggregateByteDeposit: Balance = deposit(0, 1);
    pub const AggregateRegisterHoldReason: RuntimeHoldReason = RuntimeHoldReason::Aggregate(pallet_aggregate::HoldReason::Domain);
    pub const AggregateBaseTip: Balance = 10 * CENTS;
    pub const AggregateLinearTip: Permill = Permill::from_percent(10);
    pub const AggregateMaxSize: pallet_aggregate::AggregationSize = 128;
    pub const AggregateQueueSize: u32 = 16;
}

/// Linear increment.
pub struct Linear<Base, Slope, Balance>(core::marker::PhantomData<(Base, Slope, Balance)>);
impl<Base, Slope> pallet_aggregate::ComputePublisherTip<Balance> for Linear<Base, Slope, Balance>
where
    Base: Get<Balance>,
    Slope: Get<Permill>,
{
    fn compute_tip(estimated: Balance) -> Option<Balance> {
        Base::get()
            .saturating_add(Slope::get().mul_floor(estimated))
            .into()
    }
}

impl DispatchAggregation<Balance, AccountId> for Runtime {
    fn dispatch_aggregation(
        domain_id: u32,
        aggregation_id: u64,
        aggregation: H256,
        destination_params: Destination,
        fee: Balance,
        delivery_owner: AccountId,
    ) -> DispatchResult {
        match destination_params {
            Destination::None => Ok(()),
            Destination::Hyperbridge(params) => {
                pallet_hyperbridge_aggregations::Pallet::<Runtime>::dispatch_aggregation(
                    delivery_owner,
                    Params {
                        domain_id,
                        aggregation_id,
                        aggregation,
                        module: params.destination_module,
                        destination: StateMachine::from(params.destination_chain),
                        timeout: params.timeout,
                        fee,
                    },
                )
            }
        }
    }

    fn max_weight() -> Weight {
        use pallet_hyperbridge_aggregations::WeightInfo;
        <Runtime as pallet_hyperbridge_aggregations::Config>::WeightInfo::dispatch_aggregation()
    }

    fn dispatch_weight(destination: &Destination) -> Weight {
        match destination {
            Destination::None => Default::default(),
            Destination::Hyperbridge(_) => {
                use pallet_hyperbridge_aggregations::WeightInfo;
                <Runtime as pallet_hyperbridge_aggregations::Config>::WeightInfo::dispatch_aggregation()
            }
        }
    }
}

impl pallet_aggregate::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type AggregationSize = AggregateMaxSize;
    type MaxPendingPublishQueueSize = AggregateQueueSize;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Hold = Balances;

    type Consideration = frame_support::traits::fungible::HoldConsideration<
        AccountId,
        Balances,
        AggregateRegisterHoldReason,
        frame_support::traits::LinearStoragePrice<
            AggregateBaseDeposit,
            AggregateByteDeposit,
            Balance,
        >,
    >;
    type EstimateCallFee = TransactionPayment;

    type ComputePublisherTip = Linear<AggregateBaseTip, AggregateLinearTip, Balance>;

    type WeightInfo = weights::pallet_aggregate::ZKVWeight<Runtime>;

    #[cfg(feature = "runtime-benchmarks")]
    const AGGREGATION_SIZE: u32 = AggregateMaxSize::get() as u32;

    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;

    type DispatchAggregation = Self;
}

parameter_types! {
    pub const ClaimPalletId: PalletId = PalletId(*b"zkv/pclm");
    pub const MaxBeneficiaries: u32 = 1_000_000;
    pub const MaxOpBeneficiaries: u32 = 100_000;
}

impl pallet_claim::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ClaimPalletId;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Currency = Balances;
    type UnclaimedDestination = ZKVerifyTreasuryAccount;
    type WeightInfo = weights::pallet_claim::ZKVWeight<Runtime>;
    type MaxBeneficiaries = MaxBeneficiaries;
    const MAX_OP_BENEFICIARIES: u32 = MaxOpBeneficiaries::get();
}

parameter_types! {
    pub const TokenClaimPalletId: PalletId = PalletId(*b"zkv/ptkc");
    pub const MaxClaimMessageLength: u32 = 500;
}

impl pallet_token_claim::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = TokenClaimPalletId;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Currency = Balances;
    type UnclaimedDestination = ZKVerifyTreasuryAccount;
    type WeightInfo = weights::pallet_token_claim::ZKVWeight<Runtime>;
    type Signer = MultiSigner;
    type Signature = MultiSignature;
    type MaxBeneficiaries = MaxBeneficiaries;
    type MaxClaimMessageLength = MaxClaimMessageLength;
    type AccountIdBytesToSign = pallet_token_claim::AccountId32ToSs58BytesToSign;
    const MAX_OP_BENEFICIARIES: u32 = MaxOpBeneficiaries::get();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

// We should be sure that the benchmark aggregation size matches the runtime configuration.
#[cfg(feature = "runtime-benchmarks")]
static_assertions::const_assert!(
    <Runtime as pallet_aggregate::Config>::AggregationSize::get() as u32
        == <Runtime as pallet_aggregate::Config>::AGGREGATION_SIZE,
);

pub struct ValidatorIdOf;
impl sp_runtime::traits::Convert<AccountId, Option<AccountId>> for ValidatorIdOf {
    fn convert(a: AccountId) -> Option<AccountId> {
        Some(a)
    }
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = AccountId;
    type ValidatorIdOf = ValidatorIdOf;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type WeightInfo = weights::pallet_session::ZKVWeight<Runtime>;
}

parameter_types! {
    pub SessionsPerEra: sp_staking::SessionIndex = 6 * HOURS / EpochDurationInBlocks::get(); // number of sessions in 1 era, 6h

    pub const BondingDuration: sp_staking::EraIndex = 28; // number of sessions for which staking
                                                         // remains locked
    pub const SlashDeferDuration: sp_staking::EraIndex = 24; // eras to wait before slashing is
                                                            // applied
    pub HistoryDepth: u32 = 30; // Number of eras to keep in history. Older eras cannot be claimed.
}

// Maximum number of election targets (eligible authorities) to account for. The staking pallet
// can never have more validators than this.
pub const MAX_TARGETS: u32 = 1_000;
// Maximum number of voters. This also includes targets, which implicitly vote for themselves.
pub const MAX_VOTERS: u32 = 5_000;
// The maximum number of number of active validators that we want to handle.
// This *must always be greater or equal* to staking.validatorCount storage value.
pub const MAX_ACTIVE_VALIDATORS: u32 = 20;

parameter_types! {
    // Maximum number of election voters and targets that can be handled by OnChainSeqPhragmen
    // in a single block time.
    pub ElectionBoundsOnChain: ElectionBounds = ElectionBoundsBuilder::default().voters_count((MAX_VOTERS).into()).targets_count(MAX_TARGETS.into()).build();
    // Maximum number of election winners, and thus of authorities that can be active in a given
    // era.
    pub const MaxActiveValidators: u32 = MAX_ACTIVE_VALIDATORS;
    pub const MaxWinners: u32 = MAX_TARGETS;
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
    type System = Runtime;
    type Solver = SequentialPhragmen<AccountId, sp_runtime::Perbill>;
    type DataProvider = Staking;
    type WeightInfo = weights::frame_election_provider_support::ZKVWeight<Runtime>;
    type MaxWinners = MaxWinners; // must be >= MAX_TARGETS because of the staking benchmark
    type Bounds = ElectionBoundsOnChain;
}

/// The numbers configured could always be more than the the maximum limits of staking pallet
/// to ensure election snapshot will not run out of memory.
pub struct StakingBenchmarkConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkConfig {
    type MaxNominators = ConstU32<MAX_VOTERS>;
    type MaxValidators = ConstU32<MAX_TARGETS>;
}

pub struct VoltaInflation;

impl zkv_runtime_common::InflationModel for VoltaInflation {
    type ExpPrecision = payout::ExpPrecision;
    type InflationBase = payout::InflationBase;
    type StakingTarget = payout::StakingTarget;
    type K = payout::K;
    type C = payout::C;
}

impl pallet_staking::Config for Runtime {
    type Currency = Balances;
    type CurrencyBalance = Balance;
    type UnixTime = Timestamp;
    type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
    type RewardRemainder = Treasury;
    type RuntimeEvent = RuntimeEvent;
    type Slash = Treasury;
    type Reward = (); // rewards are minted from the void
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type AdminOrigin = EnsureRoot<AccountId>;
    type SessionInterface = Self;
    type EraPayout = ZKVPayout<VoltaInflation, payout::EraPayoutValidatorsSplit>;
    type NextNewSession = Session;
    type ElectionProvider = OnChainExecution<OnChainSeqPhragmen>;
    type GenesisElectionProvider = OnChainExecution<OnChainSeqPhragmen>;
    type VoterList = VoterList;
    type NominationsQuota = pallet_staking::FixedNominationsQuota<10>;
    type TargetList = pallet_staking::UseValidatorsMap<Self>;
    type MaxUnlockingChunks = ConstU32<32>;
    type HistoryDepth = HistoryDepth; // Number of eras to keep in history
    type EventListeners = (); // NominationPools;
    type WeightInfo = weights::pallet_staking::ZKVWeight<Runtime>;
    type BenchmarkingConfig = StakingBenchmarkConfig;
    type MaxExposurePageSize = ConstU32<64>;
    type MaxControllersInDeprecationBatch = ConstU32<1>; // We do not have any controller accounts
                                                         // but we need at least 1 for benchmarks
    type DisablingStrategy = pallet_staking::UpToLimitDisablingStrategy;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type EventHandler = Staking;
}

impl<LocalCall> frame_system::offchain::CreateTransaction<LocalCall> for Runtime
where
    RuntimeCall: From<LocalCall>,
{
    type Extension = TxExtension;

    fn create_transaction(call: RuntimeCall, extension: TxExtension) -> UncheckedExtrinsic {
        UncheckedExtrinsic::new_transaction(call, extension)
    }
}

impl<C> frame_system::offchain::CreateTransactionBase<C> for Runtime
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type RuntimeCall = RuntimeCall;
}

impl<LocalCall> frame_system::offchain::CreateInherent<LocalCall> for Runtime
where
    RuntimeCall: From<LocalCall>,
{
    fn create_inherent(call: RuntimeCall) -> UncheckedExtrinsic {
        UncheckedExtrinsic::new_bare(call)
    }
}

impl pallet_session::historical::Config for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

parameter_types! {
    pub const MaxKeys: u32 = 10_000; // We need them for benchmarking
    pub const MaxPeerInHeartbeats: u32 = 10_000;
}

impl pallet_offences::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
}

parameter_types! {
    // One storage item; key size 32, value size 8; .
    pub const ProxyDepositBase: Balance = deposit(1, 8);
    // Additional storage item size of 33 bytes.
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const MaxProxies: u16 = 32;
    pub const AnnouncementDepositBase: Balance = deposit(1, 8);
    pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
    pub const MaxPending: u16 = 32;
}

impl pallet_proxy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = proxy::ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = weights::pallet_proxy::ZKVWeight<Runtime>;
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

mod vk_registration_parameters {
    use super::*;

    fn vks_key_size() -> u32 {
        Identity::max_len::<sp_core::H256>() as u32
    }
    fn tickets_key_size() -> u32 {
        Blake2_128Concat::max_len::<(AccountId, sp_core::H256)>() as u32
    }
    fn tickets_value_size() -> u32 {
        VkRegistrationHoldConsideration::max_encoded_len() as u32
    }
    parameter_types! {
        pub VkRegistrationBaseDeposit: Balance = deposit(2, vks_key_size() + tickets_key_size() + tickets_value_size());
        pub const VkRegistrationByteDeposit: Balance = deposit(0, 1);
        pub const VkRegistrationHoldReason: RuntimeHoldReason = RuntimeHoldReason::CommonVerifiers(pallet_verifiers::common::HoldReason::VkRegistration);
    }
}

use vk_registration_parameters::*;

type VkRegistrationHoldConsideration = HoldConsideration<
    AccountId,
    Balances,
    VkRegistrationHoldReason,
    LinearStoragePrice<VkRegistrationBaseDeposit, VkRegistrationByteDeposit, Balance>,
>;

impl pallet_verifiers::common::Config for Runtime {
    type CommonWeightInfo = Runtime;
}

impl pallet_verifiers::Config<pallet_fflonk_verifier::Fflonk> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnProofVerified = Aggregate;
    type WeightInfo =
        pallet_fflonk_verifier::FflonkWeight<weights::pallet_fflonk_verifier::ZKVWeight<Runtime>>;
    type Ticket = VkRegistrationHoldConsideration;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
}

pub const GROTH16_MAX_NUM_INPUTS: u32 = 64;
parameter_types! {
    pub const Groth16MaxNumInputs: u32 = GROTH16_MAX_NUM_INPUTS;
}

impl pallet_groth16_verifier::Config for Runtime {
    const MAX_NUM_INPUTS: u32 = Groth16MaxNumInputs::get();
}

// We should be sure that the max number of inputs does not exceed the max number of inputs in the verifier crate.
const_assert!(
    <Runtime as pallet_groth16_verifier::Config>::MAX_NUM_INPUTS
        <= pallet_groth16_verifier::MAX_NUM_INPUTS
);

impl pallet_verifiers::Config<pallet_groth16_verifier::Groth16<Runtime>> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnProofVerified = Aggregate;
    type WeightInfo = pallet_groth16_verifier::Groth16Weight<
        weights::pallet_groth16_verifier::ZKVWeight<Runtime>,
    >;
    type Ticket = VkRegistrationHoldConsideration;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
}

pub const SP1_MAX_PUBS_SIZE: u32 = 32 * 64;
parameter_types! {
    pub const Sp1MaxPubsSize: u32 = SP1_MAX_PUBS_SIZE;
}

impl pallet_sp1_verifier::Config for Runtime {
    type MaxPubsSize = Sp1MaxPubsSize;
}

impl pallet_verifiers::Config<pallet_sp1_verifier::Sp1<Runtime>> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnProofVerified = Aggregate;
    type WeightInfo =
        pallet_sp1_verifier::Sp1Weight<weights::pallet_sp1_verifier::ZKVWeight<Runtime>>;
    type Ticket = VkRegistrationHoldConsideration;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
}

parameter_types! {
    pub const Risc0MaxNSegment: u32 = 4;             // 4 segment of 2^20
    pub const Risc0Segment20MaxSize: u32 = 350_000; // risc0 2^20 segment size (a standard 2^22)
                                                    // proof is ~1_400_000
    pub const Risc0MaxPubsSize: u32 = 4 + 32 * 64;  // 4: bytes for payload length,
                                                    // 32 * 64: sufficient multiple of 32 bytes
}

impl pallet_risc0_verifier::Config for Runtime {
    type MaxNSegment = Risc0MaxNSegment;
    type Segment20MaxSize = Risc0Segment20MaxSize;
    type MaxPubsSize = Risc0MaxPubsSize;
    type WeightInfo = weights::pallet_risc0_verifier_verify_proof::ZKVWeight<Runtime>;
}

impl pallet_verifiers::Config<pallet_risc0_verifier::Risc0<Runtime>> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnProofVerified = Aggregate;
    type WeightInfo =
        pallet_risc0_verifier::Risc0Weight<weights::pallet_risc0_verifier::ZKVWeight<Runtime>>;
    type Ticket = VkRegistrationHoldConsideration;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
}

parameter_types! {
    pub const UltrahonkMaxPubs: u32 = 32;
}

impl pallet_ultrahonk_verifier::Config for Runtime {
    type MaxPubs = UltrahonkMaxPubs;
}

pub type UltrahonkVerifier = pallet_ultrahonk_verifier::Ultrahonk<Runtime>;

impl pallet_verifiers::Config<UltrahonkVerifier> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnProofVerified = Aggregate;
    type WeightInfo = pallet_ultrahonk_verifier::UltrahonkWeight<
        weights::pallet_ultrahonk_verifier::ZKVWeight<Runtime>,
    >;
    type Ticket = VkRegistrationHoldConsideration;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
}

parameter_types! {
    pub const UltraplonkMaxPubs: u32 = 32;
}

impl pallet_ultraplonk_verifier::Config for Runtime {
    type MaxPubs = UltraplonkMaxPubs;
}

pub type UltraplonkVerifier = pallet_ultraplonk_verifier::Ultraplonk<Runtime>;

impl pallet_verifiers::Config<UltraplonkVerifier> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnProofVerified = Aggregate;
    type WeightInfo = pallet_ultraplonk_verifier::UltraplonkWeight<
        weights::pallet_ultraplonk_verifier::ZKVWeight<Runtime>,
    >;
    type Ticket = VkRegistrationHoldConsideration;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
}

parameter_types! {
    pub const Coprocessor: Option<StateMachine> = Some(StateMachine::Kusama(4009));
    pub const HostStateMachine: StateMachine = StateMachine::Substrate(*b"zkv_");
}

impl pallet_ismp::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AdminOrigin = EnsureRoot<AccountId>;
    type HostStateMachine = HostStateMachine;
    type TimestampProvider = Timestamp;
    // TODO: Set to zk-verify token (+ need to allowlist module in relayer).
    // Potentially in the future could be a stable coin
    type Currency = Balances;
    type Balance = Balance;
    type Router = ModuleRouter;
    type Coprocessor = Coprocessor;
    type ConsensusClients = (ismp_grandpa::consensus::GrandpaConsensusClient<Runtime>,);
    type OffchainDB = ();
    type FeeHandler = pallet_ismp::fee_handler::WeightFeeHandler<()>;
}

impl pallet_hyperbridge_aggregations::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IsmpDispatcher = pallet_hyperbridge::Pallet<Runtime>;
    type WeightInfo = weights::pallet_hyperbridge_aggregations::ZKVWeight<Runtime>;
}

impl ismp_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IsmpHost = Ismp;
    type WeightInfo = weights::ismp_grandpa::ZKVWeight<Runtime>;
}

impl pallet_hyperbridge::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IsmpHost = Ismp;
}

impl pallet_assets_mock::Config for Runtime {
    type Currency = Balances;
    type AssetId = u32;
    type Balance = Balance;
}

parameter_types! {
    pub const Decimals: u8 = 18;
    pub const NativeAssetId: u32 = 0;
}

parameter_types! {
    /// Account of the treasury pallet.
    pub AssetAdmin: AccountId = Treasury::account_id();
}

impl pallet_token_gateway::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Dispatcher = Ismp;
    type Assets = pallet_assets_mock::Pallet<Runtime>;
    type NativeCurrency = Balances;
    type AssetAdmin = AssetAdmin;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type CreateOrigin = EnsureRoot<AccountId>;
    #[cfg(feature = "runtime-benchmarks")]
    type CreateOrigin = frame_system::EnsureSigned<AccountId>;
    type NativeAssetId = NativeAssetId;
    type Decimals = Decimals;
    type EvmToSubstrate = ();
    type WeightInfo = weights::pallet_token_gateway::ZKVWeight<Runtime>;
}

#[derive(Default)]
pub struct ModuleRouter;
impl IsmpRouter for ModuleRouter {
    fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
        match id.as_slice() {
            id if id == ZKV_MODULE_ID.to_bytes().as_slice() => Ok(Box::new(
                pallet_hyperbridge_aggregations::Pallet::<Runtime>::default(),
            )),
            id if TokenGateway::is_token_gateway(id) => Ok(Box::new(TokenGateway::default())),
            id if id == PALLET_HYPERBRIDGE_ID => {
                Ok(Box::new(pallet_hyperbridge::Pallet::<Runtime>::default()))
            }
            _ => Err(Error::ModuleNotFound(id))?,
        }
    }
}

parameter_types! {
    pub const Plonky2MaxPubsSize: u32 = 512; // eq of 64 public inputs
    pub const Plonky2MaxProofSize: u32 = 262_144;
    pub const Plonky2MaxVkSize: u32 = 50_000;
}

impl pallet_plonky2_verifier::Config for Runtime {
    type MaxProofSize = Plonky2MaxProofSize;
    type MaxPubsSize = Plonky2MaxPubsSize;
    type MaxVkSize = Plonky2MaxVkSize;
    type WeightInfo = weights::pallet_plonky2_verifier_verify_proof::ZKVWeight<Runtime>;
}

impl pallet_verifiers::Config<pallet_plonky2_verifier::Plonky2<Runtime>> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnProofVerified = Aggregate;
    type Ticket = VkRegistrationHoldConsideration;
    type WeightInfo = pallet_plonky2_verifier::Plonky2Weight<
        weights::pallet_plonky2_verifier::ZKVWeight<Runtime>,
    >;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub struct Runtime {
        // Basic stuff
        System: frame_system = 0,
        Scheduler: pallet_scheduler = 1,
        Preimage: pallet_preimage = 2,

        Timestamp: pallet_timestamp = 3,
        Balances: pallet_balances = 4,
        TransactionPayment: pallet_transaction_payment = 5,

        // Consensus support.
        // Authorship must be before session in order to note author in the correct session and era
        // for im-online and staking.
        Authorship: pallet_authorship = 6,
        Staking: pallet_staking = 7,
        Offences: pallet_offences = 8,
        Historical: pallet_session_historical = 9,

        // Consensus
        Babe: pallet_babe = 10,
        Session: pallet_session = 11,
        Grandpa: pallet_grandpa = 12,
        AuthorityDiscovery: pallet_authority_discovery = 13,

        // Opengov stuff
        Treasury: pallet_treasury = 14,
        ConvictionVoting: pallet_conviction_voting = 15,
        Referenda: pallet_referenda = 16,
        Origins: pallet_custom_origins = 17,
        VoterList: pallet_bags_list::<Instance1> = 19,

        // Bounties modules.
        Bounties: pallet_bounties = 25,
        ChildBounties: pallet_child_bounties = 26,

        // Utility modules.
        Utility: pallet_utility = 30,
        Multisig: pallet_multisig = 31,
        Proxy: pallet_proxy = 32,


        // Pallets that we know are to remove in a future. Start indices at 50 to leave room.
        Sudo: pallet_sudo = 50,
        // Vesting. Usable initially, but removed once all vesting is finished.
        Vesting: pallet_vesting = 51,

        // Our stuff
        Aggregate: pallet_aggregate = 81,
        Claim: pallet_claim = 82,
        TokenClaim: pallet_token_claim = 83,

        // ISMP
        Ismp: pallet_ismp = 90,
        IsmpGrandpa: ismp_grandpa = 91,
        HyperbridgeAggregations: pallet_hyperbridge_aggregations = 92,
        TokenGateway: pallet_token_gateway = 93,
        Hyperbridge: pallet_hyperbridge = 94,

        // Parachain pallets. Start indices at 100 to leave room.
        ParachainsOrigin: parachains::parachains_origin = 101,
        Configuration: parachains::configuration = 102,
        ParasShared: parachains::parachains_shared = 103,
        ParaInclusion: parachains::inclusion = 104,
        ParaInherent: parachains::paras_inherent = 105,
        ParaScheduler: parachains::parachains_scheduler = 106,
        Paras: parachains::paras = 107,
        Initializer: parachains::initializer = 108,
        Dmp: parachains::parachains_dmp = 109,
        Hrmp: parachains::hrmp = 110,
        ParaSessionInfo: parachains::parachains_session_info = 111,
        ParasDisputes: parachains::disputes = 112,
        ParasSlashing: parachains::slashing = 113,
        ParachainsAssignmentProvider: parachains::parachains_assigner_coretime = 114,
        OnDemandAssignmentProvider: parachains::on_demand = 115,
        Coretime: parachains::coretime = 116,

        // Parachain onboarding; visualization only, not intended for actual usage.
        Registrar: parachains::paras_registrar = 120,
        Slots: parachains::slots = 121,

        // Parachain chain (removable) pallets. Start indices at 130.
        ParasSudoWrapper: parachains::paras_sudo_wrapper = 130,

        // XCM Pallet: start indices at 140.
        XcmPallet: pallet_xcm = 140,
        MessageQueue: pallet_message_queue = 141,

        // Verifiers. Start indices at 160 to leave room and to the end (255). Don't add
        // any kind of other pallets after this value.
        CommonVerifiers: pallet_verifiers::common = 160,
        SettlementGroth16Pallet: pallet_groth16_verifier = 161,
        SettlementRisc0Pallet: pallet_risc0_verifier = 162,
        SettlementUltraplonkPallet: pallet_ultraplonk_verifier = 163,
        SettlementPlonky2Pallet: pallet_plonky2_verifier = 165,
        SettlementFFlonkPallet: pallet_fflonk_verifier = 166,
        SettlementSp1Pallet: pallet_sp1_verifier = 167,
        SettlementUltrahonkPallet: pallet_ultrahonk_verifier = 168,
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type TxExtension = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
pub type ParachainMigrations = parachains::Migrations;

#[allow(unused_parens)]
type Migrations = (migrations::Unreleased, ParachainMigrations);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;
/// Unchecked signature payload type as expected by this runtime.
pub type UncheckedSignaturePayload =
    generic::UncheckedSignaturePayload<Address, Signature, TxExtension>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TxExtension>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, TxExtension>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    Migrations,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [frame_system_extensions, SystemExtensionsBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_bags_list, VoterList]
        [pallet_babe, Babe]
        [pallet_grandpa, Grandpa]
        [pallet_timestamp, Timestamp]
        [pallet_sudo, Sudo]
        [pallet_multisig, Multisig]
        [pallet_scheduler, Scheduler]
        [pallet_preimage, Preimage]
        [pallet_session, SessionBench::<Runtime>]
        [pallet_staking, Staking]
        [frame_election_provider_support, ElectionProviderBench::<Runtime>]
        [pallet_conviction_voting, ConvictionVoting]
        [pallet_treasury, Treasury]
        [pallet_bounties, Bounties]
        [pallet_child_bounties, ChildBounties]
        [pallet_referenda, Referenda]
        [pallet_utility, Utility]
        [pallet_vesting, Vesting]
        [pallet_proxy, Proxy]
        [pallet_transaction_payment, TransactionPayment]
        // hyperbridge
        [ismp_grandpa, IsmpGrandpa]
        [pallet_token_gateway, TokenGateway]
        // our pallets
        [pallet_aggregate, Aggregate]
        [pallet_claim, Claim]
        [pallet_token_claim, TokenClaim]
        [pallet_hyperbridge_aggregations, HyperbridgeAggregations]
        // verifiers
        [pallet_fflonk_verifier, FflonkVerifierBench::<Runtime>]
        [pallet_groth16_verifier, Groth16VerifierBench::<Runtime>]
        [pallet_risc0_verifier, Risc0VerifierBench::<Runtime>]
        [pallet_risc0_verifier_verify_proof, Risc0VerifierVerifyProofBench::<Runtime>]
        [pallet_risc0_verifier_extend, Risc0VerifierExtendBench::<Runtime>]
        [pallet_ultrahonk_verifier, UltrahonkVerifierBench::<Runtime>]
        [pallet_ultraplonk_verifier, UltraplonkVerifierBench::<Runtime>]
        [pallet_plonky2_verifier, Plonky2VerifierBench::<Runtime>]
        [pallet_plonky2_verifier_verify_proof, Plonky2VerifierVerifyProofBench::<Runtime>]
        [pallet_sp1_verifier, Sp1VerifierBench::<Runtime>]
        // parachains
        [parachains::configuration, Configuration]
        [parachains::disputes, ParasDisputes]
        [parachains::slashing, ParasSlashing]
        [parachains::hrmp, Hrmp]
        [parachains::inclusion, ParaInclusion]
        [parachains::initializer, Initializer]
        [parachains::paras, Paras]
        [parachains::paras_inherent, ParaInherent]
        [parachains::on_demand, OnDemandAssignmentProvider]
        [parachains::coretime, Coretime]
        [pallet_message_queue, MessageQueue]
        // xcm
        [pallet_xcm, xcm::XcmPalletBench::<Runtime>]
        [xcm::pallet_xcm_benchmarks_fungible, xcm::XcmPalletBenchFungible::<Runtime>]
        [xcm::pallet_xcm_benchmarks_generic, xcm::XcmPalletBenchGeneric::<Runtime>]
    );
}

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
    };

use polkadot_primitives::{
    self as primitives, slashing,
    vstaging::{
        async_backing::BackingState, CandidateEvent,
        CommittedCandidateReceiptV2 as CommittedCandidateReceipt, CoreState, ScrapedOnChainVotes,
    },
    ApprovalVotingParams, CandidateHash, CoreIndex, DisputeState, ExecutorParams,
    GroupRotationInfo, Id as ParaId, InboundDownwardMessage, InboundHrmpMessage, NodeFeatures,
    OccupiedCoreAssumption, PersistedValidationData, SessionIndex, SessionInfo, ValidationCode,
    ValidationCodeHash, ValidatorId, ValidatorIndex, PARACHAIN_KEY_TYPE_ID,
};

use hp_dispatch::{Destination, DispatchAggregation};
use pallet_hyperbridge_aggregations::{Params, ZKV_MODULE_ID};

pub use polkadot_runtime_parachains::runtime_api_impl::{
    v11 as parachains_runtime_api_impl, vstaging as parachains_staging_runtime_api_impl,
};

// Used for testing purposes only.
sp_api::decl_runtime_apis! {
    pub trait GetLastTimestamp {
        /// Returns the last timestamp of a runtime.
        fn get_last_timestamp() -> u64;
    }
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
            sp_consensus_babe::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDurationInBlocks::get().into(),
                c: epoch_config.c,
                authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: epoch_config.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: BabeId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }

    impl authority_discovery_primitives::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            polkadot_runtime_parachains::runtime_api_impl::v11::relevant_authority_ids::<Runtime>()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            _authority_id: GrandpaId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce {
            System::account_nonce(account)
        }
    }

    impl pallet_ismp_runtime_api::IsmpRuntimeApi<Block, <Block as BlockT>::Hash> for Runtime {
        fn host_state_machine() -> StateMachine {
            <Runtime as pallet_ismp::Config>::HostStateMachine::get()
        }

        fn challenge_period(id: StateMachineId) -> Option<u64> {
            pallet_ismp::Pallet::<Runtime>::challenge_period(id)
        }

        fn block_events() -> Vec<ismp::events::Event> {
            pallet_ismp::Pallet::<Runtime>::block_events()
        }

        fn block_events_with_metadata() -> Vec<(ismp::events::Event, Option<u32>)> {
            pallet_ismp::Pallet::<Runtime>::block_events_with_metadata()
        }

        fn consensus_state(id: ConsensusClientId) -> Option<Vec<u8>> {
            pallet_ismp::Pallet::<Runtime>::consensus_states(id)
        }

        fn state_machine_update_time(height: StateMachineHeight) -> Option<u64> {
            pallet_ismp::Pallet::<Runtime>::state_machine_update_time(height)
        }

        fn latest_state_machine_height(id: StateMachineId) -> Option<u64> {
            pallet_ismp::Pallet::<Runtime>::latest_state_machine_height(id)
        }

        fn requests(commitments: Vec<H256>) -> Vec<Request> {
            pallet_ismp::Pallet::<Runtime>::requests(commitments)
        }

        fn responses(commitments: Vec<H256>) -> Vec<Response> {
            pallet_ismp::Pallet::<Runtime>::responses(commitments)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
        for Runtime
    {
        fn query_call_info(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl aggregate_rpc_runtime_api::AggregateApi<Block> for Runtime {
        fn get_statement_path(
            domain_id: u32,
            aggregation_id: u64,
            statement: sp_core::H256
        ) -> Result<aggregate_rpc_runtime_api::MerkleProof, aggregate_rpc_runtime_api::PathRequestError> {
            Aggregate::get_statement_path(domain_id, aggregation_id, statement).map(|c| c.into())
        }
    }

    #[api_version(12)]
    impl primitives::runtime_api::ParachainHost<Block> for Runtime {
        fn validators() -> Vec<ValidatorId> {
            parachains_runtime_api_impl::validators::<Runtime>()
        }

        fn validator_groups() -> (Vec<Vec<ValidatorIndex>>, GroupRotationInfo<BlockNumber>) {
            parachains_runtime_api_impl::validator_groups::<Runtime>()
        }

        fn availability_cores() -> Vec<CoreState<Hash, BlockNumber>> {
            parachains_runtime_api_impl::availability_cores::<Runtime>()
        }

        fn persisted_validation_data(para_id: ParaId, assumption: OccupiedCoreAssumption)
            -> Option<PersistedValidationData<Hash, BlockNumber>> {
            parachains_runtime_api_impl::persisted_validation_data::<Runtime>(para_id, assumption)
        }

        fn assumed_validation_data(
            para_id: ParaId,
            expected_persisted_validation_data_hash: Hash,
        ) -> Option<(PersistedValidationData<Hash, BlockNumber>, ValidationCodeHash)> {
            parachains_runtime_api_impl::assumed_validation_data::<Runtime>(
                para_id,
                expected_persisted_validation_data_hash,
            )
        }

        fn check_validation_outputs(
            para_id: ParaId,
            outputs: primitives::CandidateCommitments,
        ) -> bool {
            parachains_runtime_api_impl::check_validation_outputs::<Runtime>(para_id, outputs)
        }

        fn session_index_for_child() -> SessionIndex {
            parachains_runtime_api_impl::session_index_for_child::<Runtime>()
        }

        fn validation_code(para_id: ParaId, assumption: OccupiedCoreAssumption)
            -> Option<ValidationCode> {
            parachains_runtime_api_impl::validation_code::<Runtime>(para_id, assumption)
        }

        fn candidate_pending_availability(para_id: ParaId) -> Option<CommittedCandidateReceipt<Hash>> {
            #[allow(deprecated)]
            parachains_runtime_api_impl::candidate_pending_availability::<Runtime>(para_id)
        }

        fn candidate_events() -> Vec<CandidateEvent<Hash>> {
            parachains_runtime_api_impl::candidate_events::<Runtime, _>(|ev| {
                match ev {
                    RuntimeEvent::ParaInclusion(ev) => {
                        Some(ev)
                    }
                    _ => None,
                }
            })
        }

        fn session_info(index: SessionIndex) -> Option<SessionInfo> {
            parachains_runtime_api_impl::session_info::<Runtime>(index)
        }

        fn session_executor_params(session_index: SessionIndex) -> Option<ExecutorParams> {
            parachains_runtime_api_impl::session_executor_params::<Runtime>(session_index)
        }

        fn dmq_contents(recipient: ParaId) -> Vec<InboundDownwardMessage<BlockNumber>> {
            parachains_runtime_api_impl::dmq_contents::<Runtime>(recipient)
        }

        fn inbound_hrmp_channels_contents(
            recipient: ParaId
        ) -> alloc::collections::btree_map::BTreeMap<ParaId, Vec<InboundHrmpMessage<BlockNumber>>> {
            parachains_runtime_api_impl::inbound_hrmp_channels_contents::<Runtime>(recipient)
        }

        fn validation_code_by_hash(hash: ValidationCodeHash) -> Option<ValidationCode> {
            parachains_runtime_api_impl::validation_code_by_hash::<Runtime>(hash)
        }

        fn on_chain_votes() -> Option<ScrapedOnChainVotes<Hash>> {
            parachains_runtime_api_impl::on_chain_votes::<Runtime>()
        }

        fn submit_pvf_check_statement(
            stmt: primitives::PvfCheckStatement,
            signature: primitives::ValidatorSignature
        ) {
            parachains_runtime_api_impl::submit_pvf_check_statement::<Runtime>(stmt, signature)
        }

        fn pvfs_require_precheck() -> Vec<ValidationCodeHash> {
            parachains_runtime_api_impl::pvfs_require_precheck::<Runtime>()
        }

        fn validation_code_hash(para_id: ParaId, assumption: OccupiedCoreAssumption)
            -> Option<ValidationCodeHash>
        {
            parachains_runtime_api_impl::validation_code_hash::<Runtime>(para_id, assumption)
        }

        fn disputes() -> Vec<(SessionIndex, CandidateHash, DisputeState<BlockNumber>)> {
            parachains_runtime_api_impl::get_session_disputes::<Runtime>()
        }

        fn unapplied_slashes(
        ) -> Vec<(SessionIndex, CandidateHash, slashing::PendingSlashes)> {
            parachains_runtime_api_impl::unapplied_slashes::<Runtime>()
        }

        fn key_ownership_proof(
            validator_id: ValidatorId,
        ) -> Option<slashing::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((PARACHAIN_KEY_TYPE_ID, validator_id))
                .map(|p| p.encode())
                .map(slashing::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_dispute_lost(
            dispute_proof: slashing::DisputeProof,
            key_ownership_proof: slashing::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            parachains_runtime_api_impl::submit_unsigned_slashing_report::<Runtime>(
                dispute_proof,
                key_ownership_proof,
            )
        }

        fn minimum_backing_votes() -> u32 {
            parachains_runtime_api_impl::minimum_backing_votes::<Runtime>()
        }

        fn para_backing_state(para_id: ParaId) -> Option<BackingState> {
            parachains_runtime_api_impl::backing_state::<Runtime>(para_id)
        }

        fn async_backing_params() -> primitives::AsyncBackingParams {
            parachains_runtime_api_impl::async_backing_params::<Runtime>()
        }

        fn disabled_validators() -> Vec<ValidatorIndex> {
            parachains_runtime_api_impl::disabled_validators::<Runtime>()
        }

        fn node_features() -> NodeFeatures {
            parachains_runtime_api_impl::node_features::<Runtime>()
        }

        fn approval_voting_params() -> ApprovalVotingParams {
            parachains_runtime_api_impl::approval_voting_params::<Runtime>()
        }

        fn claim_queue() -> BTreeMap<CoreIndex, VecDeque<ParaId>> {
            parachains_runtime_api_impl::claim_queue::<Runtime>()
        }

        fn candidates_pending_availability(para_id: ParaId) -> Vec<CommittedCandidateReceipt<Hash>> {
            parachains_runtime_api_impl::candidates_pending_availability::<Runtime>(para_id)
        }

        fn validation_code_bomb_limit() -> u32 {
            parachains_staging_runtime_api_impl::validation_code_bomb_limit::<Runtime>()
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;
            use baseline::Pallet as BaselineBench;
            use pallet_election_provider_support_benchmarking::Pallet as ElectionProviderBench;
            use pallet_session_benchmarking::Pallet as SessionBench;
            use pallet_fflonk_verifier::benchmarking::Pallet as FflonkVerifierBench;
            use pallet_groth16_verifier::benchmarking::Pallet as Groth16VerifierBench;
            use pallet_risc0_verifier::benchmarking::Pallet as Risc0VerifierBench;
            use pallet_risc0_verifier::benchmarking_verify_proof::Pallet as Risc0VerifierVerifyProofBench;
            use pallet_risc0_verifier::extend_benchmarking::Pallet as Risc0VerifierExtendBench;
            use pallet_ultrahonk_verifier::benchmarking::Pallet as UltrahonkVerifierBench;
            use pallet_ultraplonk_verifier::benchmarking::Pallet as UltraplonkVerifierBench;
            use pallet_plonky2_verifier::benchmarking_verify_proof::Pallet as Plonky2VerifierVerifyProofBench;
            use pallet_plonky2_verifier::benchmarking::Pallet as Plonky2VerifierBench;
            use pallet_sp1_verifier::benchmarking::Pallet as Sp1VerifierBench;

            pub mod xcm {
                pub use pallet_xcm::benchmarking::Pallet as XcmPalletBench;
                pub use pallet_xcm_benchmarks::fungible::Pallet as XcmPalletBenchFungible;
                pub use pallet_xcm_benchmarks::generic::Pallet as XcmPalletBenchGeneric;
            }

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmarks!(list, extra);
            let storage_info = AllPalletsWithSystem::storage_info();

            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
            use sp_storage::TrackedStorageKey;
            use frame_system_benchmarking::Pallet as SystemBench;
            use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;
            use baseline::Pallet as BaselineBench;
            use pallet_election_provider_support_benchmarking::Pallet as ElectionProviderBench;
            use pallet_session_benchmarking::Pallet as SessionBench;
            use pallet_fflonk_verifier::benchmarking::Pallet as FflonkVerifierBench;
            use pallet_groth16_verifier::benchmarking::Pallet as Groth16VerifierBench;
            use pallet_risc0_verifier::benchmarking::Pallet as Risc0VerifierBench;
            use pallet_risc0_verifier::benchmarking_verify_proof::Pallet as Risc0VerifierVerifyProofBench;
            use pallet_risc0_verifier::extend_benchmarking::Pallet as Risc0VerifierExtendBench;
            use pallet_ultrahonk_verifier::benchmarking::Pallet as UltrahonkVerifierBench;
            use pallet_ultraplonk_verifier::benchmarking::Pallet as UltraplonkVerifierBench;
            use pallet_plonky2_verifier::benchmarking_verify_proof::Pallet as Plonky2VerifierVerifyProofBench;
            use pallet_plonky2_verifier::benchmarking::Pallet as Plonky2VerifierBench;
            use pallet_sp1_verifier::benchmarking::Pallet as Sp1VerifierBench;

            pub mod xcm {
                use super::*;
                use alloc::vec;
                use xcm::latest::{Asset, AssetId, Assets, Location, InteriorLocation, Junction, Junctions::Here, NetworkId, Response, Fungibility::Fungible};
                use frame_benchmarking::BenchmarkError;

                pub use pallet_xcm::benchmarking::Pallet as XcmPalletBench;
                pub use pallet_xcm_benchmarks::fungible::Pallet as XcmPalletBenchFungible;
                pub use pallet_xcm_benchmarks::generic::Pallet as XcmPalletBenchGeneric;

                parameter_types! {
                    pub ExistentialDepositAsset: Option<Asset> = Some((
                        xcm_config::TokenLocation::get(),
                        ExistentialDeposit::get()
                    ).into());
                    pub const TestParaId: ParaId = ParaId::new(xcm_config::TEST_PARA_ID);
                    pub const RndParaId: ParaId = ParaId::new(123456);
                }

                impl pallet_xcm::benchmarking::Config for Runtime {
                    type DeliveryHelper = (
                        polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper<
                            xcm_config::XcmConfig,
                            ExistentialDepositAsset,
                            xcm_config::PriceForChildParachainDelivery,
                            TestParaId,
                            ()
                        >,
                        polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper<
                            xcm_config::XcmConfig,
                            ExistentialDepositAsset,
                            xcm_config::PriceForChildParachainDelivery,
                            RndParaId,
                            ()
                        >,
                    );

                    fn get_asset() -> Asset {
                        Asset {
                            id: AssetId(xcm_config::TokenLocation::get()),
                            fun: Fungible(ExistentialDeposit::get()),
                        }
                    }

                    fn reachable_dest() -> Option<Location> {
                        Some(xcm_config::TestParaLocation::get())
                    }

                    fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
                        // Relay/native token can be teleported to/from TestPara.
                        Some((
                            Asset { fun: Fungible(ExistentialDeposit::get()), id: AssetId(xcm_config::TokenLocation::get()) },
                            xcm_config::TestParaLocation::get(),
                        ))
                    }
                }

                impl pallet_xcm_benchmarks::Config for Runtime {
                    type XcmConfig = xcm_config::XcmConfig;
                    type AccountIdConverter = xcm_config::SovereignAccountOf;
                    type DeliveryHelper = (
                        polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper<
                            xcm_config::XcmConfig,
                            ExistentialDepositAsset,
                            xcm_config::PriceForChildParachainDelivery,
                            TestParaId,
                            ()
                        >,
                    );
                    fn valid_destination() -> Result<Location, BenchmarkError> {
                        Ok(xcm_config::TestParaLocation::get())
                    }
                    fn worst_case_holding(_depositable_count: u32) -> Assets {
                        vec![Asset {
                            id: AssetId(xcm_config::TokenLocation::get()),
                            fun: Fungible(MILLIONS),
                        }].into()
                    }
                }

                parameter_types! {
                    pub TrustedTeleporter: Option<(Location, Asset)> = Some((
                        xcm_config::TestParaLocation::get(),
                        Asset {
                            id: AssetId(xcm_config::TokenLocation::get()),
                            fun: Fungible(ExistentialDeposit::get()),
                        },
                    ));
                    pub const TrustedReserve: Option<(Location, Asset)> = None;
                }

                impl pallet_xcm_benchmarks::fungible::Config for Runtime {
                    type TransactAsset = Balances;
                    type CheckedAccount = xcm_config::LocalCheckAccount;
                    type TrustedTeleporter = TrustedTeleporter;
                    type TrustedReserve = TrustedReserve;

                    fn get_asset() -> Asset {
                        Asset {
                            id: AssetId(xcm_config::TokenLocation::get()),
                            fun: Fungible(ExistentialDeposit::get()),
                        }
                    }
                }

                impl pallet_xcm_benchmarks::generic::Config for Runtime {
                    type TransactAsset = Balances;
                    type RuntimeCall = RuntimeCall;

                    fn worst_case_response() -> (u64, Response) {
                        (0u64, Response::Version(Default::default()))
                    }

                    fn worst_case_asset_exchange() -> Result<(Assets, Assets), BenchmarkError> {
                        // ZKV doesn't support asset exchanges
                        Err(BenchmarkError::Skip)
                    }

                    fn universal_alias() -> Result<(Location, Junction), BenchmarkError> {
                        // The XCM executor of ZKV doesn't have a configured `UniversalAliases`
                        Err(BenchmarkError::Skip)
                    }

                    fn transact_origin_and_runtime_call() -> Result<(Location, RuntimeCall), BenchmarkError> {
                        // Currently disabled
                        Err(BenchmarkError::Skip)
                    }

                    fn subscribe_origin() -> Result<Location, BenchmarkError> {
                        Ok(xcm_config::TestParaLocation::get())
                    }

                    fn claimable_asset() -> Result<(Location, Location, Assets), BenchmarkError> {
                        // an asset that can be trapped and claimed
                        let origin = xcm_config::TestParaLocation::get();
                        let assets: Assets = (AssetId(xcm_config::TokenLocation::get()), VFY).into();
                        let ticket = Location { parents: 0, interior: Here };
                        Ok((origin, ticket, assets))
                    }

                    fn fee_asset() -> Result<Asset, BenchmarkError> {
                        Ok(Asset {
                            id: xcm_config::FeeAssetId::get(),
                            fun: Fungible(MILLIONS),
                        })
                    }

                    fn unlockable_asset() -> Result<(Location, Location, Asset), BenchmarkError> {
                        // ZKV doesn't support asset locking
                        Err(BenchmarkError::Skip)
                    }

                    fn export_message_origin_and_destination(
                    ) -> Result<(Location, NetworkId, InteriorLocation), BenchmarkError> {
                        // ZKV doesn't support exporting messages
                        Err(BenchmarkError::Skip)
                    }

                    fn alias_origin() -> Result<(Location, Location), BenchmarkError> {
                        // The XCM executor of ZKV doesn't have a configured `Aliasers`
                        Err(BenchmarkError::Skip)
                    }
                }
            }

            use frame_support::traits::WhitelistedStorageKeys;
            let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);

            Ok(batches)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade(checks).unwrap();
            (weight, BlockWeights::get().max_block)
        }

        fn execute_block(
            block: Block,
            state_root_check: bool,
            signature_check: bool,
            select: frame_try_runtime::TryStateSelect,
        ) -> Weight {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here.
            Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            get_preset::<RuntimeGenesisConfig>(id, &genesis_config_presets::get_preset)
        }

        fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
           genesis_config_presets::preset_names()
        }
    }

    // Used only in runtime tests
    impl crate::GetLastTimestamp<Block> for Runtime {
        fn get_last_timestamp() -> u64 {
            Timestamp::now()
        }
    }

}

#[cfg(feature = "runtime-benchmarks")]
mod runtime_benchmarking_extra_config {
    use crate::Runtime;

    impl frame_system_benchmarking::Config for Runtime {}
    impl frame_benchmarking::baseline::Config for Runtime {}
    impl pallet_election_provider_support_benchmarking::Config for Runtime {}

    impl pallet_session_benchmarking::Config for Runtime {}

    impl crate::parachains::slashing::benchmarking::Config for Runtime {}
}
