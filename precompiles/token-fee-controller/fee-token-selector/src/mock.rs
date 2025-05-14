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

//! Testing utilities.

use super::*;

use std::str::FromStr;

use frame_support::{construct_runtime, parameter_types, traits::Everything, weights::Weight};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use sp_core::{H160, H256, U256};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;

pub type AccountId = MockAccount;
pub type Balance = u128;
pub type Block = frame_system::mocking::MockBlock<Runtime>;

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
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

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
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
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub MockDefaultFeeToken: H160 = H160::from_str("0x261FB2d971eFBBFd027A9C9Cebb8548Cf7d0d2d5").expect("invalid address");
	pub MeaninglessTokenAddress:H160 = H160::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").expect("invalid address");
}

pub struct MockSupportedTokensManager;

impl pallet_supported_tokens_manager::SupportedTokensManager for MockSupportedTokensManager {
	fn is_supported_token(token: H160) -> bool {
		token == MockDefaultFeeToken::get() || token == MeaninglessTokenAddress::get()
	}

	fn get_supported_tokens() -> Vec<H160> {
		vec![MockDefaultFeeToken::get(), MeaninglessTokenAddress::get()]
	}

	fn add_supported_token(_token: H160, _slot: H256) -> Result<(), Self::Error> {
		Ok(())
	}

	fn remove_supported_token(_token: H160) -> Result<(), Self::Error> {
		Ok(())
	}

	fn get_token_balance_slot(_token: H160) -> Option<H256> {
		Some(H256::default())
	}

	type Error = ();

	fn get_default_token() -> H160 {
		MockDefaultFeeToken::get()
	}

	fn set_default_token(_token: H160) -> Result<(), Self::Error> {
		Ok(())
	}
}

pub struct MockERC20Manager;

impl pallet_erc20_manager::ERC20Manager for MockERC20Manager {
	type Error = ();

	fn balance_of(_token: H160, _payer: H160) -> sp_core::U256 {
		Default::default()
	}

	fn withdraw_amount(
		_token: H160,
		_payer: H160,
		_amount: sp_core::U256,
	) -> Result<sp_core::U256, Self::Error> {
		Ok(Default::default())
	}

	fn deposit_amount(
		_token: H160,
		_payee: H160,
		_amount: sp_core::U256,
	) -> Result<sp_core::U256, Self::Error> {
		Ok(Default::default())
	}
}

impl pallet_user_fee_selector::Config for Runtime {
	type SupportedTokensManager = MockSupportedTokensManager;
	type ERC20Manager = MockERC20Manager;
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	PrecompileAt<AddressU64<1>, FeeTokenPrecompile<R, pallet_user_fee_selector::Pallet<R>>>,
>;

pub type PCall = FeeTokenPrecompileCall<Runtime, pallet_user_fee_selector::Pallet<Runtime>, ()>;

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub const GasLimitPovSizeRatio: u64 = 15;
	pub const SuicideQuickClearLimit: u32 = 64;
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
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
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		UserFeeTokenSelector: pallet_user_fee_selector,
	}
);

/// ERC20 metadata for the native token.
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
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
