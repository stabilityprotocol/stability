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
