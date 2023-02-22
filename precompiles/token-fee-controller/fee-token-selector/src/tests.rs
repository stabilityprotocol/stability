use core::str::FromStr;
use frame_support::parameter_types;
use precompile_utils::{
	prelude::Address,
	testing::{CryptoAlith, Precompile1, PrecompileTesterExt},
};
use sp_core::{H160, H256};

use crate::mock::{
	ExtBuilder, MockDefaultFeeToken, PCall, Precompiles, PrecompilesValue, Runtime,
};

// No test of invalid selectors since we have a fallback behavior (deposit).
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

parameter_types! {
	pub MeaninglessTokenAddress:H160 = H160::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").expect("invalid address");
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
			.execute_returns_encoded(Into::<H256>::into(MockDefaultFeeToken::get()));
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

		precompiles()
			.prepare_test(
				CryptoAlith,
				Precompile1,
				PCall::get_fee_token {
					address: Address(CryptoAlith.into()),
				},
			)
			.execute_returns_encoded(Into::<H256>::into(MeaninglessTokenAddress::get()));
	});
}
