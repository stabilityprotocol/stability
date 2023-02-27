use super::*;

use frame_support::{
	construct_runtime, parameter_types, traits::Everything, traits::GenesisBuild, weights::Weight,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use sp_core::{H160, H256, U256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	AccountId32, MultiSignature,
};

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
pub type Block = frame_system::mocking::MockBlock<Runtime>;

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}

impl pallet_balances::Config for Runtime {
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

pub type Precompiles<R> =
	PrecompileSetBuilder<R, PrecompileAt<AddressU64<1>, MapSvmEvmControllerPrecompile<R>>>;

pub type PCall = MapSvmEvmControllerPrecompileCall<Runtime>;

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = pallet_evm::EnsureAddressNever<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
	type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
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

impl pallet_map_svm_evm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		MapSvmEvm: pallet_map_svm_evm::{Pallet, Call, Storage, Event<T>},
	}
);

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	linked_accounts: Vec<(AccountId, H160)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			linked_accounts: vec![],
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn with_linked_accounts(mut self, linked_accounts: Vec<(AccountId, H160)>) -> Self {
		self.linked_accounts = linked_accounts;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		pallet_map_svm_evm::GenesisConfig::<Runtime> {
			linked_accounts: self.linked_accounts,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet map svm evm storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
