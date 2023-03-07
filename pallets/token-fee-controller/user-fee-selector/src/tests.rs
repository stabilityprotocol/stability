use core::str::FromStr;

use crate::mock::{
	ExtBuilder, MeaninglessTokenAddress, MockDefaultFeeToken, MockSupportedTokensManager, Runtime,
	UserFeeTokenSelector,
};

use super::*;

use frame_support::{parameter_types, traits::Hooks};
use frame_system::pallet_prelude::BlockNumberFor;
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

		// not be changed until block is finalized
		assert_eq!(
			UserFeeTokenSelector::get_user_fee_token(MeaninglessAccount::get()),
			MockDefaultFeeToken::get()
		);

		<UserFeeTokenSelector as Hooks<BlockNumberFor<Runtime>>>::on_finalize(1);

		assert_eq!(
			UserFeeTokenSelector::get_user_fee_token(MeaninglessAccount::get()),
			MeaninglessTokenAddress::get()
		);
	});
}
