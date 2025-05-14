// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

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

use crate::mock::{new_test_ext, FundedAccount, NoFundsAccount, Runtime};

use ethereum::{EIP2930Transaction, TransactionAction, TransactionV2 as Transaction};
use fp_evm::FeeCalculator;
use frame_support::parameter_types;
use sp_core::{H160, H256, U256};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

parameter_types! {
	pub SampleTransaction: Transaction = Transaction::EIP2930(EIP2930Transaction {
		chain_id: 0u64,
		nonce: U256::from(0),
		gas_price: <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0,
		gas_limit: U256::from(21000),
		action: TransactionAction::Call(H160::zero()),
		value: U256::from(1_000_000_000),
		input: Default::default(),
		access_list: Default::default(),
		odd_y_parity: false,
		r: H256::from_low_u64_be(0),
		s: H256::from_low_u64_be(0),
	});
}

parameter_types! {
	pub NoFeesTransaction: Transaction = Transaction::EIP2930(EIP2930Transaction {
		chain_id: 0u64,
		nonce: U256::from(0),
		gas_price: 0.into(),
		gas_limit: U256::from(21000),
		action: TransactionAction::Call(H160::zero()),
		value: U256::from(1_000_000_000),
		input: Default::default(),
		access_list: Default::default(),
		odd_y_parity: false,
		r: H256::from_low_u64_be(0),
		s: H256::from_low_u64_be(0),
	});
}

#[test]
fn fail_sending_over_limit() {
	new_test_ext().execute_with(|| {
		let result = crate::FallbackTransactionValidator::<Runtime>::check_actual_balance(
			&NoFundsAccount::get(),
			&pallet_ethereum::Call::transact {
				transaction: SampleTransaction::get().clone(),
			},
		);

		assert!(result.is_err())
	})
}

#[test]
fn success_sending_under_limit() {
	new_test_ext().execute_with(|| {
		let result = crate::FallbackTransactionValidator::<Runtime>::check_actual_balance(
			&FundedAccount::get(),
			&pallet_ethereum::Call::transact {
				transaction: SampleTransaction::get().clone(),
			},
		);

		assert!(result.is_ok())
	})
}

#[test]
fn no_fee_transaction_rejected() {
	new_test_ext().execute_with(|| {
		let result = crate::FallbackTransactionValidator::<Runtime>::check_actual_balance(
			&FundedAccount::get(),
			&pallet_ethereum::Call::transact {
				transaction: NoFeesTransaction::get().clone(),
			},
		);

		assert!(matches!(
			result,
			Err(TransactionValidityError::Invalid(
				InvalidTransaction::Payment
			))
		))
	})
}
