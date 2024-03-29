use precompile_utils::{
	prelude::Address,
	testing::{CryptoAlith, Precompile1, PrecompileTesterExt},
};
use sp_core::{H160, H256};

use crate::mock::{
	ExtBuilder, MeaninglessTokenAddress, MockDefaultFeeToken, PCall, Precompiles, PrecompilesValue,
	Runtime,
};

// No test of invalid selectors since we have a fallback behavior (deposit).
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn default_token_address() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::get_fee_token {
					address: Address(CryptoAlith.into()),
				},
			)
			.execute_returns(Into::<H256>::into(MockDefaultFeeToken::get()));
	});
}

#[test]
fn fail_set_for_unsupported_token() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_fee_token {
					token_address: Address(CryptoAlith.into()),
				},
			)
			.execute_reverts(|x| x == b"UserFeeTokenController: token not supported");
	});
}

#[test]
fn fail_set_for_zero_address() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_fee_token {
					token_address: Address(H160::zero()),
				},
			)
			.execute_reverts(|x| x == b"UserFeeTokenController: zero address is invalid");
	});
}

#[test]
fn set_token() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::set_fee_token {
					token_address: MeaninglessTokenAddress::get().into(),
				},
			)
			.execute_some();
	});
}
