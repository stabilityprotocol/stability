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

use std::str::FromStr;

use frame_support::traits::GenesisBuild;
use frame_support::{construct_runtime, parameter_types, traits::Everything, weights::Weight};
use frame_system::EnsureRoot;
use sp_runtime::traits::Convert;
use std::collections::BTreeMap;

use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use pallet_session::{SessionHandler, ShouldEndSession};
use precompile_utils::{
	precompile_set::*,
	testing::{CryptoAlith, MockAccount},
};
use sp_core::{H160, H256, U256};
use sp_runtime::{
	impl_opaque_keys,
	testing::UintAuthorityId,
	traits::{BlakeTwo256, IdentityLookup, OpaqueKeys},
};

impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy: UintAuthorityId,
	}
}

impl From<UintAuthorityId> for MockSessionKeys {
	fn from(dummy: UintAuthorityId) -> Self {
		Self { dummy }
	}
}

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
	pub MockDefaultFeeToken: H160 = H160::from_str("0xDc2B93f3291030F3F7a6D9363ac37757f7AD5C43").expect("invalid address");
	pub UnpermissionedAccount:H160 = H160::from_str("0x1000000000000000000000000000000000000000").expect("invalid address");
	pub UnpermissionedAccount2:H160 = H160::from_str("0x2000000000000000000000000000000000000000").expect("invalid address");
	pub MeaninglessTokenAddress: H160 = H160::from_str("0x3000000000000000000000000000000000000000").expect("invalid address");
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

impl pallet_validator_fee_selector::Config for Runtime {
	type SupportedTokensManager = MockSupportedTokensManager;
	type SimulatorRunner = pallet_evm::runner::stack::Runner<Self>;
}

parameter_types! {
	pub DefaultOwner : H160 = H160::from_str("0x2dEA828C816cC4D7CF195E0D220CB75354f47F2F").unwrap();
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	PrecompileAt<
		AddressU64<1>,
		ValidatorFeeManagerPrecompile<R, pallet_validator_fee_selector::Pallet<R>, DefaultOwner>,
	>,
>;

pub type PCall = ValidatorFeeManagerPrecompileCall<
	Runtime,
	pallet_validator_fee_selector::Pallet<Runtime>,
	DefaultOwner,
	(),
>;

parameter_types! {
		pub BlockGasLimit: U256 = U256::max_value();
		pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
		pub const WeightPerGas: Weight = Weight::from_ref_time(1);
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
}

pub struct TestSessionHandler;
impl SessionHandler<AccountId> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[sp_runtime::KeyTypeId(*b"ecds")];
	fn on_genesis_session<T: OpaqueKeys>(_validators: &[(AccountId, T)]) {}
	fn on_new_session<T: OpaqueKeys>(
		_changed: bool,
		_validators: &[(AccountId, T)],
		_queued_validators: &[(AccountId, T)],
	) {
	}
	fn on_disabled(_validator_index: u32) {}
	fn on_before_session_ending() {}
}

pub struct TestShouldEndSession;
impl ShouldEndSession<BlockNumber> for TestShouldEndSession {
	fn should_end_session(_now: BlockNumber) -> bool {
		true
	}
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

pub struct MockSessionBlockManager;
impl pallet_validator_set::SessionBlockManager<BlockNumber> for MockSessionBlockManager {
	fn session_start_block(session_index: sp_staking::SessionIndex) -> BlockNumber {
		session_index as BlockNumber
	}
}
pub struct MockFindAuthor;
impl frame_support::traits::FindAuthor<AccountId> for MockFindAuthor {
	fn find_author<'a, I>(_digests: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
	{
		Some(AccountId::from_u64(1))
	}
}

pub struct AccountIdOfValidator;
impl Convert<UintAuthorityId, AccountId> for AccountIdOfValidator {
	fn convert(a: UintAuthorityId) -> AccountId {
		MockAccount::from_u64(a.0)
	}
}

parameter_types! {
	pub const MinAuthorities: u32 = 1u32;
	pub const MaxKeys: u32 = 1000u32;
}
impl pallet_validator_set::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type AddRemoveOrigin = EnsureRoot<AccountId>;

	type MinAuthorities = MinAuthorities;

	type SessionBlockManager = MockSessionBlockManager;

	type FindAuthor = MockFindAuthor;

	type AuthorityId = UintAuthorityId;

	type AccountIdOfValidator = AccountIdOfValidator;

	type MaxKeys = MaxKeys;
}

impl pallet_session::Config for Runtime {
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_validator_set::ValidatorOf<Self>;
	type ShouldEndSession = TestShouldEndSession;
	type NextSessionRotation = ();
	type SessionManager = ValidatorSet;
	type SessionHandler = TestSessionHandler;
	type Keys = MockSessionKeys;
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
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
		ValidatorFeeSelector: pallet_validator_fee_selector,
		ValidatorSet: pallet_validator_set,
		Session: pallet_session,
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

parameter_types! {
	pub NonCryptoAlith: H160 = H160::from_low_u64_be(0);
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let config = pallet_validator_fee_selector::GenesisConfig {
			initial_default_conversion_rate_controller: H160::from_str(
				"0x444212d6E4827893A70d19921E383130281Cda4a",
			)
			.expect("invalid address"),
		};

		let custom_controller = MeaninglessTokenAddress::get();

		<pallet_evm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_evm::GenesisConfig {
				accounts: {
					let mut map = BTreeMap::new();
					let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];
					map.insert(
						custom_controller,
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

		<pallet_validator_fee_selector::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&config,
			&mut t,
		)
		.expect("Pallet validator fee selector storage can be assimilated");

		pallet_validator_set::GenesisConfig::<Runtime> {
			initial_validators: vec![CryptoAlith.into()],
			max_epochs_missed: U256::max_value(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet validator set storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
