// Copyright 2023 Stability Solutions.
// This file is part of Stability.

// Stability is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stability is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stability.  If not, see <http://www.gnu.org/licenses/>.

//! Testing utilities.

use super::*;

use core::str::FromStr;

use frame_support::{
	construct_runtime,
	pallet_prelude::{StorageValue, ValueQuery},
	parameter_types,
	traits::{Everything, GenesisBuild, StorageInstance},
	weights::Weight,
};
use frame_system::{EnsureSigned, RawOrigin};
use pallet_evm::{EvmConfig, IdentityAddressMapping};
use sp_core::{H160, H256, ConstU32};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_std::vec;
use std::collections::BTreeMap;

pub type AccountId = H160;
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
	type MaxHolds = ();
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub MockDefaultFeeToken: H160 = H160::from_str("0xDc2B93f3291030F3F7a6D9363ac37757f7AD5C43").expect("invalid address");
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
	pub WeightPerGas : Weight = Weight::from_ref_time(1);
	pub EVMChainId: u64 = 1;
	pub BlockGasLimit: U256 = U256::MAX;
	pub const GasLimitPovSizeRatio: u64 = 15;
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
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
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
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		let initial_default_conversion_rate_controller =
			crate::GenesisConfig::default().initial_default_conversion_rate_controller;

		<pallet_evm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_evm::GenesisConfig {
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
			},
			&mut t,
		)
		.unwrap();

		<crate::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&crate::GenesisConfig::default(),
			&mut t,
		)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
