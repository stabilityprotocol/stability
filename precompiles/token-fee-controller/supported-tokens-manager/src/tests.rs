use core::str::FromStr;

use frame_support::parameter_types;
use precompile_utils::{
	prelude::log1,
	testing::{Precompile1, PrecompileTesterExt},
	EvmDataWriter,
};
use sp_core::{H160, H256};

use crate::{
	mock::{DefaultOwner, ExtBuilder, PCall, Precompiles, PrecompilesValue, Runtime},
	SELECTOR_LOG_NEW_OWNER,
};

// No test of invalid selectors since we have a fallback behavior (deposit).
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
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
	pub MeaninglessTokenAddress: H160 = H160::from_str("0x3000000000000000000000000000000000000000").expect("invalid address");
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
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"SupportedTokensManager: Sender is not owner")
			});
	})
}

#[test]
fn fail_claim_ownership_if_not_claimable() {
	let new_owner = UnpermissionedAccount::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::claim_ownership {})
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"SupportedTokensManager: Target owner is not the claimer")
			})
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
fn fail_add_token_if_not_owner() {
	let new_owner = UnpermissionedAccount::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				new_owner,
				Precompile1,
				PCall::add_token {
					token: precompile_utils::data::Address(new_owner),
					slot: H256::from_low_u64_be(0),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"SupportedTokensManager: Caller is not the owner")
			})
	});
}

#[test]
fn fail_remove_token_if_not_owner() {
	let new_owner = UnpermissionedAccount::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				new_owner,
				Precompile1,
				PCall::remove_token {
					token: precompile_utils::data::Address(new_owner),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"SupportedTokensManager: Caller is not the owner")
			})
	});
}

#[test]
fn fail_add_token_if_already_added() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_token {
					token: precompile_utils::data::Address(MeaninglessTokenAddress::get()),
					slot: H256::from_low_u64_be(0),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_token {
					token: precompile_utils::data::Address(MeaninglessTokenAddress::get()),
					slot: H256::from_low_u64_be(0),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"SupportedTokensManager: Token is already supported")
			})
	});
}

#[test]
fn fail_remove_token_if_not_added() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::remove_token {
					token: precompile_utils::data::Address(MeaninglessTokenAddress::get()),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(
					b"SupportedTokensManager: Token not found in supported tokens",
				)
			})
	});
}

#[test]
fn add_token_and_remove_after() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_token {
					token: precompile_utils::data::Address(MeaninglessTokenAddress::get()),
					slot: H256::from_low_u64_be(0),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::is_token_supported {
					token: precompile_utils::data::Address(MeaninglessTokenAddress::get()),
				},
			)
			.execute_returns_encoded(true);

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::remove_token {
					token: precompile_utils::data::Address(MeaninglessTokenAddress::get()),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::is_token_supported {
					token: precompile_utils::data::Address(MeaninglessTokenAddress::get()),
				},
			)
			.execute_returns_encoded(false);
	});
}
