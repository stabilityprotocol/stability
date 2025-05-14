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

use frame_support::parameter_types;
use pallet_supported_tokens_manager::SupportedTokensManager;
use sp_core::H160;

use crate::mock::{ExtBuilder, MockSupportedTokensManager, ValidatorFeeSelector, MeaninglessTokenAddress, NotSupportedToken, Runtime, };

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
fn update_default_controller() {
    let conversion_rate_controller : H160 = crate::GenesisConfig::<Runtime>::default().initial_default_conversion_rate_controller;
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::conversion_rate_controller(
            MeaninglessAccount::get()
        ), conversion_rate_controller);

        crate::Pallet::<Runtime>::set_default_controller(
            MeaninglessAccount::get()
        );

        assert_eq!(<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::conversion_rate_controller(
            MeaninglessAccount::get()
        ), MeaninglessAccount::get());
    });
}

#[test]
fn updated_token_conversion_rate() {
    ExtBuilder::default().build().execute_with(|| {
        let conversion_rate_controller : H160 = crate::GenesisConfig::<Runtime>::default().initial_default_conversion_rate_controller;
        assert!(<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::update_conversion_rate_controller(
            MeaninglessAccount::get(),
            conversion_rate_controller,
        ).is_ok());

        assert_eq!(<ValidatorFeeSelector as crate::ValidatorFeeTokenController>::conversion_rate_controller(
            MeaninglessAccount::get()
        ), conversion_rate_controller);
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