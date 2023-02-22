use core::str::FromStr;
use frame_support::parameter_types;
use precompile_utils::{
	prelude::{log3, Address},
	testing::{CryptoAlith, Precompile1, PrecompileTesterExt},
	EvmDataWriter,
};
use sp_core::{H160, U256};

use crate::{
	mock::{ExtBuilder, PCall, Precompiles, PrecompilesValue, Runtime},
	SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED, SELECTOR_LOG_VALIDATOR_TOKEN_RATE_CHANGED,
};
use crate::{DefaultAcceptance, DefaultConversionRate};

// No test of invalid selectors since we have a fallback behavior (deposit).
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

parameter_types! {
	pub MeaninglessTokenAddress:H160 = H160::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").expect("invalid address");
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
fn fail_to_update_conversion_rate_for_default_token() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_token_conversion_rate {
					token_address: Address(crate::mock::MockDefaultFeeToken::get()),
					numerator: U256::from(1),
					denominator: U256::from(1),
				},
			)
			.execute_reverts(|e| {
				e.eq_ignore_ascii_case(
					b"ValidatorFeeTokenController: default token conversion rate cannot be updated",
				)
			})
	});
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
