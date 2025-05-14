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

//! Tests for the Validator Set pallet.

#![cfg(test)]

use super::*;
use crate::mock::{
	authorities, ExtBuilder, NextBlockValidator, RuntimeOrigin, Session, Test, ValidatorSet,
	SESSION_BLOCK_LENGTH,
};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};
use frame_system::RawOrigin;
use sp_application_crypto::RuntimeAppPublic;
use sp_core::U256;
use sp_runtime::testing::UintAuthorityId;

#[test]
fn simple_setup_should_work() {
	ExtBuilder::build().execute_with(|| {
		assert_eq!(
			authorities(),
			vec![UintAuthorityId(1), UintAuthorityId(2), UintAuthorityId(3)]
		);
		assert_eq!(ValidatorSet::validators(), vec![1u64, 2u64, 3u64]);
		assert_eq!(Session::validators(), vec![1, 2, 3]);
	});
}

#[test]
fn add_validator_updates_approved_validators_list() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(ValidatorSet::add_validator(RuntimeOrigin::root(), 4));
		assert_eq!(
			ValidatorSet::approved_validators(),
			vec![1u64, 2u64, 3u64, 4u64]
		)
	});
}

#[test]
fn remove_validator_updates_validators_list() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(ValidatorSet::remove_validator(RuntimeOrigin::root(), 2));
		assert_eq!(ValidatorSet::validators(), vec![1u64, 3u64]);
		assert_eq!(ValidatorSet::approved_validators(), vec![1u64, 3u64]);
	});
}

#[test]
fn add_validator_fails_with_invalid_origin() {
	ExtBuilder::build().execute_with(|| {
		assert_noop!(
			ValidatorSet::add_validator(RuntimeOrigin::signed(1), 4),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn remove_validator_fails_with_invalid_origin() {
	ExtBuilder::build().execute_with(|| {
		assert_noop!(
			ValidatorSet::remove_validator(RuntimeOrigin::signed(1), 4),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn duplicate_check() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(ValidatorSet::add_validator(RuntimeOrigin::root(), 4));
		assert_eq!(
			ValidatorSet::approved_validators(),
			vec![1u64, 2u64, 3u64, 4u64]
		);
		assert_noop!(
			ValidatorSet::add_validator(RuntimeOrigin::root(), 4),
			Error::<Test>::Duplicate
		);
	});
}

// im-online tests

#[test]
fn validator_goes_off_and_reconnects() {
	ExtBuilder::build().execute_with(|| {
		for i in 0..SESSION_BLOCK_LENGTH {
			mock_mine_block(1, i);
		}

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(0);

		for i in 0..SESSION_BLOCK_LENGTH {
			mock_mine_block(1, i + SESSION_BLOCK_LENGTH);
		}

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(1);

		let new_validators = <pallet::Validators<Test>>::get();

		assert!(new_validators.contains(&authorities()[1].0) == false);

		let heartbeat = Heartbeat {
			block_number: 7,
			session_index: 1,
			authority_id: UintAuthorityId(2),
			authority_index: 1,
		};

		let signature = UintAuthorityId(2).sign(&heartbeat.encode()).unwrap();

		pallet::Pallet::<Test>::add_validator_again(RawOrigin::None.into(), heartbeat, signature)
			.expect("not to fail");

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(2);

		let new_validators = <pallet::Validators<Test>>::get();

		assert!(new_validators.contains(&authorities()[1].0));
	});
}

#[test]
fn validator_misses_one_and_reconnects() {
	ExtBuilder::build().execute_with(|| {
		for i in 0..SESSION_BLOCK_LENGTH {
			mock_mine_block(1, i);
		}

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(0);

		for i in 0..SESSION_BLOCK_LENGTH {
			mock_mine_block(2, i + SESSION_BLOCK_LENGTH);
		}

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(1);

		let new_validators = <pallet::Validators<Test>>::get();

		assert!(new_validators.contains(&authorities()[1].0));
		// Safety check for ensuring that the validator id and index are correct
		assert!(&authorities()[1].0.eq(&2u64));
		assert!(EpochsMissed::<Test>::get(2) == U256::zero());
	});
}

#[test]
fn validator_tries_to_reconnect_with_mismatch_parameters() {
	ExtBuilder::build().execute_with(|| {
		for i in 0..SESSION_BLOCK_LENGTH {
			mock_mine_block(1, i);
		}

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(0);

		for i in 0..SESSION_BLOCK_LENGTH {
			mock_mine_block(1, i + SESSION_BLOCK_LENGTH);
		}

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(1);

		let new_validators = <pallet::Validators<Test>>::get();

		assert!(new_validators.contains(&authorities()[1].0) == false);

		let heartbeat = Heartbeat {
			block_number: 7,
			session_index: 1,
			authority_id: UintAuthorityId(2),
			authority_index: 0,
		};

		let signature = UintAuthorityId(2).sign(&heartbeat.encode()).unwrap();

		assert!(pallet::Pallet::<Test>::add_validator_again(
			RawOrigin::None.into(),
			heartbeat,
			signature
		)
		.is_err());
	});
}

#[test]
fn non_approved_validator_tries_to_connect() {
	ExtBuilder::build().execute_with(|| {
		for i in 0..SESSION_BLOCK_LENGTH {
			mock_mine_block(1, i);
		}

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(0);

		for i in 0..SESSION_BLOCK_LENGTH {
			mock_mine_block(1, i + SESSION_BLOCK_LENGTH);
		}

		<pallet::Pallet<Test> as pallet_session::SessionManager<u64>>::end_session(1);

		let new_validators = <pallet::Validators<Test>>::get();

		assert!(new_validators.contains(&authorities()[1].0) == false);

		let heartbeat = Heartbeat {
			block_number: 7,
			session_index: 1,
			authority_id: UintAuthorityId(800),
			authority_index: 0,
		};

		let signature = UintAuthorityId(800).sign(&heartbeat.encode()).unwrap();

		assert!(pallet::Pallet::<Test>::add_validator_again(
			RawOrigin::None.into(),
			heartbeat,
			signature
		)
		.is_err());
	});
}

// tools

fn mock_mine_block(validator: u64, block_number: u64) {
	NextBlockValidator::set(Some(validator));
	pallet::Pallet::<Test>::on_finalize(block_number);
}
