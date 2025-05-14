// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use crate::mock::{
	ExtBuilder, MeaninglessTokenAddress, MockDefaultFeeToken, MockSupportedTokensManager,
	UserFeeTokenSelector,
};

use super::*;

use frame_support::parameter_types;
use pallet_supported_tokens_manager::SupportedTokensManager;
use sp_core::H160;

parameter_types! {
	pub MeaninglessAccount: H160 = H160::from_low_u64_le(1);
}

#[test]
fn get_default_token() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			UserFeeTokenSelector::get_user_fee_token(MeaninglessAccount::get()),
			<MockSupportedTokensManager as pallet_supported_tokens_manager::SupportedTokensManager>::get_default_token()
		);
	});
}

#[test]
fn set_token() {
	ExtBuilder::default().build().execute_with(|| {
		UserFeeTokenSelector::set_user_fee_token(
			MeaninglessAccount::get(),
			MeaninglessTokenAddress::get(),
		)
		.unwrap();

		assert_eq!(
			UserFeeTokenSelector::get_user_fee_token(MeaninglessAccount::get()),
			MeaninglessTokenAddress::get()
		);
	});
}

#[test]
fn set_token_if_no_longer_available_is_default() {
	ExtBuilder::default().build().execute_with(|| {
		UserFeeTokenSelector::set_user_fee_token(
			MeaninglessAccount::get(),
			MeaninglessTokenAddress::get(),
		)
		.unwrap();

		assert_eq!(
			UserFeeTokenSelector::get_user_fee_token(MeaninglessAccount::get()),
			MeaninglessTokenAddress::get()
		);

		assert!(
			MockSupportedTokensManager::remove_supported_token(MeaninglessTokenAddress::get())
				.is_ok()
		);

		assert_eq!(
			UserFeeTokenSelector::get_user_fee_token(MeaninglessAccount::get()),
			MockDefaultFeeToken::get()
		);
	});
}
