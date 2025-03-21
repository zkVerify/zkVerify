use crate::{self as pallet_stwo_verifier, weights::MockWeightInfo};
use frame_support::{
    parameter_types,
    traits::ConstU16,
    pallet_prelude::ConstU32, // Tambahkan import ini
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        StwoVerifier: pallet_stwo_verifier,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaxProofSize: u32 = 1024 * 1024; // 1MB
    pub const MaxKeySize: u32 = 1024; // 1KB
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_stwo_verifier::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxProofSize = MaxProofSize;
    type MaxKeySize = MaxKeySize;
    type WeightInfo = MockWeightInfo;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
