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

//! Mock helpers for Validator Set pallet.

#![cfg(test)]

use super::*;
use frame_support::{
	parameter_types,
	traits::{FindAuthor, StorageInstance},
};
use frame_system::EnsureRoot;
use pallet_session::*;
use sp_core::{crypto::key_types::DUMMY, H256};
use sp_runtime::BuildStorage;
use sp_runtime::{
	impl_opaque_keys,
	testing::UintAuthorityId,
	traits::{BlakeTwo256, IdentityLookup, OpaqueKeys},
	KeyTypeId, RuntimeAppPublic,
};
use sp_state_machine::BasicExternalities;
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

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		ValidatorSet: pallet_validator_set,
		ValidatorKeysController: crate,
		Session: pallet_session,
	}
);

pub const SESSION_BLOCK_LENGTH: u64 = 6;

thread_local! {
	pub static VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
	pub static NEXT_VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
	pub static AUTHORITIES: RefCell<Vec<UintAuthorityId>> =
		RefCell::new(vec![UintAuthorityId(1), UintAuthorityId(2), UintAuthorityId(3)]);
	pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
	pub static SESSION_LENGTH: u64 = SESSION_BLOCK_LENGTH;
	pub static SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
	pub static DISABLED: RefCell<bool> = RefCell::new(false);
	pub static BEFORE_SESSION_END_CALLED: RefCell<bool> = RefCell::new(false);
}

pub struct TestSessionHandler;
impl SessionHandler<u64> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
	fn on_genesis_session<T: OpaqueKeys>(_validators: &[(u64, T)]) {}
	fn on_new_session<T: OpaqueKeys>(
		changed: bool,
		validators: &[(u64, T)],
		_queued_validators: &[(u64, T)],
	) {
		SESSION_CHANGED.with(|l| *l.borrow_mut() = changed);
		AUTHORITIES.with(|l| {
			*l.borrow_mut() = validators
				.iter()
				.map(|(_, id)| id.get::<UintAuthorityId>(DUMMY).unwrap_or_default())
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
		let l = SESSION_LENGTH.with(|l| *l);
		now % l == 0
			|| FORCE_SESSION_END.with(|l| {
				let r = *l.borrow();
				*l.borrow_mut() = false;
				r
			})
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();
	let keys: Vec<_> = NEXT_VALIDATORS.with(|l| {
		l.borrow()
			.iter()
			.cloned()
			.map(|i| {
				(
					i,
					i,
					MockSessionKeys {
						dummy: UintAuthorityId(i),
					},
				)
			})
			.collect()
	});
	BasicExternalities::execute_with_storage(&mut t, || {
		for (ref k, ..) in &keys {
			frame_system::Pallet::<Test>::inc_providers(k);
		}
		frame_system::Pallet::<Test>::inc_providers(&4);
		frame_system::Pallet::<Test>::inc_providers(&69);
	});
	pallet_validator_set::GenesisConfig::<Test> {
		initial_validators: keys.iter().map(|x| x.1).collect::<Vec<_>>(),
		max_epochs_missed: 1.into(),
	}
	.assimilate_storage(&mut t)
	.unwrap();
	pallet_session::GenesisConfig::<Test> { keys: keys.clone() }
		.assimilate_storage(&mut t)
		.unwrap();
	sp_io::TestExternalities::new(t)
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
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
	pub const MinAuthorities: u32 = 0;
}

pub struct PeriodicSessionBlockManager;
impl pallet_validator_set::SessionBlockManager<u64> for PeriodicSessionBlockManager {
	fn session_start_block(session_index: sp_staking::SessionIndex) -> u64 {
		return (session_index as u64) * SESSION_BLOCK_LENGTH;
	}
}

pub struct Prefix;
impl StorageInstance for Prefix {
	fn pallet_prefix() -> &'static str {
		"test"
	}
	const STORAGE_PREFIX: &'static str = "test";
}

pub type NextBlockValidator = StorageValue<Prefix, u64, OptionQuery>;

pub type ConsensusEngineId = [u8; 4];

pub struct FindBlockAuthorityId;
impl FindAuthor<u64> for FindBlockAuthorityId {
	fn find_author<'a, I>(_digests: I) -> Option<u64>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		return NextBlockValidator::get();
	}
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}
pub struct AccountIdOfValidator;
impl Convert<UintAuthorityId, u64> for AccountIdOfValidator {
	fn convert(authority_id: UintAuthorityId) -> u64 {
		authority_id.0.clone()
	}
}
parameter_types! {
	pub const MaxKeys: u32 = 100;
}
impl pallet_validator_set::Config for Test {
	type AddRemoveOrigin = EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type MinAuthorities = MinAuthorities;

	type SessionBlockManager = PeriodicSessionBlockManager;

	type FindAuthor = FindBlockAuthorityId;

	type AuthorityId = UintAuthorityId;

	type MaxKeys = MaxKeys;

	type AccountIdOfValidator = AccountIdOfValidator;
}

impl pallet_session::Config for Test {
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

pub struct SessionKeysBuilder;
impl crate::SessionKeysBuilder<UintAuthorityId, UintAuthorityId, MockSessionKeys>
	for SessionKeysBuilder
{
	fn new(aura: UintAuthorityId, _grandpa: UintAuthorityId) -> MockSessionKeys {
		MockSessionKeys {
			dummy: aura.clone(),
		}
	}
}
pub struct ValidatorIdMapping;
impl Convert<UintAuthorityId, u64> for ValidatorIdMapping {
	fn convert(a: UintAuthorityId) -> u64 {
		return a.0;
	}
}
impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;

	type FinalizationId = UintAuthorityId;

	type SessionKeysBuilder = SessionKeysBuilder;

	type ValidatorIdOfValidation = ValidatorIdMapping;
}
