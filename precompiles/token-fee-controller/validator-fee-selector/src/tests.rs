use precompile_utils::{
	prelude::{log3, Address},
	testing::{CryptoAlith, Precompile1, PrecompileTesterExt},
	EvmDataWriter,
};
use sp_core::{H160, U256};

use crate::{
	mock::{ExtBuilder, NonCryptoAlith, PCall, Precompiles, PrecompilesValue, Runtime},
	SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED, SELECTOR_LOG_VALIDATOR_TOKEN_RATE_CHANGED,
};
use crate::{DefaultAcceptance, DefaultConversionRate};

// No test of invalid selectors since we have a fallback behavior (deposit).
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

use crate::mock::MeaninglessTokenAddress;

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
			.execute_returns_encoded(DefaultAcceptance::get());

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
			.execute_returns_encoded(!DefaultAcceptance::get());
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
			.execute_returns_encoded(true);

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
			.execute_returns_encoded(false);
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
				EvmDataWriter::new().write(true).build(),
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
			.execute_returns_encoded(true);

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
				EvmDataWriter::new().write(false).build(),
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
			.execute_returns_encoded(false);
	});
}

// conversion rate management

#[test]
fn default_conversion_rate() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::token_conversion_rate {
					validator: Address(CryptoAlith.into()),
					token_address: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_returns(
				EvmDataWriter::new()
					.write(DefaultConversionRate::get().0)
					.write(DefaultConversionRate::get().1)
					.build(),
			);
	})
}

#[test]
fn update_conversion_rate() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_token_conversion_rate {
					token_address: MeaninglessTokenAddress::get().into(),
					numerator: U256::from(100),
					denominator: U256::from(3),
				},
			)
			.expect_log(log3(
				Precompile1,
				SELECTOR_LOG_VALIDATOR_TOKEN_RATE_CHANGED,
				H160::from(CryptoAlith),
				MeaninglessTokenAddress::get(),
				EvmDataWriter::new().write(100u128).write(3u128).build(),
			))
			.execute_some();

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::token_conversion_rate {
					validator: Address(CryptoAlith.into()),
					token_address: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_returns(EvmDataWriter::new().write(100u128).write(3u128).build());
	})
}

#[test]
fn fail_update_conversion_rate_non_validator() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				NonCryptoAlith::get(),
				Precompile1,
				PCall::set_token_conversion_rate {
					token_address: MeaninglessTokenAddress::get().into(),
					numerator: U256::from(100),
					denominator: U256::from(3),
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

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::token_conversion_rate {
					validator: Address(CryptoAlith.into()),
					token_address: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_returns(
				EvmDataWriter::new()
					.write(DefaultConversionRate::get().0)
					.write(DefaultConversionRate::get().1)
					.build(),
			);
	})
}
