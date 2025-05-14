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

use core::str::FromStr;

use frame_support::{
	construct_runtime,
	pallet_prelude::{StorageValue, ValueQuery},
	parameter_types,
	traits::{Everything, StorageInstance},
	weights::Weight,
};
use frame_system::{EnsureSigned, RawOrigin};
use pallet_evm::{EvmConfig, IdentityAddressMapping};
use sp_core::{ConstU32, H160, H256};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;
use sp_std::vec;
use std::collections::BTreeMap;

pub type AccountId = H160;
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
	pub NotSupportedToken:H160 = H160::from_str("0xdAC17F958D2ee523a2206206994597C13D831123").expect("invalid address");
}

pub struct MockSupportedTokensManager;

struct MockStorageInstance;

impl StorageInstance for MockStorageInstance {
	fn pallet_prefix() -> &'static str {
		"MockSupportedTokensManager"
	}

	const STORAGE_PREFIX: &'static str = "Tokens";
}

parameter_types! {
	pub MockInitialTokens:Vec<H160> = vec![MockDefaultFeeToken::get(), MeaninglessTokenAddress::get()];
}
type MockStorageTokens =
	StorageValue<MockStorageInstance, Vec<H160>, ValueQuery, MockInitialTokens>;

impl pallet_supported_tokens_manager::SupportedTokensManager for MockSupportedTokensManager {
	fn is_supported_token(token: H160) -> bool {
		MockStorageTokens::get().contains(&token)
	}

	fn get_supported_tokens() -> Vec<H160> {
		MockStorageTokens::get()
	}

	fn add_supported_token(_token: H160, _slot: H256) -> Result<(), Self::Error> {
		MockStorageTokens::mutate(|x| x.push(_token));
		Ok(())
	}

	fn remove_supported_token(_token: H160) -> Result<(), Self::Error> {
		MockStorageTokens::mutate(|x| x.retain(|t| t != &_token));
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

impl crate::Config for Runtime {
	type SupportedTokensManager = MockSupportedTokensManager;
	type SimulatorRunner = pallet_evm::runner::stack::Runner<Self>;
}

pub struct EnsureAddressLinkedOrTruncated;

impl<OuterOrigin> pallet_evm::EnsureAddressOrigin<OuterOrigin> for EnsureAddressLinkedOrTruncated
where
	OuterOrigin: Into<Result<RawOrigin<AccountId>, OuterOrigin>> + From<RawOrigin<AccountId>>,
{
	type Success = AccountId;

	fn try_address_origin(address: &H160, origin: OuterOrigin) -> Result<AccountId, OuterOrigin> {
		origin.into().and_then(|o| match o {
			RawOrigin::Signed(who) if Into::<H160>::into(who).eq(address) => Ok(who),
			r => Err(OuterOrigin::from(r)),
		})
	}
}

static LONDON_CONFIG: EvmConfig = EvmConfig::london();

parameter_types! {
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub EVMChainId: u64 = 1;
	pub BlockGasLimit: U256 = U256::MAX;
	pub const GasLimitPovSizeRatio: u64 = 15;
	pub const SuicideQuickClearLimit: u32 = 64;
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressLinkedOrTruncated;
	type WithdrawOrigin = EnsureAddressLinkedOrTruncated;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = EVMChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type FindAuthor = ();
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
	type SuicideQuickClearLimit = SuicideQuickClearLimit;

	fn config() -> &'static pallet_evm::EvmConfig {
		&LONDON_CONFIG
	}
}

impl pallet_root_controller::Config for Runtime {
	type ControlOrigin = EnsureSigned<H160>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: pallet_ethereum::PostLogContent = pallet_ethereum::PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		ValidatorFeeSelector: crate,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
		RootController: pallet_root_controller
	}
);

/// ERC20 metadata for the native token.
pub(crate) struct ExtBuilder {}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {}
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		let initial_default_conversion_rate_controller =
			crate::GenesisConfig::<Runtime>::default().initial_default_conversion_rate_controller;

		pallet_evm::GenesisConfig::<Runtime> {
			accounts: {
				let mut map = BTreeMap::new();
				let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];
				map.insert(
					initial_default_conversion_rate_controller,
					fp_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1000000000000000000u128),
						storage: BTreeMap::new(),
						code: revert_bytecode,
					},
				);
				map
			},
			_marker: Default::default(),
		}
		.assimilate_storage(&mut t)
		.expect("pallet_evm GenesisConfig is valid; qed");

		crate::GenesisConfig::<Runtime> {
			initial_default_conversion_rate_controller: initial_default_conversion_rate_controller,
			_config: Default::default(),
		}
		.assimilate_storage(&mut t)
		.expect("crate GenesisConfig is valid; qed");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
