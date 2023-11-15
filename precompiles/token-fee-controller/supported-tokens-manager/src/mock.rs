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

use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, GenesisBuild},
	weights::Weight,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use sp_core::{H160, H256, U256};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

pub type AccountId = MockAccount;
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
	pub DefaultOwner: H160 = H160::from_str("0xaf537bd156c7E548D0BF2CD43168dABF7aF2feb5").expect("invalid address");
	pub InitialDefaultTokenFee: H160 = H160::from_str("0x000000035b5e3FF2591dAa4dBC2b3f8Ea5C5b82f").expect("invalid address");
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompileAt<
			AddressU64<1>,
			SupportedTokensManagerPrecompile<
				R,
				pallet_supported_tokens_manager::Pallet<R>,
				DefaultOwner,
			>,
		>,
	),
>;

pub type PCall = SupportedTokensManagerPrecompileCall<
	Runtime,
	pallet_supported_tokens_manager::Pallet<Runtime>,
	DefaultOwner,
	(),
>;

impl pallet_supported_tokens_manager::Config for Runtime {}

parameter_types! {
		pub BlockGasLimit: U256 = U256::max_value();
		pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
		pub const WeightPerGas: Weight = Weight::from_ref_time(1);
		pub const GasLimitPovSizeRatio: u64 = 15;
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
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
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
		SupportedTokensManager: pallet_supported_tokens_manager,
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

		GenesisBuild::<Runtime>::assimilate_storage(
			&pallet_supported_tokens_manager::GenesisConfig {
				initial_default_token: InitialDefaultTokenFee::get(),
				initial_default_token_slot: H256::from_low_u64_be(0),
			},
			&mut t,
		)
		.expect("Pallet supported tokens manager storage failed assimilation");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
