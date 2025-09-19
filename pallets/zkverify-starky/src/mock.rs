 #![cfg(test)]

 use crate as pallet_zkverify_starky;
use frame_support::traits::{ConstU32, ConstU64, ConstU16, Everything};
 use frame_system as system;
use sp_runtime::BuildStorage;

 type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        ZkStarky: pallet_zkverify_starky::{Pallet, Call, Storage, Event<T>},
    }
);

 impl system::Config for Test {
	type BaseCallFilter = Everything;
 	type BlockWeights = ();
 	type BlockLength = ();
 	type DbWeight = ();
 	type RuntimeOrigin = RuntimeOrigin;
 	type RuntimeCall = RuntimeCall;
 	type RuntimeEvent = RuntimeEvent;
    type Block = Block;
	type Hash = sp_runtime::testing::H256;
 	type Hashing = sp_runtime::traits::BlakeTwo256;
 	type AccountId = u64;
 	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
    type PalletInfo = PalletInfo;
 	type AccountData = ();
 	type OnNewAccount = ();
 	type OnKilledAccount = ();
	type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
 	type OnSetCode = ();
	type Nonce = u64;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

 impl pallet_zkverify_starky::Config for Test {
 	type RuntimeEvent = RuntimeEvent;
 	type WeightInfo = crate::weights::DefaultWeight;
 }

 pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = system::GenesisConfig::<Test>::default().build_storage().unwrap();
 	let mut ext = sp_io::TestExternalities::new(storage);
 	ext.execute_with(|| {
		System::set_block_number(1);
	});
 	ext
 }

