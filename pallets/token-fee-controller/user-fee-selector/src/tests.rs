use super::*;

use sp_core::H160;

use mock::{new_test_ext, MockDefaultFeeToken, Test, MeaninglessAccount};

parameter_types! {
	pub const MeaninglessAccount: H160 = H160::from_low_u64_be(1);
}

#[test]
fn get_default_token() {
	new_test_ext().execute_with(|| {
		assert_eq(Pallet::<Test>::get_user_fee_token(MeaninglessAccount::get()), MockDefaultFeeToken::get());
	});
}
