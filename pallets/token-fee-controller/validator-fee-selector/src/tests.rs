use frame_support::parameter_types;
use pallet_supported_tokens_manager::SupportedTokensManager;
use sp_core::{H160, U256};

use crate::mock::{
	ExtBuilder, MeaninglessTokenAddress, MockSupportedTokensManager, NotSupportedToken,
	ValidatorFeeSelector,
};

parameter_types! {
	pub MeaninglessAccount: H160 = H160::from_low_u64_le(1);
}

#[test]
fn build() {
	ExtBuilder::default().build().execute_with(|| {});
}

#[test]
fn validator_supports_default_unless_explicitly_denies() {
	ExtBuilder::default().build().execute_with(|| {
        assert!(
            <ValidatorFeeSelector as crate::ValidatorFeeTokenController>::validator_supports_fee_token(
                MeaninglessAccount::get(),
                MockSupportedTokensManager::get_default_token(),
            )
        );
    
        assert!(
            <ValidatorFeeSelector as crate::ValidatorFeeTokenController>::update_fee_token_acceptance(
                MeaninglessAccount::get(),
                MockSupportedTokensManager::get_default_token(),
                false,
            )
            .is_ok()
        );
    
        assert_eq!(
            <ValidatorFeeSelector as crate::ValidatorFeeTokenController>::validator_supports_fee_token(
                MeaninglessAccount::get(),
                MockSupportedTokensManager::get_default_token(),
            ), false
        );
    });
}

#[test]
fn non_default_token_not_accepted_as_default() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::validator_supports_fee_token(
            MeaninglessAccount::get(),
            MeaninglessTokenAddress::get(),
        ) == false)
	});
}

#[test]
fn fail_update_not_supported_token_acceptance() {
	ExtBuilder::default().build().execute_with(|| {
        assert!(
            <ValidatorFeeSelector as crate::ValidatorFeeTokenController>::update_fee_token_acceptance(
                MeaninglessAccount::get(),
                NotSupportedToken::get(),
                false,
            )
            .is_err()
        );
    });
}

#[test]
fn updated_token_conversion_rate() {
	ExtBuilder::default().build().execute_with(|| {
		let conversion_rate: (U256, U256) = (2.into(), 1.into());
		assert!(
			<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::update_conversion_rate(
				MeaninglessAccount::get(),
				MeaninglessTokenAddress::get(),
				conversion_rate,
			)
			.is_ok()
		);

		assert_eq!(
			<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::conversion_rate(
				MeaninglessAccount::get(),
				MeaninglessTokenAddress::get()
			),
			conversion_rate
		);
	});
}

#[test]
fn not_supported_by_validator_if_not_supported_by_chain() {
	ExtBuilder::default().build().execute_with(|| {
        let token = MeaninglessTokenAddress::get();

        assert!(<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::update_fee_token_acceptance(
            MeaninglessAccount::get(),
            token,
            true,
        ).is_ok());

        assert!(MockSupportedTokensManager::remove_supported_token(MeaninglessTokenAddress::get()).is_ok());

        assert_eq!(<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::validator_supports_fee_token(
            MeaninglessAccount::get(),
            token,
        ), false);
    });
}
