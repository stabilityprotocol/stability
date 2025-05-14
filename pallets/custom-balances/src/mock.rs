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

use frame_support::pallet_prelude::Weight;
use frame_support::parameter_types;
use frame_support::traits::{Everything, StorageInstance};
use sp_core::{H160, H256, U256};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;
use std::str::FromStr;

pub type AccountId = stbl_core_primitives::AccountId;

pub type Balance = u128;
type Block = frame_system::mocking::MockBlock<Test>;

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

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub const GasLimitPovSizeRatio: u64 = 15;
	pub const SuicideQuickClearLimit: u32 = 64;
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
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
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
	pub enum Test {
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

	fn transfer(_from: H160, _to: H160, _value: U256) -> Result<(), Self::Error> {
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.expect("frame_system::GenesisConfig::build_storage should success");

	pallet_evm::GenesisConfig::<Test> {
		accounts: Default::default(),
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.expect("pallet_evm::GenesisConfig::assimilate_storage should success");

	t.into()
}
