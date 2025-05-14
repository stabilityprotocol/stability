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

use frame_support::traits::{
	fungible::Inspect,
	tokens::{DepositConsequence, WithdrawConsequence},
	Currency, ExistenceRequirement, WithdrawReasons,
};
use sp_runtime::DispatchError;

use crate::mock::{new_test_ext, AccountId, CustomBalances, OneAddress, ZeroAddress};

// Currency<AccountId> functions

#[test]
fn return_balance_address_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances, ZeroAddress};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::total_balance(&ZeroAddress::get().into()),
			1
		)
	});
}

#[test]
fn return_balance_address_one() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances, OneAddress};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::total_balance(&OneAddress::get().into()),
			0
		)
	});
}

#[test]
fn return_free_balance_address_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::free_balance(&ZeroAddress::get().into()),
			1
		)
	});
}

#[test]
fn return_reducible_balance_address_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::free_balance(&ZeroAddress::get().into()),
			1
		)
	});
}

#[test]
fn can_slash_balance_not_higher() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		let balance =
			<CustomBalances as Currency<AccountId>>::total_balance(&ZeroAddress::get().into());
		assert!(<CustomBalances as Currency<AccountId>>::can_slash(
			&ZeroAddress::get().into(),
			balance,
		));

		assert_eq!(
			<CustomBalances as Currency<AccountId>>::can_slash(
				&ZeroAddress::get().into(),
				balance + 1,
			),
			false
		);
	});
}

#[test]
fn issue_returns_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::total_issuance(),
			0u128
		);
	});
}

#[test]
fn minimum_balance_returns_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::minimum_balance(),
			0u128
		);
	});
}

#[test]
fn burn_returns_zero_imbalance() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::burn(5),
			<CustomBalances as Currency<AccountId>>::PositiveImbalance::new(0)
		);
	});
}

#[test]
fn mint_returns_zero_imbalance() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::issue(5),
			<CustomBalances as Currency<AccountId>>::NegativeImbalance::new(0)
		);
	});
}

#[test]
fn ensure_can_withdraw_address_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};
	use frame_support::traits::WithdrawReasons;
	use sp_runtime::DispatchError;

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::ensure_can_withdraw(
				&ZeroAddress::get().into(),
				1,
				WithdrawReasons::all(),
				1,
			),
			Ok(())
		);

		assert_eq!(
			<CustomBalances as Currency<AccountId>>::ensure_can_withdraw(
				&ZeroAddress::get().into(),
				2,
				WithdrawReasons::all(),
				1,
			),
			Err(DispatchError::Other("Not enough balance"))
		);
	});
}

#[test]
fn ensure_transfer_dispatches_error_amount_higher_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::transfer(
				&ZeroAddress::get().into(),
				&ZeroAddress::get().into(),
				1u128,
				ExistenceRequirement::AllowDeath,
			),
			Ok(())
		);
	});
}

#[test]
fn ensure_transfer_success_amount_equals_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::transfer(
				&ZeroAddress::get().into(),
				&ZeroAddress::get().into(),
				0u128,
				ExistenceRequirement::AllowDeath,
			),
			Ok(())
		);
	});
}

#[test]
fn ensure_slash_returns_zero_imbalance() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::slash(&ZeroAddress::get().into(), 1u128,),
			(
				<CustomBalances as Currency<AccountId>>::PositiveImbalance::new(0),
				0u128
			)
		);
	});
}

#[test]
fn ensure_deposit_into_existing_dispatches_error() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::deposit_into_existing(
				&ZeroAddress::get().into(),
				1u128,
			),
			Err(DispatchError::Other(
				"Deposits are not allowed in this pallet",
			))
		);
	});
}

#[test]
fn ensure_resolve_into_existing_returns_err_imbalance() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		let imbalance = <CustomBalances as Currency<AccountId>>::NegativeImbalance::new(1);
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::resolve_into_existing(
				&ZeroAddress::get().into(),
				imbalance.clone(),
			),
			Err(imbalance)
		);
	});
}

#[test]
fn ensure_deposit_creating_returns_zero() {
	use frame_support::traits::Currency;

	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		let value = 1u128;
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::deposit_creating(
				&ZeroAddress::get().into(),
				value,
			),
			<CustomBalances as Currency<AccountId>>::NegativeImbalance::new(0)
		);
	});
}

#[test]
fn withdraw_dispatches_error() {
	new_test_ext().execute_with(|| {
		let value = 1u128;
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::withdraw(
				&ZeroAddress::get().into(),
				value,
				WithdrawReasons::all(),
				ExistenceRequirement::AllowDeath
			),
			Err(DispatchError::Other(
				"Withdrawals are not allowed in this pallet",
			)),
		);
	});
}

#[test]
fn settle_returns_error() {
	new_test_ext().execute_with(|| {
		let imbalance = <CustomBalances as Currency<AccountId>>::NegativeImbalance::new(1u128);
		assert_eq!(
			<CustomBalances as Currency<AccountId>>::settle(
				&ZeroAddress::get().into(),
				imbalance.clone(),
				WithdrawReasons::all(),
				ExistenceRequirement::AllowDeath
			),
			Err(imbalance),
		);
	});
}

// Inspect<AccountId> functions

#[test]
fn inspect_total_issuance_returns_zero() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::total_issuance(),
			0u128
		);
	});
}

#[test]
fn inspect_active_issuance_returns_zero() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::active_issuance(),
			0u128
		);
	});
}

#[test]
fn inspect_minimum_balance_returns_zero() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::minimum_balance(),
			0u128
		);
	});
}

#[test]
fn inspect_balance_returns_actual_balance_for_zero_address() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::balance(&ZeroAddress::get().into()),
			1u128
		);
	});
}

#[test]
fn inspect_balance_returns_actual_balance_for_one_address() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::balance(&OneAddress::get().into()),
			0u128
		);
	});
}

#[test]
fn inspect_reducible_balance_returns_actual_balance_for_zero_address() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::reducible_balance(
				&ZeroAddress::get().into(),
				frame_support::traits::tokens::Preservation::Preserve,
				frame_support::traits::tokens::Fortitude::Polite
			),
			1u128
		);
	});
}

#[test]
fn inspect_reducible_balance_returns_actual_balance_for_one_address() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::reducible_balance(
				&OneAddress::get().into(),
				frame_support::traits::tokens::Preservation::Preserve,
				frame_support::traits::tokens::Fortitude::Polite
			),
			0u128
		);
	});
}

#[test]
fn inspect_can_deposit_returns_error() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::can_deposit(
				&OneAddress::get().into(),
				10u128,
				frame_support::traits::tokens::Provenance::Extant
			),
			DepositConsequence::UnknownAsset
		);
	});
}

#[test]
fn inspect_can_withdraw_returns_error_when_not_enough_balance() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::can_withdraw(
				&ZeroAddress::get().into(),
				10u128
			),
			WithdrawConsequence::BalanceLow
		);
	});
}

#[test]
fn inspect_can_withdraw_success() {
	use crate::mock::{new_test_ext, AccountId, CustomBalances};

	new_test_ext().execute_with(|| {
		assert_eq!(
			<CustomBalances as Inspect<AccountId>>::can_withdraw(&ZeroAddress::get().into(), 1u128),
			WithdrawConsequence::Success
		);
	});
}
