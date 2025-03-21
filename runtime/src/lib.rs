#![cfg_attr(not(feature = "std"), no_std)]

pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem},
    weights::{
        constants::WEIGHT_REF_TIME_PER_SECOND, IdentityFee, Weight,
    },
    StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_stwo_verifier;

impl frame_system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u32;
    type BlockNumber = u32;
    type Hash = sp_core::H256;
    type Hashing = sp_runtime::traits::BlakeTwo256;
    type AccountId = u32;
    type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
    type Header = sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU32<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MaxProofSize: u32 = 1024 * 1024; // 1MB
    pub const MaxKeySize: u32 = 1024; // 1KB
}

impl pallet_stwo_verifier::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxProofSize = MaxProofSize;
    type MaxKeySize = MaxKeySize;
    type WeightInfo = pallet_stwo_verifier::weights::SubstrateWeight<Runtime>;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system,
        StwoVerifier: pallet_stwo_verifier,
    }
);
