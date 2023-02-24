#![cfg(test)]

use super::*;
use crate as map_svm_evm;

use frame_support::traits::{ConstU32, ConstU64, Contains};
use sp_core::H256;

use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

pub struct BlockEverything;
impl Contains<RuntimeCall> for BlockEverything {
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

impl frame_system::Config for Test {
	type BaseCallFilter = BlockEverything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

frame_support::construct_runtime!(
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic, {
			System: frame_system,
			MapSvmEvm: map_svm_evm
		}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	t.into()
}
