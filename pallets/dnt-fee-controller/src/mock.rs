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

#![cfg(test)]

use frame_support::pallet_prelude::{ValueQuery, Weight};
use frame_support::traits::{Contains, Everything, StorageInstance};
use frame_support::{parameter_types, Blake2_128};
use sp_core::{H160, H256, U256};
use sp_runtime::BuildStorage;
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use std::str::FromStr;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Balance = u128;
type Block = frame_system::mocking::MockBlock<Test>;

pub struct BlockEverything;
impl Contains<RuntimeCall> for BlockEverything {
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
	pub FeeVaultAddress: H160 = H160::from_low_u64_be(0);
	pub MeaninglessTokenAddress : H160 = H160::from_str("0x22D598E0a9a1b474CdC7c6fBeA0B4F83E12046a9").unwrap();
	pub MeaninglessAddress : H160 = H160::from_low_u64_be(1);
	pub MeaninglessAddress2 : H160 = H160::from_low_u64_be(2);
	pub MeaninglessConversionRate : (U256, U256) = (1.into(), 1.into());
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
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

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub const GasLimitPovSizeRatio: u64 = 15;
	pub const SuicideQuickClearLimit: u32 = 64;
}

impl pallet_evm_chain_id::Config for Test {}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
	type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
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
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
}

impl pallet_fee_rewards_vault::Config for Test {}

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		EVM: pallet_evm,
		EVMChainId: pallet_evm_chain_id,
		FeeRewardsVault: pallet_fee_rewards_vault,
		DNTFeeController: crate,
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
		Default::default()
	}
	fn transfer(_from: H160, _to: H160, _value: U256) -> Result<(), Self::Error> {
		Ok(())
	}
}

pub struct MockValidatorTokenController;
impl pallet_validator_fee_selector::ValidatorFeeTokenController for MockValidatorTokenController {
	type Error = ();

	fn validator_supports_fee_token(_validator: H160, _token: H160) -> bool {
		true
	}

	fn update_fee_token_acceptance(
		_validator: H160,
		_token: H160,
		_support: bool,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn conversion_rate(_s: H160, _validator: H160, _token: H160) -> (U256, U256) {
		MeaninglessConversionRate::get()
	}

	fn conversion_rate_controller(_validator: H160) -> H160 {
		Default::default()
	}

	fn update_conversion_rate_controller(
		_validator: H160,
		_conversion_rate_controller: H160,
	) -> Result<(), Self::Error> {
		Ok(Default::default())
	}

	fn update_default_controller(_controller: H160) -> Result<(), Self::Error> {
		Ok(())
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

pub type MockCallsStorage = frame_support::pallet_prelude::StorageMap<
	MockPrefix,
	Blake2_128,
	String,
	Vec<(H160, H160, U256)>,
	ValueQuery,
	DefaultMockCallsVec,
>;

pub type MockFailsStorage =
	frame_support::pallet_prelude::StorageValue<MockPrefix, bool, ValueQuery, DefaultFails>;

pub struct MockERC20Manager;
impl pallet_erc20_manager::ERC20Manager for MockERC20Manager {
	type Error = ();

	fn balance_of(_token: H160, _payer: H160) -> U256 {
		Default::default()
	}

	fn withdraw_amount(token: H160, payer: H160, amount: U256) -> Result<U256, Self::Error> {
		if MockFailsStorage::get() {
			return Err(());
		}
		let mut array = MockCallsStorage::get("withdraw_amount");
		array.push((token, payer, amount));
		MockCallsStorage::insert("withdraw_amount", array);
		Ok(0.into())
	}

	fn deposit_amount(token: H160, payee: H160, amount: U256) -> Result<U256, Self::Error> {
		if MockFailsStorage::get() {
			return Err(());
		}
		let mut array = MockCallsStorage::get("deposit_amount");
		array.push((token, payee, amount));
		MockCallsStorage::insert("deposit_amount", array);
		Ok(0.into())
	}
}

impl crate::Config for Test {
	type UserFeeTokenController = MockUserFeeTokenController;
	type ValidatorTokenController = MockValidatorTokenController;
	type ERC20Manager = MockERC20Manager;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	pallet_evm::GenesisConfig::<Test> {
		accounts: Default::default(),
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.expect("failed to build pallet_evm genesis");

	crate::GenesisConfig::<Test> {
		fee_vault_precompile_address: FeeVaultAddress::get(),
		validator_percentage: 50.into(),
		_config: Default::default(),
	}
	.assimilate_storage(&mut t)
	.expect("Failed to build pallet_dnt_fee_controller genesis");

	t.into()
}
