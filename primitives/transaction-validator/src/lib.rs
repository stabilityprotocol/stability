#![cfg_attr(not(feature = "std"), no_std)]

use core::{marker::PhantomData, ops::Mul};

use ethereum::TransactionV2;
use fp_ethereum::TransactionData;
use pallet_user_fee_selector::UserFeeTokenController;
use sp_core::H160;
use sp_runtime::transaction_validity::{
	InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransactionBuilder,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub struct FallbackTransactionValidator<T>(PhantomData<T>);

impl<T> FallbackTransactionValidator<T>
where
	T: frame_system::Config
		+ pallet_evm::Config
		+ pallet_ethereum::Config
		+ pallet_user_fee_selector::Config,
	Result<pallet_ethereum::RawOrigin, <T as frame_system::Config>::RuntimeOrigin>:
		From<<T as frame_system::Config>::RuntimeOrigin>,
{
	pub fn check_actual_balance(
		origin: &H160,
		call: &pallet_ethereum::Call<T>,
	) -> TransactionValidity {
		if let pallet_ethereum::Call::transact { transaction } = call {
			let gas_price = stbl_tools::eth::transaction_gas_price(&transaction);
			let transaction_data: TransactionData = (transaction).into();
			let total_transaction_price =
				transaction_data.gas_limit.mul(gas_price) + transaction_data.value;

			let balance =
				<pallet_user_fee_selector::Pallet<T> as UserFeeTokenController>::balance_of(
					*origin,
				);

			match balance >= total_transaction_price {
				true => Self::build_validity_success_transaction(origin, transaction),
				false => Err(TransactionValidityError::Invalid(
					InvalidTransaction::Payment,
				)),
			}
		} else {
			Err(TransactionValidityError::Invalid(
				InvalidTransaction::Payment,
			))
		}
	}

	fn build_validity_success_transaction(
		origin: &H160,
		transaction: &TransactionV2,
	) -> TransactionValidity {
		let transaction_data: TransactionData = transaction.into();

		ValidTransactionBuilder::default()
			.and_provides((origin, transaction_data.nonce))
			.priority(stbl_tools::eth::transaction_gas_price(transaction))
			.build()
	}
}
