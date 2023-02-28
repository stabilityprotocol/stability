use super::*;

use sp_core::H160;

use mock::{
	AllowedAccountId, Logger, LoggerCall, NotAllowedAccountId, RootController, RuntimeCall,
	RuntimeEvent as TestEvent, RuntimeOrigin, System, Test, MockDefaultFeeToken, MeaninglessTokenAddress
};

parameter_types! {
	pub const MeaninglessAccount: H160 = H160::from_low_u64_be(1);
}

#[test]
fn get_default_token() {
	new_test_ext().execute_with(|| {
		assert_eq(Pallet::<Test>::get_user_fee_token(MeaninglessAccount::get()), MockDefaultFeeToken::get());
	});
}
