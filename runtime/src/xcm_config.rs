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

#![cfg(feature = "relay")]

//! XCM configuration for zkVerify.
//! FIXME: this configuration is meant for testing only, and MUST not deployed to a production
//! network without proper assessment.

use super::{
    AccountId, AllPalletsWithSystem, Balances, Dmp, ParaId, Runtime, RuntimeCall, RuntimeEvent,
    RuntimeOrigin, TransactionByteFee, XcmPallet,
};
use crate::{governance::GeneralAdmin, parachains::parachains_origin};
use frame_support::{
    parameter_types,
    traits::{Contains, Equals, Everything, Nothing},
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_runtime_common::{
    xcm_sender::{ChildParachainRouter, ExponentialPrice},
    ToAuthor,
};

use crate::currency::CENTS;
use sp_core::ConstU32;
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, ChildParachainAsNative, ChildParachainConvertsVia,
    DescribeAllTerminal, DescribeFamily, FrameTransactionalProcessor, FungibleAdapter,
    HashedDescription, IsConcrete, MintLocation, OriginToPluralityVoice, SignedAccountId32AsNative,
    SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, TrailingSetTopicAsId,
    UsingComponents, WeightInfoBounds, WithComputedOrigin, WithUniqueTopic,
    XcmFeeManagerFromComponents, XcmFeeToAccount,
};

use crate::weights::pallet_xcm::ZKVWeight as XcmPalletZKVWeight;
use crate::weights::xcm::ZKVWeight as XcmZKVWeight;

const ZKV_GENESIS_HASH: [u8; 32] =
    hex_literal::hex!("e2a4f521dbcba897cd2359adc5e7725f409b17f9ae129737904e5e8939c22a05");

parameter_types! {
    pub const RootLocation: Location = Here.into_location();
    /// The location of the ACME token, from the context of this chain. Since this token is native to this
    /// chain, we make it synonymous with it and thus it is the `Here` location, which means "equivalent to
    /// the context".
    pub const TokenLocation: Location = Here.into_location();
    /// The ZKV network ID.
    pub const ThisNetwork: NetworkId = NetworkId::ByGenesis(ZKV_GENESIS_HASH);
    /// Our location in the universe of consensus systems.
    pub UniversalLocation: InteriorLocation = [GlobalConsensus(ThisNetwork::get())].into();
    /// The Checking Account, which holds any native assets that have been teleported out and not back in (yet).
    pub CheckAccount: AccountId = XcmPallet::check_account();
    /// The Checking Account along with the indication that the local chain is able to mint tokens.
    pub LocalCheckAccount: (AccountId, MintLocation) = (CheckAccount::get(), MintLocation::Local);
    /// Account of the treasury pallet.
    pub TreasuryAccount: AccountId = crate::Treasury::account_id();
}

/// The canonical means of converting a `Location` into an `AccountId`, used when we want to
/// determine the sovereign account controlled by a location.
pub type SovereignAccountOf = (
    // Child parachain id to its local sovereign `AccountId`.
    ChildParachainConvertsVia<ParaId, AccountId>,
    // `AccountId32` location on the local chain to a local account.
    AccountId32Aliases<ThisNetwork, AccountId>,
    // Foreign locations alias into accounts according to a hash of their standard description
    // (e.g. remote origins).
    HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
);

/// Our asset transactor. This is what allows us to interact with the runtime assets from the point
/// of view of XCM-only concepts like `Location` and `Asset`.
///
/// Ours is only aware of the Balances pallet, which is mapped to `TokenLocation`.
pub type LocalAssetTransactor = FungibleAdapter<
    // Use this currency:
    Balances,
    // Use this currency when it is a fungible asset matching the given location or name:
    IsConcrete<TokenLocation>,
    // We can convert the `Location`s with our converter above:
    SovereignAccountOf,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We track our teleports in/out to keep total issuance correct.
    LocalCheckAccount,
>;

/// The means that we convert an XCM origin `Location` into the runtime's `Origin` type for
/// local dispatch. This is a conversion function from an `OriginKind` type along with the
/// `Location` value and returns an `Origin` value or an error.
type LocalOriginConverter = (
    // If the origin kind is `Sovereign`, then return a `Signed` origin with the account determined
    // by the `SovereignAccountOf` converter.
    SovereignSignedViaLocation<SovereignAccountOf, RuntimeOrigin>,
    // If the origin kind is `Native` and the XCM origin is a child parachain, then we can express
    // it with the special `parachains_origin::Origin` origin variant.
    ChildParachainAsNative<parachains_origin::Origin, RuntimeOrigin>,
    // If the origin kind is `Native` and the XCM origin is the `AccountId32` location, then it can
    // be expressed using the `Signed` origin variant.
    SignedAccountId32AsNative<ThisNetwork, RuntimeOrigin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
    /// The amount of weight an XCM operation takes. This is a safe overestimate.
    pub const BaseXcmWeight: Weight = Weight::from_parts(1_000_000_000, 1024);
    /// Maximum number of instructions in a single XCM fragment. A sanity check against weight
    /// calculations getting too crazy.
    pub const MaxInstructions: u32 = 100;
    /// The asset ID for the asset that we use to pay for message delivery fees.
    pub FeeAssetId: AssetId = AssetId(TokenLocation::get());
    /// The base fee for the message delivery fees.
    pub const BaseDeliveryFee: u128 = CENTS.saturating_mul(3);
}

pub type PriceForChildParachainDelivery =
    ExponentialPrice<FeeAssetId, BaseDeliveryFee, TransactionByteFee, Dmp>;

