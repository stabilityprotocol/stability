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

use precompile_utils::prelude::*;
use precompile_utils::{
	prelude::{log1, log2, log3, Address},
	testing::{CryptoAlith, Precompile1, PrecompileTesterExt},
};
use sp_core::{H160, H256};

use crate::{
	mock::{
		DefaultOwner, ExtBuilder, InitialDefaultConversionRateController, MeaninglessTokenAddress,
		NonCryptoAlith, PCall, Precompiles, PrecompilesValue, Runtime, UnpermissionedAccount,
		UnpermissionedAccount2,
	},
	DefaultAcceptance, SELECTOR_LOG_NEW_OWNER, SELECTOR_LOG_VALIDATOR_CONTROLLER_CHANGED,
	SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED,
};

// No test of invalid selectors since we have a fallback behavior (deposit).
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

// Ownable

#[test]
fn owner_correctly_init() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::owner {})
			.execute_returns(Into::<H256>::into(DefaultOwner::get()));
	})
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
				x.eq_ignore_ascii_case(b"ValidatorFeeTokenController: Sender is not owner")
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
				x.eq_ignore_ascii_case(
					b"ValidatorFeeTokenController: Target owner is not the claimer",
				)
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
fn update_default_controller() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::update_default_controller {
					controller: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_some();
	})
}

#[test]
fn fail_update_default_controller() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				UnpermissionedAccount::get(),
				Precompile1,
				PCall::update_default_controller {
					controller: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(b"ValidatorFeeTokenController: sender is not the owner")
			});
	})
}

// fee token acceptance management

#[test]
fn non_default_token_address() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::validator_supports_token {
					validator: Address(CryptoAlith.into()),
					token_address: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_returns(DefaultAcceptance::get());

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_token_acceptance {
					token_address: MeaninglessTokenAddress::get().into(),
					acceptance_value: !DefaultAcceptance::get(),
				},
			)
			.execute_some();
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::validator_supports_token {
					validator: Address(CryptoAlith.into()),
					token_address: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_returns(!DefaultAcceptance::get());
	});
}

#[test]
fn fail_to_set_for_unsupported_token() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_token_acceptance {
					token_address: Address(CryptoAlith.into()),
					acceptance_value: true,
				},
			)
			.execute_reverts(|x| x == b"ValidatorFeeTokenController: token not supported");
	});
}

#[test]
fn default_token_address() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::validator_supports_token {
					validator: Address(CryptoAlith.into()),
					token_address: crate::mock::MockDefaultFeeToken::get().into(),
				},
			)
			.execute_returns(true);

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_token_acceptance {
					token_address: crate::mock::MockDefaultFeeToken::get().into(),
					acceptance_value: false,
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::validator_supports_token {
					validator: Address(CryptoAlith.into()),
					token_address: crate::mock::MockDefaultFeeToken::get().into(),
				},
			)
			.execute_returns(false);
	});
}

#[test]
fn fail_to_accept_token_if_not_validator() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				NonCryptoAlith::get(),
				Precompile1,
				PCall::set_token_acceptance {
					token_address: MeaninglessTokenAddress::get().into(),
					acceptance_value: true,
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(
					b"ValidatorFeeTokenController: sender is not an approved validator",
				)
			});
	});
}

#[test]
fn accept_token_and_revoke() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_token_acceptance {
					token_address: MeaninglessTokenAddress::get().into(),
					acceptance_value: true,
				},
			)
			.expect_log(log3(
				Precompile1,
				SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED,
				H160::from(CryptoAlith),
				MeaninglessTokenAddress::get(),
				solidity::encode_event_data(true),
			))
			.execute_some();

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::validator_supports_token {
					validator: Address(CryptoAlith.into()),
					token_address: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_returns(true);

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_token_acceptance {
					token_address: MeaninglessTokenAddress::get().into(),
					acceptance_value: false,
				},
			)
			.expect_log(log3(
				Precompile1,
				SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED,
				H160::from(CryptoAlith),
				MeaninglessTokenAddress::get(),
				solidity::encode_event_data(false),
			))
			.execute_some();

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::validator_supports_token {
					validator: Address(CryptoAlith.into()),
					token_address: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_returns(false);
	});
}

// conversion rate management

#[test]
fn default_conversion_rate() {
	let default: Address = InitialDefaultConversionRateController::get().into();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::conversion_rate_controller {
					validator: Address(CryptoAlith.into()),
				},
			)
			.execute_returns(default);
	})
}

#[test]
fn fail_update_conversion_rate_controller_of_eoa() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::update_conversion_rate_controller {
					cr_controller: Address(CryptoAlith.into()),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(
					b"ValidatorFeeTokenController: default token conversion rate cannot be updated",
				)
			});
	})
}

#[test]
fn update_conversion_rate_controller() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::update_conversion_rate_controller {
					cr_controller: MeaninglessTokenAddress::get().into(),
				},
			)
			.expect_log(log2(
				Precompile1,
				SELECTOR_LOG_VALIDATOR_CONTROLLER_CHANGED,
				H160::from(CryptoAlith),
				solidity::encode_event_data(Address(MeaninglessTokenAddress::get())),
			))
			.execute_some();

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::conversion_rate_controller {
					validator: Address(CryptoAlith.into()),
				},
			)
			.execute_returns(Address(MeaninglessTokenAddress::get()));
	})
}

#[test]
fn fail_update_conversion_rate_non_validator() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				NonCryptoAlith::get(),
				Precompile1,
				PCall::update_conversion_rate_controller {
					cr_controller: Address(CryptoAlith.into()),
				},
			)
			.execute_reverts(|x| {
				x.eq_ignore_ascii_case(
					b"ValidatorFeeTokenController: sender is not an approved validator",
				)
			});
	})
}

#[test]
fn reverts_if_validator_dont_accepts_token() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_token_acceptance {
					token_address: MeaninglessTokenAddress::get().into(),
					acceptance_value: true,
				},
			)
			.execute_some();
	})
}
