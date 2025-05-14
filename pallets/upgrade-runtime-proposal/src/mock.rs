// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use super::*;
use crate as pallet_upgrade_runtime_proposal;

use frame_support::parameter_types;
use frame_support::traits::{ConstU32, ConstU64, Contains};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::generic;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;
use sp_version::RuntimeVersion;

type Block = frame_system::mocking::MockBlock<Test>;

pub struct BlockEverything;
impl Contains<RuntimeCall> for BlockEverything {
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

parameter_types! {
	pub Version: RuntimeVersion = RuntimeVersion {
		spec_name: sp_version::create_runtime_str!("test"),
		impl_name: sp_version::create_runtime_str!("system-test"),
		authoring_version: 1,
		spec_version: 1,
		impl_version: 1,
		apis: sp_version::create_apis_vec!([]),
		transaction_version: 1,
		state_version: 1,
	};
}

impl frame_system::Config for Test {
	type BaseCallFilter = BlockEverything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type RuntimeTask = ();
	type Nonce = u64;
	type Block = Block;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub const MaxSizeOfCode: u32 = 10 * 1024 * 1024; // 10 MB
}

impl pallet_upgrade_runtime_proposal::Config for Test {
	type ControlOrigin = EnsureRoot<u64>;
	type MaxSizeOfCode = MaxSizeOfCode;
}

frame_support::construct_runtime!(
	pub enum Test {
			System: frame_system,
			UpgradeRuntimeProposal: pallet_upgrade_runtime_proposal
		}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	t.into()
}

pub fn assert_runtime_updated_digest(num: usize) {
	assert_eq!(
		System::digest()
			.logs
			.into_iter()
			.filter(|item| *item == generic::DigestItem::RuntimeEnvironmentUpdated)
			.count(),
		num,
		"Incorrect number of Runtime Updated digest items",
	);
}