/// The XCM router. When we want to send an XCM message, we use this type. It amalgamates all of our
/// individual routers.
pub type XcmRouter = WithUniqueTopic<(
    // Only one router so far - use DMP to communicate with child parachains.
    ChildParachainRouter<Runtime, XcmPallet, PriceForChildParachainDelivery>,
)>;

pub const TEST_PARA_ID: u32 = 1599;
parameter_types! {
    pub const Acme: AssetFilter = Wild(AllOf { fun: WildFungible, id: AssetId(TokenLocation::get()) });
    pub TestParaLocation: Location = Parachain(TEST_PARA_ID).into_location();
    pub AcmeForTest: (AssetFilter, Location) = (Acme::get(), TestParaLocation::get());
    pub const MaxAssetsIntoHolding: u32 = 64;
}

/// ZKV Relay recognizes/respects Test parachain as teleporter for ACME.
pub type TrustedTeleporters = xcm_builder::Case<AcmeForTest>;

pub struct OnlyParachains;
impl Contains<Location> for OnlyParachains {
    fn contains(loc: &Location) -> bool {
        matches!(loc.unpack(), (0, [Parachain(_)]))
    }
}

pub struct LocalPlurality;
impl Contains<Location> for LocalPlurality {
    fn contains(loc: &Location) -> bool {
        matches!(loc.unpack(), (0, [Plurality { .. }]))
    }
}

/// The barriers one of which must be passed for an XCM message to be executed.
pub type Barrier = TrailingSetTopicAsId<(
    // TrailingSetTopicAsId consumes any trailing SetTopic instruction, to set the message id.
    // Consume expected weight.
    TakeWeightCredit,
    // Expected responses are OK.
    AllowKnownQueryResponses<XcmPallet>,
    WithComputedOrigin<
        (
            // If the message is one that immediately attempts to pay for execution, then allow it.
            AllowTopLevelPaidExecutionFrom<Everything>,
            // Subscriptions for version tracking are OK.
            AllowSubscriptionsFrom<OnlyParachains>,
        ),
        UniversalLocation,
        ConstU32<8>,
    >,
)>;

/// Locations that will not be charged fees in the executor, neither for execution nor delivery.
/// We only waive fees for system functions, which these locations represent.
pub type WaivedLocations = (Equals<RootLocation>, LocalPlurality);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = XcmRouter;
    type XcmRecorder = XcmPallet;
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = LocalOriginConverter;
    // ZKV Relay recognises no chains which act as reserves.
    type IsReserve = ();
    type IsTeleporter = TrustedTeleporters;
    type UniversalLocation = UniversalLocation;
    type Barrier = Barrier;
    type Weigher = WeightInfoBounds<XcmZKVWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
    // The weight trader piggybacks on the existing transaction-fee conversion logic.
    type Trader = UsingComponents<
        crate::IdentityFee<crate::Balance>,
        TokenLocation,
        AccountId,
        Balances,
        ToAuthor<Runtime>,
    >;
    type ResponseHandler = XcmPallet;
    type AssetTrap = XcmPallet;
    type AssetLocker = ();
    type AssetExchanger = ();
    type AssetClaims = XcmPallet;
    type SubscriptionService = XcmPallet;
    type PalletInstancesInfo = AllPalletsWithSystem;
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type FeeManager = XcmFeeManagerFromComponents<
        WaivedLocations,
        XcmFeeToAccount<Self::AssetTransactor, AccountId, TreasuryAccount>,
    >;
    // No bridges on the Relay Chain
    type MessageExporter = ();
    type UniversalAliases = Nothing;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
    type Aliasers = Nothing;
    type TransactionalProcessor = FrameTransactionalProcessor;
    type HrmpNewChannelOpenRequestHandler = ();
    type HrmpChannelAcceptedHandler = ();
    type HrmpChannelClosingHandler = ();
}

const FELLOWSHIP_ADMIN_INDEX: u32 = 1; // to be moved to some constants mod

parameter_types! {
    // `GeneralAdmin` pluralistic body.
    pub const GeneralAdminBodyId: BodyId = BodyId::Administration;
    // StakingAdmin pluralistic body.
    pub const StakingAdminBodyId: BodyId = BodyId::Defense;
    // FellowshipAdmin pluralistic body.
    pub const FellowshipAdminBodyId: BodyId = BodyId::Index(FELLOWSHIP_ADMIN_INDEX);
    // `Treasurer` pluralistic body.
    pub const TreasurerBodyId: BodyId = BodyId::Treasury;
}

/// Type to convert the `GeneralAdmin` origin to a Plurality `Location` value.
pub type GeneralAdminToPlurality =
    OriginToPluralityVoice<RuntimeOrigin, GeneralAdmin, GeneralAdminBodyId>;

/// Type to convert an `Origin` type value into a `Location` value which represents an interior
/// location of this chain.
pub type LocalOriginToLocation = (
    GeneralAdminToPlurality,
    // And a usual Signed origin to be used in XCM as a corresponding AccountId32
    SignedToAccountId32<RuntimeOrigin, AccountId, ThisNetwork>,
);

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    // We allow signed accounts to send XCM messages. We use this to test remote proof verification
    // on the relay chain through XCM.
    type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    // Anyone can execute XCM messages locally (needed for teleporting).
    type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Everything; // == Allow All
    type XcmReserveTransferFilter = Everything; // == Allow All
    type Weigher = WeightInfoBounds<XcmZKVWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
    type UniversalLocation = UniversalLocation;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type TrustedLockers = ();
    type SovereignAccountOf = SovereignAccountOf;
    type MaxLockers = ConstU32<8>;
    type MaxRemoteLockConsumers = ConstU32<0>;
    type RemoteLockConsumerIdentifier = ();
    type WeightInfo = XcmPalletZKVWeight<Runtime>;
    type AdminOrigin = EnsureRoot<AccountId>;
}
