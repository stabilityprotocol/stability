// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use super::*;

use frame_support::traits::{ConstU32, ConstU64, Contains};
use frame_support::{parameter_types, weights::Weight};
use frame_system::EnsureRoot;
use hex::FromHex;
use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot, IdentityAddressMapping};
use pallet_session::{SessionHandler, ShouldEndSession};
use precompile_utils::precompile_set::*;
use sp_core::H256;
use sp_runtime::testing::UintAuthorityId;
use sp_runtime::traits::{Convert, OpaqueKeys};
use sp_runtime::BuildStorage;
use sp_runtime::{impl_opaque_keys, KeyTypeId};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	RuntimeAppPublic,
};
use sp_std::vec;
use sp_version::RuntimeVersion;
use std::collections::BTreeMap;

type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub struct BlockEverything;
impl Contains<RuntimeCall> for BlockEverything {
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

type BlockNumber = u64;
type AccountId = H160;

parameter_types! {
	pub DefaultOwner: H160 = H160::from_str("0xa58482131a8d67725e996af72D91A849AcC0F4A1").expect("invalid address");
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	PrecompileAt<
		AddressU64<1>,
		FeeRewardsVaultControllerPrecompile<R, DefaultOwner>,
		(
			CallableByContract,
			CallableByPrecompile,
			SubcallWithMaxNesting<1>,
		),
	>,
>;

pub type PCall = FeeRewardsVaultControllerPrecompileCall<Test, DefaultOwner>;

parameter_types! {
	pub Version: RuntimeVersion = RuntimeVersion {
		spec_name: sp_version::create_runtime_str!("test"),
		impl_name: sp_version::create_runtime_str!("system-test"),
		authoring_version: 1,
		spec_version: 1,
		impl_version: 1,
		apis: sp_version::create_apis_vec!([]),
		transaction_version: 1,
		state_version: 1,
	};
}

impl frame_system::Config for Test {
	type BaseCallFilter = BlockEverything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
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

pub struct NumberAddressMapping;

impl AddressMapping<u64> for NumberAddressMapping {
	fn into_account_id(address: H160) -> u64 {
		let address_bytes: [u8; 8] = (*address.as_fixed_bytes())[12..].try_into().unwrap();
		u64::from_be_bytes(address_bytes)
	}
}

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub PrecompilesValue: Precompiles<Test> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE)
	};
	pub const SuicideQuickClearLimit: u32 = 64;
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<H160>;
	type WithdrawOrigin = EnsureAddressNever<H160>;
	type AddressMapping = IdentityAddressMapping;
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
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

pub type Balance = u128;

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

impl pallet_fee_rewards_vault::Config for Test {}

impl pallet_dnt_fee_controller::Config for Test {
	type ERC20Manager = pallet_erc20_manager::Pallet<Self>;
	type UserFeeTokenController = pallet_user_fee_selector::Pallet<Self>;
	type ValidatorTokenController = pallet_validator_fee_selector::Pallet<Self>;
}

impl pallet_erc20_manager::Config for Test {
	type SupportedTokensManager = pallet_supported_tokens_manager::Pallet<Self>;
}

parameter_types! {
	pub DefaultFeeToken: H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}
impl pallet_user_fee_selector::Config for Test {
	type SupportedTokensManager = pallet_supported_tokens_manager::Pallet<Self>;
	type ERC20Manager = pallet_erc20_manager::Pallet<Self>;
}

impl pallet_validator_fee_selector::Config for Test {
	type SupportedTokensManager = pallet_supported_tokens_manager::Pallet<Self>;
	type SimulatorRunner = pallet_evm::runner::stack::Runner<Self>;
}

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

pub const KEY_ID_A: KeyTypeId = KeyTypeId([4; 4]);
pub const KEY_ID_B: KeyTypeId = KeyTypeId([9; 4]);

#[derive(Debug, Clone, parity_scale_codec::Encode, parity_scale_codec::Decode, PartialEq, Eq)]
pub struct PreUpgradeMockSessionKeys {
	pub a: [u8; 32],
	pub b: [u8; 64],
}

impl OpaqueKeys for PreUpgradeMockSessionKeys {
	type KeyTypeIdProviders = ();

	fn key_ids() -> &'static [KeyTypeId] {
		&[KEY_ID_A, KEY_ID_B]
	}

	fn get_raw(&self, i: KeyTypeId) -> &[u8] {
		match i {
			i if i == KEY_ID_A => &self.a[..],
			i if i == KEY_ID_B => &self.b[..],
			_ => &[],
		}
	}
}

pub struct TestSessionHandler;
impl SessionHandler<H160> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
	fn on_genesis_session<T: OpaqueKeys>(_validators: &[(H160, T)]) {}
	fn on_new_session<T: OpaqueKeys>(
		_changed: bool,
		_validators: &[(H160, T)],
		_queued_validators: &[(H160, T)],
	) {
	}
	fn on_disabled(_validator_index: u32) {}
	fn on_before_session_ending() {}
}

pub struct TestShouldEndSession;
impl ShouldEndSession<u64> for TestShouldEndSession {
	fn should_end_session(_now: u64) -> bool {
		false
	}
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
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
		Some(AccountId::default())
	}
}

