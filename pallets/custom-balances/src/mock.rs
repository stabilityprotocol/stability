#![cfg(test)]

use frame_support::pallet_prelude::Weight;
use frame_support::traits::{Everything, GenesisBuild, StorageInstance};
use sp_core::{H160, H256, U256};
use std::str::FromStr;

use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

use frame_support::parameter_types;

pub type AccountId = stbl_core_primitives::AccountId;

pub type Balance = u128;
pub type BlockNumber = u32;

type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
	pub MeaninglessTokenAddress : H160 = H160::from_str("0x22D598E0a9a1b474CdC7c6fBeA0B4F83E12046a9").unwrap();
	pub ZeroAddress : H160 = H160::from_low_u64_be(0);
	pub OneAddress : H160 = H160::from_low_u64_be(1);
}

impl frame_system::Config for Test {
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

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
	pub const GasLimitPovSizeRatio: u64 = 15;
}

impl pallet_evm_chain_id::Config for Test {}

pub struct IdentityAddressMapping;
impl pallet_evm::AddressMapping<AccountId> for IdentityAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		address.into()
	}
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = CustomBalances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = EVMChainId;
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

pub struct AccountIdToH160Mapping;
impl crate::AccountIdMapping<Test> for AccountIdToH160Mapping {
	fn into_evm_address(address: &AccountId) -> H160 {
		(*address).into()
	}
}

impl crate::Config for Test {
	type UserFeeTokenController = MockUserFeeTokenController;
	type AccountIdMapping = AccountIdToH160Mapping;
}

frame_support::construct_runtime!(
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic, {
			System: frame_system,
			Timestamp: pallet_timestamp,
			EVM: pallet_evm,
			EVMChainId: pallet_evm_chain_id,
			CustomBalances: crate
		}
);

pub struct MockUserFeeTokenController;
impl pallet_user_fee_selector::UserFeeTokenController for MockUserFeeTokenController {
	type Error = ();
	fn get_user_fee_token(_account: H160) -> H160 {
		MeaninglessTokenAddress::get()
	}

	fn set_user_fee_token(_account: H160, _token: H160) -> Result<(), Self::Error> {
		Ok(())
	}

	fn balance_of(_account: H160) -> U256 {
		if _account.eq(&H160::from_low_u64_be(0)) {
			U256::from(1)
		} else {
			U256::from(0)
		}
	}
}

pub struct MockPrefix;
impl StorageInstance for MockPrefix {
	fn pallet_prefix() -> &'static str {
		"Mock"
	}

	const STORAGE_PREFIX: &'static str = "Mock";
}

parameter_types! {
	pub DefaultMockCallsVec: Vec<(H160, H160, U256)> = Vec::new();
	pub DefaultFails: bool = false;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let evm_config = pallet_evm::GenesisConfig {
		accounts: Default::default(),
	};

	<pallet_evm::GenesisConfig as GenesisBuild<Test>>::assimilate_storage(&evm_config, &mut t)
		.unwrap();

	t.into()
}
