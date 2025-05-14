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

use frame_support::traits::{ConstU32, ConstU64, Contains};
use frame_support::{parameter_types, weights::Weight};
use frame_system::EnsureRoot;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::precompile_set::*;
use sp_core::H256;
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
	pub DefaultOwner: H160 = H160::from_str("0xa58482131a8d67725e996af72D91A849AcC0F4A1").expect("invalid address");
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	PrecompileAt<AddressU64<1>, UpgradeRuntimeControllerPrecompile<R, DefaultOwner>>,
>;

pub type PCall = UpgradeRuntimeControllerPrecompileCall<Test, DefaultOwner>;

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
pub type AccountId = stbl_core_primitives::AccountId;

impl frame_system::Config for Test {
	type BaseCallFilter = BlockEverything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
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
	type ControlOrigin = EnsureRoot<AccountId>;
	type MaxSizeOfCode = MaxSizeOfCode;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

pub struct IdentityAddressMapping;
impl pallet_evm::AddressMapping<AccountId> for IdentityAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		address.into()
	}
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Test> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub const GasLimitPovSizeRatio: u64 = 15;
	pub const SuicideQuickClearLimit: u32 = 64;
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

pub type Balance = u128;

impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
}

type TechCommitteeInstance = pallet_collective::Instance1;

use frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS;
use stbl_core_primitives::BlockNumber;

// Block time
pub const MILLISECS_PER_BLOCK: u64 = 2000;

/// How much of time of block time is consumed (at most) in computing normal extrinsics
const COMPUTATION_BLOCK_TIME_RATIO: (u64, u64) = (2, 3); // 2 third parts of the block time

const COMPUTATION_POWER_MULTIPLIER: u64 = 6; // 6 times more computation power than normal

// how much weight for normal extrinsics could be processed in a block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_MILLIS
		* MILLISECS_PER_BLOCK
		* COMPUTATION_POWER_MULTIPLIER
		* COMPUTATION_BLOCK_TIME_RATIO.0
		/ COMPUTATION_BLOCK_TIME_RATIO.1,
	u64::MAX,
);

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 120;
	pub const CouncilMaxProposals: u32 = 2;
	pub const CouncilMaxMembers: u32 = 2;
	pub const MaxProposalWeight: Weight = MAXIMUM_BLOCK_WEIGHT;
}

impl pallet_collective::Config<TechCommitteeInstance> for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Self>;
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxProposalWeight;
}

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		UpgradeRuntimeProposal: pallet_upgrade_runtime_proposal,
		Timestamp: pallet_timestamp,
		EVM: pallet_evm,
		Balances: pallet_balances,
		TechCommitteeCollective: pallet_collective::<Instance1>,
	}
);

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		pallet_collective::GenesisConfig::<Test, TechCommitteeInstance> {
			members: vec![],
			phantom: Default::default(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet collective storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
