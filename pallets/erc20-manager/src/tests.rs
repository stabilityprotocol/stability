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

use crate::mock::{new_test_ext, ERC20SlotZero, MeaninglessAddress, Test};
use pallet_evm::Runner;
use sp_core::{H160, U256};

type TestRunner = <Test as pallet_evm::Config>::Runner;

fn get_balance_of(erc20: H160, address: H160) -> U256 {
	TestRunner::call(
		address,
		erc20,
		stbl_tools::eth::generate_calldata("balanceOf(address)", &vec![address.into()]),
		0.into(),
		u64::MAX,
		None,
		None,
		None,
		Default::default(),
		false,
		false,
		None,
		None,
		&pallet_evm::EvmConfig::istanbul(),
	)
	.unwrap()
	.value
	.as_slice()
	.try_into()
	.unwrap()
}

#[test]
fn default_balance_is_zero() {
	new_test_ext().execute_with(|| {
		let balance = get_balance_of(ERC20SlotZero::get(), MeaninglessAddress::get());

		assert_eq!(balance, 0.into());

		let storage_balance = <crate::Pallet<Test> as crate::ERC20Manager>::balance_of(
			ERC20SlotZero::get(),
			MeaninglessAddress::get(),
		);

		assert_eq!(storage_balance, 0.into());
	});
}

#[test]
fn deposit_and_withdraw() {
	new_test_ext().execute_with(|| {
		let value_minted = U256::MAX;

		assert!(
			<crate::Pallet<Test> as crate::ERC20Manager>::deposit_amount(
				ERC20SlotZero::get(),
				MeaninglessAddress::get(),
				value_minted,
			)
			.is_ok()
		);

		let mut balance = get_balance_of(ERC20SlotZero::get(), MeaninglessAddress::get());

		assert_eq!(balance, value_minted);

		let storage_balance = <crate::Pallet<Test> as crate::ERC20Manager>::balance_of(
			ERC20SlotZero::get(),
			MeaninglessAddress::get(),
		);

		assert_eq!(storage_balance, value_minted);

		assert!(
			<crate::Pallet<Test> as crate::ERC20Manager>::withdraw_amount(
				ERC20SlotZero::get(),
				MeaninglessAddress::get(),
				value_minted,
			)
			.is_ok()
		);

		balance = get_balance_of(ERC20SlotZero::get(), MeaninglessAddress::get());

		assert_eq!(balance, 0.into());
	});
}

#[test]
fn fail_not_enough_balance() {
	new_test_ext().execute_with(|| {
		assert!(
			<crate::Pallet<Test> as crate::ERC20Manager>::withdraw_amount(
				ERC20SlotZero::get(),
				MeaninglessAddress::get(),
				1000.into(),
			)
			.is_err()
		);
	});
}

#[test]
fn fail_when_overflow() {
	new_test_ext().execute_with(|| {
		assert!(
			<crate::Pallet<Test> as crate::ERC20Manager>::deposit_amount(
				ERC20SlotZero::get(),
				MeaninglessAddress::get(),
				U256::MAX,
			)
			.is_ok()
		);

		assert!(
			<crate::Pallet<Test> as crate::ERC20Manager>::deposit_amount(
				ERC20SlotZero::get(),
				MeaninglessAddress::get(),
				U256::MAX,
			)
			.is_err()
		);
	});
}
