#![cfg(test)]

use crate::mock::{new_test_ext, FundedAccount, NoFundsAccount, Runtime};

use ethereum::{EIP2930Transaction, TransactionAction, TransactionV2 as Transaction};
use frame_support::parameter_types;
use sp_core::{H160, H256, U256};

parameter_types! {
	pub SampleTransaction: Transaction = Transaction::EIP2930(EIP2930Transaction {
		chain_id: 0u64,
		nonce: U256::from(0),
		gas_price: U256::from(0),
		gas_limit: U256::from(0),
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
