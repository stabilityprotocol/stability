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

#![cfg(test)]

use crate::mock::{
	DNTFeeController, FeeVaultAddress, MeaninglessConversionRate, MeaninglessTokenAddress,
	MockCallsStorage, MockFailsStorage,
};

use super::*;
use mock::{new_test_ext, MeaninglessAddress, MeaninglessAddress2, Test};
use runner::OnChargeDecentralizedNativeTokenFee;

#[test]
fn withdraw_fee_calls_deposit_and_withdraw() {
	new_test_ext().execute_with(|| {
		let meaningless_amount = 100.into();
		let result = <Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::withdraw_fee(
			MeaninglessAddress::get(),
			MeaninglessTokenAddress::get(),
			(1.into(), 1.into()),
			meaningless_amount,
		);

		assert!(result.is_ok());

		assert_eq!(
			MockCallsStorage::get("withdraw_amount"),
			vec![(
				MeaninglessTokenAddress::get(),
				MeaninglessAddress::get(),
				meaningless_amount
			)]
		);

		assert_eq!(
			MockCallsStorage::get("deposit_amount"),
			vec![(
				MeaninglessTokenAddress::get(),
				FeeVaultAddress::get(),
				meaningless_amount
			)]
		);
	})
}

#[test]
fn correct_fee_calls_deposit_and_withdraw() {
	new_test_ext().execute_with(|| {
		let paid_amount = 100.into();
		let actual_amount = 10.into();
		let result = <Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::correct_fee(
			MeaninglessAddress::get(),
			MeaninglessTokenAddress::get(),
			(1.into(), 1.into()),
			paid_amount,
			actual_amount,
		);

		assert!(result.is_ok());

		assert_eq!(
			MockCallsStorage::get("withdraw_amount"),
			vec![(
				MeaninglessTokenAddress::get(),
				FeeVaultAddress::get(),
				paid_amount - actual_amount
			)]
		);

		assert_eq!(
			MockCallsStorage::get("deposit_amount"),
			vec![(
				MeaninglessTokenAddress::get(),
				MeaninglessAddress::get(),
				paid_amount - actual_amount
			)]
		);
	});
}

#[test]
fn transaction_fee_token_returns_user_token() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			<Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::get_transaction_fee_token(
				MeaninglessAddress::get(),
			),
			MeaninglessTokenAddress::get()
		);
	})
}

#[test]
fn transaction_fee_cr_returns_validator_cr() {
	assert_eq!(
		<Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::get_transaction_conversion_rate(
			MeaninglessAddress::get(),
			MeaninglessAddress::get(),
			MeaninglessTokenAddress::get()
		),
		MeaninglessConversionRate::get()
	);
}

#[test]
fn pay_fees_calls_vault_pallet() {
	new_test_ext().execute_with(|| {
		let meaningless_amount = 100.into();
		let validator_amount =
			DNTFeeController::validator_percentage().unwrap() * meaningless_amount / 100;
		let result = <Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::pay_fees(
			MeaninglessTokenAddress::get(),
			(1.into(), 1.into()),
			meaningless_amount,
			MeaninglessAddress::get(),
			Some(MeaninglessAddress2::get()),
		);

		assert!(result.is_ok());

		assert_eq!(
			pallet_fee_rewards_vault::Pallet::<Test>::claimable_reward(
				MeaninglessAddress::get(),
				MeaninglessTokenAddress::get(),
			),
			validator_amount,
		);

		assert_eq!(
			pallet_fee_rewards_vault::Pallet::<Test>::claimable_reward(
				MeaninglessAddress2::get(),
				MeaninglessTokenAddress::get(),
			),
			meaningless_amount - validator_amount,
		);
	})
}

#[test]
fn pay_fees_calls_to_none_address() {
	new_test_ext().execute_with(|| {
		let meaningless_amount = 100.into();
		let validator_amount = meaningless_amount;

		let result = <Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::pay_fees(
			MeaninglessTokenAddress::get(),
			(1.into(), 1.into()),
			meaningless_amount,
			MeaninglessAddress::get(),
			None,
		);

		assert!(result.is_ok());

		let (returned_validator_amount, returned_dapp_amount) = result.unwrap();

		assert_eq!(returned_validator_amount, validator_amount);

		assert_eq!(returned_dapp_amount, 0.into(),);

		assert_eq!(
			pallet_fee_rewards_vault::Pallet::<Test>::claimable_reward(
				MeaninglessAddress::get(),
				MeaninglessTokenAddress::get(),
			),
			validator_amount,
		);

		assert_eq!(
			pallet_fee_rewards_vault::Pallet::<Test>::claimable_reward(
				MeaninglessAddress2::get(),
				MeaninglessTokenAddress::get(),
			),
			0.into(),
		);
	})
}

#[test]
fn pay_fees_calls_with_updated_percentages() {
	new_test_ext().execute_with(|| {
		let new_percentage = 10.into();

		assert!(DNTFeeController::set_validator_percentage(new_percentage).is_ok());

		let meaningless_amount = 100.into();
		let validator_amount = new_percentage * meaningless_amount / 100;

		let result = <Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::pay_fees(
			MeaninglessTokenAddress::get(),
			(1.into(), 1.into()),
			meaningless_amount,
			MeaninglessAddress::get(),
			Some(MeaninglessAddress2::get()),
		);

		assert!(result.is_ok());

		let (returned_validator_amount, returned_dapp_amount) = result.unwrap();

		assert_eq!(returned_validator_amount, validator_amount);
		assert_eq!(returned_dapp_amount, meaningless_amount - validator_amount);

		assert_eq!(
			pallet_fee_rewards_vault::Pallet::<Test>::claimable_reward(
				MeaninglessAddress::get(),
				MeaninglessTokenAddress::get(),
			),
			validator_amount,
		);

		assert_eq!(
			pallet_fee_rewards_vault::Pallet::<Test>::claimable_reward(
				MeaninglessAddress2::get(),
				MeaninglessTokenAddress::get(),
			),
			meaningless_amount - validator_amount,
		);
	})
}

#[test]
fn fails_correct_if_erc20_manager_fails() {
	new_test_ext().execute_with(|| {
		MockFailsStorage::put(true);
		let paid_amount = 100.into();
		let actual_amount = 10.into();
		let result = <Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::correct_fee(
			MeaninglessAddress::get(),
			MeaninglessTokenAddress::get(),
			(1.into(), 1.into()),
			paid_amount,
			actual_amount,
		);

		assert!(result.is_err());
	})
}

#[test]
fn fails_withdraw_if_erc20_manager_fails() {
	new_test_ext().execute_with(|| {
		MockFailsStorage::put(true);
		let paid_amount = 100.into();
		let result = <Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::withdraw_fee(
			MeaninglessAddress::get(),
			MeaninglessTokenAddress::get(),
			(1.into(), 1.into()),
			paid_amount,
		);

		assert!(result.is_err());
	})
}

#[test]
fn validator_percentage_updates() {
	new_test_ext().execute_with(|| {
		assert_eq!(DNTFeeController::validator_percentage().unwrap(), 50.into());

		assert!(DNTFeeController::set_validator_percentage(10.into()).is_ok());

		assert_eq!(DNTFeeController::validator_percentage().unwrap(), 10.into());
	});
}