pub struct AccountIdOfValidator;
impl Convert<UintAuthorityId, AccountId> for AccountIdOfValidator {
	fn convert(a: UintAuthorityId) -> AccountId {
		AccountId::from_low_u64_be(a.0)
	}
}

parameter_types! {
	pub const MinAuthorities: u32 = 0;
	pub const MaxKeys: u32 = 1000u32;
}
impl pallet_validator_set::Config for Test {
	type RuntimeEvent = RuntimeEvent;

	type AddRemoveOrigin = EnsureRoot<AccountId>;

	type MinAuthorities = MinAuthorities;

	type SessionBlockManager = MockSessionBlockManager;

	type FindAuthor = MockFindAuthor;

	type AuthorityId = UintAuthorityId;

	type MaxKeys = MaxKeys;

	type AccountIdOfValidator = AccountIdOfValidator;
}

impl pallet_session::Config for Test {
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_validator_set::ValidatorOf<Self>;
	type ShouldEndSession = TestShouldEndSession;
	type NextSessionRotation = ();
	type SessionManager = pallet_validator_set::Pallet<Self>;
	type SessionHandler = TestSessionHandler;
	type Keys = MockSessionKeys;
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_supported_tokens_manager::Config for Test {}

frame_support::construct_runtime!(
	pub enum Test {
			System: frame_system,
			FeeRewardsVault: pallet_fee_rewards_vault,
			DNTFeeController: pallet_dnt_fee_controller,
			Timestamp: pallet_timestamp,
			UserFeeSelector: pallet_user_fee_selector,
			ValidatorFeeSelector: pallet_validator_fee_selector,
			SupportedTokensManager: pallet_supported_tokens_manager,
			ERC20Manager: pallet_erc20_manager,
			EVM: pallet_evm,
			Balances: pallet_balances,
			ValidatorSet: pallet_validator_set,
			Session: pallet_session,
		}
);

parameter_types! {
	pub OwnerOfDapp: H160 = H160::from_str("0x2681896781c7b49A68dCBaB833D5544501794D78").expect("invalid address");
	pub NotOwner: H160 = H160::from_str("0xd5A80D493a784883Dbf7f0fd0FED4D3156C0235F").expect("invalid address");
	pub SmartContractWithOwner: H160 = H160::from_str("0x6f533d42ade638B8c3dBE3F6822697Ccd2662615").expect("invalid address");
	pub SmartContractWithOwnerCode: Vec<u8> = Vec::<u8>::from_hex("6080604052732681896781c7b49a68dcbab833d5544501794d785f806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550348015610062575f80fd5b5060ef8061006f5f395ff3fe6080604052348015600e575f80fd5b50600436106026575f3560e01c80638da5cb5b14602a575b5f80fd5b60306044565b604051603b919060a2565b60405180910390f35b5f8054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f608e826067565b9050919050565b609c816086565b82525050565b5f60208201905060b35f8301846095565b9291505056fea264697066735822122083c807511a1b11ff9b84332ab62ec10c76f4d49535c421ba72d35840bdd61ff864736f6c63430008140033").expect("invalid code");
	pub SmartContractWithoutOwner: H160= H160::from_str("0x85D30C2b6273E7d3b6e5B9E3c00860D42e3C25bd").expect("invalid address");
	pub SmartContractWithoutOwnerCode: Vec<u8> = Vec::<u8>::from_hex("6080604052348015600f57600080fd5b506004361060285760003560e01c8063cb7acdd914602d575b600080fd5b60336035565b005b56fea264697066735822122080cab20cbc1d3f2f36c19e83c8902d7ba237ffaac3a82bb14feb8f8aa8cda4f064736f6c63430008120033").expect("invalid code");
	pub Token1: H160 = H160::from_str("0x04f76FD58926D457AAcC69Cf6C1C47FACc8Eee6b").expect("invalid address");
	pub Token2: H160 = H160::from_str("0x6EBfe6dE95D12dbEA550D19239a9cd926d0c06dE").expect("invalid address");
	pub Validators: Vec<H160> = sp_std::vec![H160::from_str("0xaB92667238213A2f616D23BE86F3d285A8c33F67").expect("invalid address"),H160::from_str("0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b").expect("invalid address")];
}
pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(H160, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system`s builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		pallet_evm::GenesisConfig::<Test> {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					SmartContractWithOwner::get(),
					fp_evm::GenesisAccount {
						balance: Default::default(),
						code: SmartContractWithOwnerCode::get(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					SmartContractWithoutOwner::get(),
					fp_evm::GenesisAccount {
						balance: Default::default(),
						code: SmartContractWithoutOwnerCode::get(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
			..Default::default()
		}
		.assimilate_storage(&mut t)
		.expect("Pallet evm storage can be assimilated");

		pallet_dnt_fee_controller::GenesisConfig::<Test> {
			_config: Default::default(),
			fee_vault_precompile_address: SmartContractWithOwner::get(),
			validator_percentage: U256::from(0),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet dnt fee controller storage can be assimilated");

		pallet_validator_set::GenesisConfig::<Test>::assimilate_storage(
			&pallet_validator_set::GenesisConfig::<Test> {
				initial_validators: Validators::get(),
				max_epochs_missed: U256::max_value(),
			},
			&mut t,
		)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
