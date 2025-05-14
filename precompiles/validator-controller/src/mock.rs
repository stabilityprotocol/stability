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

use frame_support::{construct_runtime, parameter_types, traits::Everything, weights::Weight};
use frame_system::EnsureRoot;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use pallet_session::*;
use precompile_utils::precompile_set::*;
use sp_core::crypto::key_types::DUMMY;
use sp_core::{H160, H256, U256};
use sp_runtime::impl_opaque_keys;
use sp_runtime::testing::UintAuthorityId;
use sp_runtime::traits::{BlakeTwo256, Convert, IdentityLookup, OpaqueKeys};
use sp_runtime::BuildStorage;
use sp_runtime::KeyTypeId;
use sp_runtime::RuntimeAppPublic;
use std::cell::RefCell;

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

pub type AccountId = stbl_core_primitives::AccountId;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
pub type Block = frame_system::mocking::MockBlock<Runtime>;

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
	pub MeaninglessTokenAddress : H160 = H160::from_str("0x22D598E0a9a1b474CdC7c6fBeA0B4F83E12046a9").unwrap();
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
	type FreezeIdentifier = ();
	type RuntimeFreezeReason = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub const GasLimitPovSizeRatio: u64 = 15;
	pub const SuicideQuickClearLimit: u32 = 64;
}

pub struct IdentityAddressMapping;
impl pallet_evm::AddressMapping<AccountId> for IdentityAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		address.into()
	}
}

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

pub struct AccountIdToH160Mapping;
impl crate::AccountIdMapping<Runtime> for AccountIdToH160Mapping {
	fn into_evm_address(address: &AccountId) -> H160 {
		(*address).into()
	}
}

impl pallet_custom_balances::Config for Runtime {
	type AccountIdMapping = AccountIdToH160Mapping;
	type UserFeeTokenController = MockUserFeeTokenController;
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
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
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

pub struct MockSessionBlockManager;
impl pallet_validator_set::SessionBlockManager<u64> for MockSessionBlockManager {
	fn session_start_block(session_index: sp_staking::SessionIndex) -> u64 {
		session_index as u64
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
	fn convert(_authority_id: UintAuthorityId) -> AccountId {
		stbl_core_primitives::AccountId::default()
	}
}

parameter_types! {
	pub const MinAuthorities: u32 = 0u32;
	pub const MaxKeys: u32 = 1000u32;
}
impl pallet_validator_set::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type AddRemoveOrigin = EnsureRoot<AccountId>;

	type MinAuthorities = MinAuthorities;

	type SessionBlockManager = MockSessionBlockManager;

	type FindAuthor = MockFindAuthor;

	type AuthorityId = UintAuthorityId;

	type MaxKeys = MaxKeys;

	type AccountIdOfValidator = AccountIdOfValidator;
}

thread_local! {
	pub static VALIDATORS: RefCell<Vec<AccountId>> = RefCell::new(vec![AccountId::from_str("0x0000000000000000000000000000000000000000").unwrap(), AccountId::from_str("0x0000000000000000000000000000000000000001").unwrap(), AccountId::from_str("0x0000000000000000000000000000000000000003").unwrap()]);
	pub static NEXT_VALIDATORS: RefCell<Vec<AccountId>> = RefCell::new(vec![AccountId::from_str("0x0000000000000000000000000000000000000000").unwrap(),AccountId::from_str("0x0000000000000000000000000000000000000002").unwrap(), AccountId::from_str("0x0000000000000000000000000000000000000003").unwrap()]);
	pub static AUTHORITIES: RefCell<Vec<AccountId>> =
		RefCell::new(vec![AccountId::from_str("0x0000000000000000000000000000000000000000").unwrap(), AccountId::from_str("0x0000000000000000000000000000000000000001").unwrap(), AccountId::from_str("0x0000000000000000000000000000000000000003").unwrap()]);
	pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
	pub static SESSION_LENGTH: RefCell<BlockNumber> = RefCell::new(2);
	pub static SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
	pub static DISABLED: RefCell<bool> = RefCell::new(false);
	pub static BEFORE_SESSION_END_CALLED: RefCell<bool> = RefCell::new(false);
}

pub struct TestSessionHandler;
impl SessionHandler<AccountId> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
	fn on_genesis_session<T: OpaqueKeys>(_validators: &[(AccountId, T)]) {}
	fn on_new_session<T: OpaqueKeys>(
		changed: bool,
		validators: &[(AccountId, T)],
		_queued_validators: &[(AccountId, T)],
	) {
		SESSION_CHANGED.with(|l| *l.borrow_mut() = changed);
		AUTHORITIES.with(|l| {
			*l.borrow_mut() = validators
				.iter()
				.map(|(_, id)| id.get::<AccountId>(DUMMY).unwrap_or_default())
				.collect()
		});
	}
	fn on_disabled(_validator_index: u32) {
		DISABLED.with(|l| *l.borrow_mut() = true)
	}
	fn on_before_session_ending() {
		BEFORE_SESSION_END_CALLED.with(|b| *b.borrow_mut() = true);
	}
}

pub struct TestShouldEndSession;
impl ShouldEndSession<u64> for TestShouldEndSession {
	fn should_end_session(now: u64) -> bool {
		let l = SESSION_LENGTH.with(|l| *l.borrow()) as u64;
		now % l == 0
			|| FORCE_SESSION_END.with(|l| {
				let r = *l.borrow();
				*l.borrow_mut() = false;
				r
			})
	}
}

parameter_types! {
	pub const Period: u32 = 10;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_validator_set::ValidatorOf<Self>;
	type ShouldEndSession = TestShouldEndSession;
	type NextSessionRotation = ();
	type SessionManager = ValidatorSet;
	type SessionHandler = TestSessionHandler;
	type Keys = MockSessionKeys;
	type WeightInfo = ();
}

parameter_types! {
	pub DefaultOwner: H160 = H160::from_str("0xa58482131a8d67725e996af72D91A849AcC0F4A1").expect("invalid address");
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	PrecompileAt<AddressU64<1>, ValidatorControllerPrecompile<R, DefaultOwner>>,
>;

pub type PCall = ValidatorControllerPrecompileCall<Runtime, DefaultOwner>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		ValidatorSet: pallet_validator_set,
		Session: pallet_session,
	}
);

pub(crate) struct ExtBuilder {
	validators: Vec<AccountId>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { validators: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn with_validators(mut self, validators: Vec<AccountId>) -> Self {
		self.validators = validators;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_validator_set::GenesisConfig::<Runtime> {
			initial_validators: self.validators.clone(),
			max_epochs_missed: U256::max_value(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
