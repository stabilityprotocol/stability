#![cfg(test)]

use core::str::FromStr;

use runner::Runner as StabilityRunner;

use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, GenesisBuild},
	weights::Weight,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use sp_core::{H160, H256, U256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use std::collections::BTreeMap;

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type BlockNumber = u32;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub struct MockUserFeeTokenController;
impl pallet_user_fee_selector::UserFeeTokenController for MockUserFeeTokenController {
	type Error = ();

	fn balance_of(_token: H160) -> sp_core::U256 {
		return U256::from(1000000);
	}

	fn get_user_fee_token(_account: H160) -> H160 {
		return ERC20SlotZero::get();
	}

	fn set_user_fee_token(_account: H160, _token: H160) -> Result<(), Self::Error> {
		Ok(())
	}
}

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
	type Lookup = IdentityLookup<AccountId>;
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

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

pub struct MockDNTFeeController;
impl runner::OnChargeDecentralizedNativeTokenFee for MockDNTFeeController {
	type Error = ();

	fn get_transaction_fee_token(_from: H160) -> H160 {
		Default::default()
	}

	fn get_transaction_conversion_rate(_validator: H160, _token: H160) -> (U256, U256) {
		(1.into(), 1.into())
	}

	fn get_fee_vault() -> H160 {
		Default::default()
	}

	fn withdraw_fee(
		_from: H160,
		_token: H160,
		_conversion_rate: (U256, U256),
		_amount: U256,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn correct_fee(
		_from: H160,
		_token: H160,
		_conversion_rate: (U256, U256),
		_paid_amount: U256,
		_actual_amount: U256,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn pay_fees(
		_token: H160,
		_conversion_rate: (U256, U256),
		_actual_amount: U256,
		_validator: H160,
		_to: Option<H160>,
	) -> Result<(U256, U256), Self::Error> {
		Ok((Default::default(), Default::default()))
	}
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
	pub ERC20SlotZero: H160 = H160::from_str("0x22D598E0a9a1b474CdC7c6fBeA0B4F83E12046a9").unwrap();
	pub ZeroSlot : H256 = H256::from_low_u64_be(0);
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = StabilityRunner<Self, MockDNTFeeController, MockUserFeeTokenController>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type StateRoot = ();
}

pub struct MockERC20Manager;
impl pallet_erc20_manager::ERC20Manager for MockERC20Manager {
	type Error = ();

	fn balance_of(_token: H160, _payer: H160) -> U256 {
		U256::max_value()
	}

	fn withdraw_amount(_: H160, _: H160, amount: U256) -> Result<U256, Self::Error> {
		Ok(amount.into())
	}

	fn deposit_amount(_: H160, _: H160, amount: U256) -> Result<U256, Self::Error> {
		Ok(amount.into())
	}
}

impl crate::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type ERC20Manager = MockERC20Manager;
	type DNTFeeController = MockDNTFeeController;
}

// Configure a mock runtime to test the pallet.
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
		Ethereum: pallet_ethereum,
		MetaTransactions: crate
	}
);

parameter_types! {
	pub RawTransaction0: Vec<u8> = hex::decode("f86b80843b9aca0082520894f0a57f274781b0ed17f2ae5f1709a0c360cbe489880de0b6b3a7640000801ba0e4cb4bbd414f044046b741e5ec6d874ca73a178e6e84e5fd2b6e2f6d0bcac869a05f334d87f56a77435a2547188a935a37358b5e36e76789ac5278bd0963a931ae").unwrap();
	pub RawTransaction1: Vec<u8> = hex::decode("f86b01843b9aca0082520894f0a57f274781b0ed17f2ae5f1709a0c360cbe489880de0b6b3a7640000801ca029343cf22be72d879c6392a2ff5dd7e3675b6b70ffc7b7a15529ed9bdad33668a04d5de0209d299c37af1642ea17ca43fdcf0b07d4aa44fe87e338b4fe9b2fa995").unwrap();
	pub MetaTransaction0Signature: Vec<u8> = hex::decode("bce0eab34bf451c7ab98ecab110220a77f8d15b1d3c641cb02eb85ae79dddec9284b01839010ffc65dff384b17676bb1875dda86cb313b9bdef1146f2b0fd17f1c").unwrap();
	pub MetaTransaction1Signature: Vec<u8> = hex::decode("fb0482cfa593aa03aaa1d7c2e5a07269d65a4a589b14c109de8d8ca0c268efca198d1d898e7022002e34d91ced41deda8ae526856a8e23d100ab92e5dbb948c31b").unwrap();
	pub Sponsor: H160 = H160::from_str("0xaf537bd156c7e548d0bf2cd43168dabf7af2feb5").unwrap();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();
	let config = pallet_evm::GenesisConfig {
		accounts: BTreeMap::new(),
	};
	<pallet_evm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(&config, &mut t)
		.unwrap();

	let eth_config: pallet_ethereum::GenesisConfig = Default::default();

	<pallet_ethereum::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&eth_config,
		&mut t,
	)
	.unwrap();

	t.into()
}
