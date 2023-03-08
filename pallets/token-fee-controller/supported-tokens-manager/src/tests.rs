use crate::mock::{
	ExtBuilder, MeaninglessTokenAddress, MockDefaultFeeToken, SupportedTokensManager,
};

use super::*;

use frame_support::parameter_types;
use sp_core::H160;

parameter_types! {
	pub MeaninglessAccount: H160 = H160::from_low_u64_le(1);
}

#[test]
fn only_supported_default_token() {
	ExtBuilder::default().build().execute_with(|| {
		let default_token = SupportedTokensManager::default_token().unwrap();

		let supported_tokens =
			<SupportedTokensManager as crate::SupportedTokensManager>::get_supported_tokens();
		assert_eq!(supported_tokens.contains(&default_token), true);
		assert_eq!(supported_tokens.len(), 1);

		assert_eq!(
			<SupportedTokensManager as crate::SupportedTokensManager>::is_supported_token(
				default_token
			),
			true
		);
	});
}

#[test]
fn fail_to_remove_default() {
	ExtBuilder::default().build().execute_with(|| {
		let default_token = SupportedTokensManager::default_token().unwrap();

		assert!(
			<SupportedTokensManager as crate::SupportedTokensManager>::remove_supported_token(
				default_token
			)
			.is_err()
		);
	});
}

#[test]
fn add_token_set_as_default_and_remove_former() {
	ExtBuilder::default().build().execute_with(|| {
		<SupportedTokensManager as crate::SupportedTokensManager>::add_supported_token(
			MeaninglessTokenAddress::get(),
			H256::from_low_u64_be(2),
		)
		.expect("add token failed");

		let supported_tokens =
			<SupportedTokensManager as crate::SupportedTokensManager>::get_supported_tokens();

		assert!(supported_tokens.contains(&MeaninglessTokenAddress::get()));
		assert!(
			<SupportedTokensManager as crate::SupportedTokensManager>::is_supported_token(
				MeaninglessTokenAddress::get()
			)
		);

		assert!(
			<SupportedTokensManager as crate::SupportedTokensManager>::set_default_token(
				MeaninglessTokenAddress::get()
			)
			.is_ok()
		);

		assert_eq!(
			<SupportedTokensManager as crate::SupportedTokensManager>::get_default_token(),
			MeaninglessTokenAddress::get()
		);

		<SupportedTokensManager as crate::SupportedTokensManager>::remove_supported_token(
			MockDefaultFeeToken::get(),
		)
		.expect("update token failed");

		let supported_tokens =
			<SupportedTokensManager as crate::SupportedTokensManager>::get_supported_tokens();

		assert!(supported_tokens.contains(&MeaninglessTokenAddress::get()));
		assert_eq!(supported_tokens.len(), 1);

		assert!(
			<SupportedTokensManager as crate::SupportedTokensManager>::is_supported_token(
				MeaninglessTokenAddress::get()
			)
		);

		assert!(
			!<SupportedTokensManager as crate::SupportedTokensManager>::is_supported_token(
				MockDefaultFeeToken::get()
			)
		);
	});
}

#[test]
fn fail_to_set_not_supported_token_as_default() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(
			<SupportedTokensManager as crate::SupportedTokensManager>::set_default_token(
				MeaninglessTokenAddress::get()
			)
			.is_err()
		);
	});
}

#[test]
fn fail_to_add_already_supported_token() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(
			<SupportedTokensManager as crate::SupportedTokensManager>::add_supported_token(
				MockDefaultFeeToken::get(),
				H256::from_low_u64_be(2),
			)
			.is_err()
		);
	});
}

#[test]
fn fail_to_remove_not_present_token() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(
			<SupportedTokensManager as crate::SupportedTokensManager>::remove_supported_token(
				MeaninglessTokenAddress::get()
			)
			.is_err()
		);
	});
}
