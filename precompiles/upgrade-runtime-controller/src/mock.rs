use super::*;

use frame_support::traits::{ConstU32, ConstU64, Contains};
use frame_support::{parameter_types, weights::Weight};
use frame_system::EnsureRoot;
use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::precompile_set::*;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use sp_version::RuntimeVersion;

type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

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
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const MaxSizeOfCode: u32 = 10 * 1024 * 1024; // 10 MB
}

impl pallet_upgrade_runtime_proposal::Config for Test {
	type ControlOrigin = EnsureRoot<u64>;
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

pub struct NumberAddressMapping;

impl AddressMapping<u64> for NumberAddressMapping {
	fn into_account_id(address: H160) -> u64 {
		let address_bytes: [u8; 8] = (*address.as_fixed_bytes())[12..].try_into().unwrap();
		u64::from_be_bytes(address_bytes)
	}
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Test> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<u64>;
	type WithdrawOrigin = EnsureAddressNever<u64>;
	type AddressMapping = NumberAddressMapping;
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
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
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
}

frame_support::construct_runtime!(
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic, {
			System: frame_system,
			UpgradeRuntimeProposal: pallet_upgrade_runtime_proposal,
			Timestamp: pallet_timestamp,
			EVM: pallet_evm,
			Balances: pallet_balances,
		}
);

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(u64, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
