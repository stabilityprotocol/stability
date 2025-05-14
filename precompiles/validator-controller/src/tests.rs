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

use crate::{mock::*, *};

use precompile_utils::testing::*;
use sha3::{Digest, Keccak256};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn account_id_to_evm_address(account_id: AccountId) -> Address {
	H160(account_id.0).into()
}

#[test]
fn selectors() {
	assert!(PCall::owner_selectors().contains(&0x8da5cb5b));
	assert!(PCall::pending_owner_selectors().contains(&0xe30c3978));
	assert!(PCall::transfer_ownership_selectors().contains(&0xf2fde38b));
	assert!(PCall::claim_ownership_selectors().contains(&0x79ba5097));
	assert!(PCall::add_validator_selectors().contains(&0x4d238c8e));
	assert!(PCall::remove_validator_selectors().contains(&0x40a141ff));
	assert!(PCall::get_validator_list_selectors().contains(&0xe35c0f7d));
	assert!(PCall::get_validator_missing_blocks_selectors().contains(&0x41ee9a53));
	assert_eq!(
		crate::SELECTOR_LOG_NEW_OWNER,
		&Keccak256::digest(b"NewOwner(address)")[..]
	);
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), CryptoAlith, Precompile1);

		tester.test_view_modifier(PCall::owner_selectors());
		tester.test_view_modifier(PCall::pending_owner_selectors());
		tester.test_view_modifier(PCall::get_validator_missing_blocks_selectors());
		tester.test_default_modifier(PCall::transfer_ownership_selectors());
		tester.test_default_modifier(PCall::claim_ownership_selectors());
		tester.test_default_modifier(PCall::add_validator_selectors());
		tester.test_default_modifier(PCall::remove_validator_selectors());
	});
}

#[test]
fn owner_correctly_init() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::owner {})
			.execute_returns(Into::<H256>::into(DefaultOwner::get()));
	})
}

parameter_types! {
	pub UnpermissionedAccount:H160 = H160::from_str("0x1000000000000000000000000000000000000000").expect("invalid address");
	pub UnpermissionedAccount2:H160 = H160::from_str("0x2000000000000000000000000000000000000000").expect("invalid address");
}

#[test]

fn transfer_ownership_set_target_if_owner_twice() {
	ExtBuilder::default().build().execute_with(|| {
		let new_owner = UnpermissionedAccount::get();
		let other_owner = UnpermissionedAccount2::get();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::transfer_ownership {
					new_owner: solidity::codec::Address(new_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::pending_owner {})
			.execute_returns(Into::<H256>::into(new_owner));

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::transfer_ownership {
					new_owner: solidity::codec::Address(other_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::pending_owner {})
			.execute_returns(Into::<H256>::into(other_owner));
	})
}

#[test]
fn fail_transfer_ownership_if_not_owner() {
	ExtBuilder::default().build().execute_with(|| {
		let new_owner = UnpermissionedAccount::get();

		precompiles()
			.prepare_test(
				new_owner,
				Precompile1,
				PCall::transfer_ownership {
					new_owner: solidity::codec::Address(new_owner),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	})
}

#[test]
fn fail_claim_ownership_if_not_claimable() {
	let new_owner = UnpermissionedAccount::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::claim_ownership {})
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"target owner is not the claimer"))
	});
}

#[test]
fn claim_ownership_if_claimable() {
	let owner = DefaultOwner::get();
	let new_owner = UnpermissionedAccount::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				owner,
				Precompile1,
				PCall::transfer_ownership {
					new_owner: solidity::codec::Address(new_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::claim_ownership {})
			.expect_log(log1(
				Precompile1,
				SELECTOR_LOG_NEW_OWNER,
				solidity::encode_event_data(Into::<H256>::into(new_owner)),
			))
			.execute_some();

		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::owner {})
			.execute_returns(Into::<H256>::into(new_owner));
	});
}

#[test]
fn add_validator() {
	let owner = DefaultOwner::get();
	let validator = ValidatorCandidate::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				owner,
				Precompile1,
				PCall::add_validator {
					new_validator: account_id_to_evm_address(validator.clone()),
				},
			)
			.expect_log(log1(
				Precompile1,
				SELECTOR_VALIDATOR_ADDED,
				solidity::encode_event_data(account_id_to_evm_address(validator.clone())),
			))
			.execute_some();

		assert_eq!(ValidatorSet::approved_validators(), vec![validator]);
	});
}

parameter_types! {
	pub ValidatorCandidate:AccountId = AccountId::from_str("0x42a4ACa0201918116E0C5569d93faaD0E435aB46").expect("invalid address");
}

#[test]
fn add_validator_fails_if_sender_not_owner() {
	let sender = UnpermissionedAccount::get();
	let validator = ValidatorCandidate::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				sender,
				Precompile1,
				PCall::add_validator {
					new_validator: account_id_to_evm_address(validator.clone()),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	});
}

parameter_types! {
	pub ValidatorInitial:AccountId = AccountId::from_str("0x42a4ACa0201918116E0C5569d93faaD0E435aB45").expect("invalid address");
}

