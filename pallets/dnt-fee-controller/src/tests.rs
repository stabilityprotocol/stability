#![cfg(test)]

use crate::mock::{
	FeeVaultAddress, MeaninglessConversionRate, MeaninglessTokenAddress, MockCallsStorage,
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
			MeaninglessTokenAddress::get()
		),
		MeaninglessConversionRate::get()
	);
}

#[test]
fn pay_fees_calls_vault_pallet() {
	new_test_ext().execute_with(|| {
		let meaningless_amount = 100.into();
		let result = <Pallet<Test> as OnChargeDecentralizedNativeTokenFee>::pay_fees(
			MeaninglessTokenAddress::get(),
			(1.into(), 1.into()),
			meaningless_amount,
			MeaninglessAddress::get(),
			MeaninglessAddress2::get(),
		);

		assert!(result.is_ok());

		assert!(
			pallet_fee_rewards_vault::Pallet::<Test>::claimable_reward(
				MeaninglessAddress::get(),
				MeaninglessTokenAddress::get(),
			) > 0.into(),
		);

		assert!(
			pallet_fee_rewards_vault::Pallet::<Test>::claimable_reward(
				MeaninglessAddress2::get(),
				MeaninglessTokenAddress::get(),
			) > 0.into(),
		);
	})
}
