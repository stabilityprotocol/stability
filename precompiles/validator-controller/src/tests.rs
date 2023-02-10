use crate::{mock::*, *};

use codec::Encode;
use precompile_utils::testing::*;
use sha3::{Digest, Keccak256};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn account_id_to_h256(account_id: AccountId) -> H256 {
	let mut h256 = H256::default();
	h256.as_bytes_mut().copy_from_slice(&account_id.encode());
	h256
}

#[test]
fn selectors() {
	assert!(PCall::owner_selectors().contains(&0x8da5cb5b));
	assert!(PCall::pending_owner_selectors().contains(&0xe30c3978));
	assert!(PCall::transfer_ownership_selectors().contains(&0xf2fde38b));
	assert!(PCall::claim_ownership_selectors().contains(&0x79ba5097));
	assert!(PCall::add_validator_selectors().contains(&0x4eedc9b0));
	assert!(PCall::remove_validator_selectors().contains(&0xcd993ba5));

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
			.execute_returns_encoded(Into::<H256>::into(DefaultOwner::get()));
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
					new_owner: precompile_utils::data::Address(new_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::pending_owner {})
			.execute_returns_encoded(Into::<H256>::into(new_owner));

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::transfer_ownership {
					new_owner: precompile_utils::data::Address(other_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::pending_owner {})
			.execute_returns_encoded(Into::<H256>::into(other_owner));
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
					new_owner: precompile_utils::data::Address(new_owner),
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
					new_owner: precompile_utils::data::Address(new_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::claim_ownership {})
			.expect_log(log1(
				Precompile1,
				SELECTOR_LOG_NEW_OWNER,
				EvmDataWriter::new()
					.write(Into::<H256>::into(new_owner))
					.build(),
			))
			.execute_some();

		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::owner {})
			.execute_returns_encoded(Into::<H256>::into(new_owner));
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
					new_validator: account_id_to_h256(validator.clone()),
				},
			)
			.expect_log(log1(
				Precompile1,
				SELECTOR_VALIDATOR_ADDED,
				EvmDataWriter::new()
					.write(account_id_to_h256(validator.clone()))
					.build(),
			))
			.execute_some();

		assert_eq!(ValidatorSet::validators(), vec![validator]);
	});
}

parameter_types! {
	pub ValidatorCandidate:AccountId = AccountId::from_str("0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22").expect("invalid address");
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
					new_validator: account_id_to_h256(validator.clone()),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	});
}

parameter_types! {
	pub ValidatorInitial:AccountId = AccountId::from_str("0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe21").expect("invalid address");
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
						new_validator: account_id_to_h256(validator.clone()),
					},
				)
				.expect_log(log1(
					Precompile1,
					SELECTOR_VALIDATOR_ADDED,
					EvmDataWriter::new()
						.write(account_id_to_h256(validator.clone()))
						.build(),
				))
				.execute_some();

			assert_eq!(
				ValidatorSet::validators(),
				vec![initial_validator, validator]
			);
		})
}

parameter_types! {
	pub SecondValidatorCandidate:AccountId = AccountId::from_str("0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe23").expect("invalid address");
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
					new_validator: account_id_to_h256(validator.clone()),
				},
			)
			.execute_some();
		precompiles()
			.prepare_test(
				owner,
				Precompile1,
				PCall::add_validator {
					new_validator: account_id_to_h256(second_validator.clone()),
				},
			)
			.execute_some();

		assert_eq!(
			ValidatorSet::validators(),
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
						new_validator: account_id_to_h256(validator.clone()),
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
						removed_validator: account_id_to_h256(validator.clone()),
					},
				)
				.expect_log(log1(
					Precompile1,
					SELECTOR_VALIDATOR_REMOVED,
					EvmDataWriter::new()
						.write(account_id_to_h256(validator.clone()))
						.build(),
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
						removed_validator: account_id_to_h256(validator.clone()),
					},
				)
				.execute_reverts(|_| true);
		});
}
