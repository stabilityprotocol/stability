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

use core::str::FromStr;

use frame_support::parameter_types;
use pallet_supported_tokens_manager::SupportedTokensManager as SupportedTokensManagerT;
use precompile_utils::prelude::*;
use precompile_utils::{
	prelude::{log1, Address},
	testing::{Precompile1, PrecompileTesterExt},
};
use sp_core::{H160, H256};

use crate::{
	mock::{
		DefaultOwner, ExtBuilder, InitialDefaultTokenFee, PCall, Precompiles, PrecompilesValue,
		Runtime, SupportedTokensManager,
	},
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
			.execute_returns(Into::<H256>::into(DefaultOwner::get()));
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
fn fail_add_token_if_not_owner() {
	let new_owner = UnpermissionedAccount::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				new_owner,
				Precompile1,
				PCall::add_token {
					token: solidity::codec::Address(new_owner),
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
					token: solidity::codec::Address(new_owner),
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
					token: solidity::codec::Address(MeaninglessTokenAddress::get()),
					slot: H256::from_low_u64_be(0),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_token {
					token: solidity::codec::Address(MeaninglessTokenAddress::get()),
					slot: H256::from_low_u64_be(0),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"SupportedTokensManager: Token is already supported")
			})
	});
}

#[test]
fn fail_add_token_if_address_zero() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_token {
					token: H160::zero().into(),
					slot: H256::from_low_u64_be(0),
				},
			)
			.execute_reverts(|x| x == b"SupportedTokensManager: Invalid address");
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
					token: solidity::codec::Address(MeaninglessTokenAddress::get()),
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
fn zero_address_should_not_be_included_never() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::is_token_supported {
					token: sp_core::H160::zero().into(),
				},
			)
			.execute_returns(false);
	})
}

#[test]
fn add_token_and_remove_after() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_token {
					token: solidity::codec::Address(MeaninglessTokenAddress::get()),
					slot: H256::from_low_u64_be(0),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::is_token_supported {
					token: solidity::codec::Address(MeaninglessTokenAddress::get()),
				},
			)
			.execute_returns(true);

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::supported_tokens {})
			.execute_returns(vec![
				Address(InitialDefaultTokenFee::get()),
				Address(MeaninglessTokenAddress::get()),
			]);

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::remove_token {
					token: solidity::codec::Address(MeaninglessTokenAddress::get()),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::is_token_supported {
					token: solidity::codec::Address(MeaninglessTokenAddress::get()),
				},
			)
			.execute_returns(false);
	});
}

#[test]
fn update_default_controller() {
	ExtBuilder::default().build().execute_with(|| {
		<SupportedTokensManager as SupportedTokensManagerT>::add_supported_token(
			MeaninglessTokenAddress::get(),
			Default::default(),
		)
		.unwrap();
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::update_default_token {
					token: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_returns(());
	})
}

#[test]
fn fail_update_default_token() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				UnpermissionedAccount::get(),
				Precompile1,
				PCall::update_default_token {
					token: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"SupportedTokensManager: Caller is not the owner")
			});
	})
}

#[test]
fn fail_update_default_token_to_zero() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::update_default_token {
					token: H160::zero().into(),
				},
			)
			.execute_reverts(|x| x == b"SupportedTokensManager: Invalid address");
	})
}

#[test]
fn fail_update_default_token_is_not_supported() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::update_default_token {
					token: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"SupportedTokensManager: Target token is not supported")
			});
	})
}