#[test]
fn add_validator_if_already_init() {
	let owner = DefaultOwner::get();
	let initial_validator = ValidatorInitial::get();
	let validator = ValidatorCandidate::get();
	ExtBuilder::default()
		.with_validators(vec![initial_validator.clone()])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					owner,
					Precompile1,
					PCall::add_validator {
						new_validator: account_id_to_evm_address(validator.clone()),
					},
				)
				.expect_log(log1(
					Precompile1,
					SELECTOR_VALIDATOR_ADDED,
					solidity::encode_event_data(account_id_to_evm_address(validator.clone())),
				))
				.execute_some();

			assert_eq!(
				ValidatorSet::approved_validators(),
				vec![initial_validator, validator]
			);
		})
}

parameter_types! {
	pub SecondValidatorCandidate:AccountId = AccountId::from_str("0x42a4ACa0201918116E0C5569d93faaD0E435aB47").expect("invalid address");
}
#[test]
fn add_two_validators() {
	let owner = DefaultOwner::get();
	let validator = ValidatorCandidate::get();
	let second_validator = SecondValidatorCandidate::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				owner,
				Precompile1,
				PCall::add_validator {
					new_validator: account_id_to_evm_address(validator.clone()),
				},
			)
			.execute_some();
		precompiles()
			.prepare_test(
				owner,
				Precompile1,
				PCall::add_validator {
					new_validator: account_id_to_evm_address(second_validator.clone()),
				},
			)
			.execute_some();

		assert_eq!(
			ValidatorSet::approved_validators(),
			vec![validator, second_validator]
		);
	});
}

#[test]
fn add_validator_fails_if_add_already_validator() {
	let owner = DefaultOwner::get();
	let validator = ValidatorInitial::get();
	ExtBuilder::default()
		.with_validators(vec![validator.clone()])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					owner,
					Precompile1,
					PCall::add_validator {
						new_validator: account_id_to_evm_address(validator.clone()),
					},
				)
				.execute_reverts(|_| true);
		});
}

#[test]
fn remove_validator() {
	let owner = DefaultOwner::get();
	let validator = ValidatorInitial::get();
	ExtBuilder::default()
		.with_validators(vec![validator.clone()])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					owner,
					Precompile1,
					PCall::remove_validator {
						removed_validator: account_id_to_evm_address(validator.clone()),
					},
				)
				.expect_log(log1(
					Precompile1,
					SELECTOR_VALIDATOR_REMOVED,
					solidity::encode_event_data(account_id_to_evm_address(validator.clone())),
				))
				.execute_some();

			assert_eq!(ValidatorSet::validators(), vec![]);
		});
}

#[test]
fn remove_validator_fails_if_sender_not_owner() {
	let sender = UnpermissionedAccount::get();
	let validator = ValidatorInitial::get();
	ExtBuilder::default()
		.with_validators(vec![validator.clone()])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					sender,
					Precompile1,
					PCall::remove_validator {
						removed_validator: account_id_to_evm_address(validator.clone()),
					},
				)
				.execute_reverts(|_| true);
		});
}

#[test]
fn get_default_validator_list() {
	let sender = UnpermissionedAccount::get();
	let validator = ValidatorInitial::get();
	let validators = vec![validator.clone()];
	ExtBuilder::default()
		.with_validators(validators.clone())
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(sender, Precompile1, PCall::get_validator_list {})
				.execute_returns(
					validators
						.iter()
						.map(|v| account_id_to_evm_address(*v))
						.collect::<Vec<Address>>(),
				);
		});
}

#[test]
fn get_default_active_validator_list() {
	let sender = UnpermissionedAccount::get();
	let validator = ValidatorInitial::get();
	let validators = vec![validator.clone()];
	ExtBuilder::default()
		.with_validators(validators.clone())
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(sender, Precompile1, PCall::get_active_validator_list {})
				.execute_returns(
					validators
						.iter()
						.map(|v| account_id_to_evm_address(*v))
						.collect::<Vec<Address>>(),
				);
		});
}

#[test]
fn checks_if_validator_is_missing_blocks() {
	let sender = UnpermissionedAccount::get();
	let validator = ValidatorInitial::get();
	let validators = vec![validator.clone()];
	ExtBuilder::default()
		.with_validators(validators.clone())
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					sender,
					Precompile1,
					PCall::get_validator_missing_blocks {
						validator: account_id_to_evm_address(validator.clone()),
					},
				)
				.execute_returns(U256::zero());
		});
}

#[test]
fn checks_a_validator_missing_blocks() {
	let sender = UnpermissionedAccount::get();
	let validator = ValidatorInitial::get();
	let validators = vec![validator.clone()];
	ExtBuilder::default()
		.with_validators(validators.clone())
		.build()
		.execute_with(|| {
			pallet_validator_set::EpochsMissed::<Runtime>::set(validator.clone(), U256::from(2));
			precompiles()
				.prepare_test(
					sender,
					Precompile1,
					PCall::get_validator_missing_blocks {
						validator: account_id_to_evm_address(validator.clone()),
					},
				)
				.execute_returns(U256::from(2));
		});
}
