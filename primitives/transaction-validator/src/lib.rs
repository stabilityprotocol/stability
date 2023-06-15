#![cfg_attr(not(feature = "std"), no_std)]

use core::{marker::PhantomData, ops::Mul};

use ethereum::TransactionV2;
use fp_ethereum::TransactionData;
use fp_evm::{CheckEvmTransaction, CheckEvmTransactionConfig, FeeCalculator};
use pallet_ethereum::InvalidTransactionWrapper;
use pallet_user_fee_selector::UserFeeTokenController;
use sp_core::{Get, H160, U256};
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
			let base_fee = <T as pallet_evm::Config>::FeeCalculator::min_gas_price().0;

			let gas_price = stbl_tools::eth::transaction_gas_price(base_fee, &transaction, true)
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;

			let transaction_data: TransactionData = (transaction).into();
			let total_transaction_price =
				transaction_data.gas_limit.mul(gas_price) + transaction_data.value;

			let balance =
				<pallet_user_fee_selector::Pallet<T> as UserFeeTokenController>::balance_of(
					*origin,
				);

			match balance >= total_transaction_price {
				true => Self::build_validity_success_transaction(origin, transaction, gas_price),
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
		gas_price: U256,
	) -> TransactionValidity {
		let transaction_data: TransactionData = transaction.into();

		let (account, _) = pallet_evm::Pallet::<T>::account_basic(origin);

		let base_fee = <T as pallet_evm::Config>::FeeCalculator::min_gas_price().0;

		CheckEvmTransaction::<InvalidTransactionWrapper>::new(
			CheckEvmTransactionConfig {
				evm_config: <T as pallet_evm::Config>::config(),
				block_gas_limit: <T as pallet_evm::Config>::BlockGasLimit::get(),
				base_fee: base_fee.clone(),
				chain_id: <T as pallet_evm::Config>::ChainId::get(),
				is_transactional: true,
			},
			transaction_data.clone().into(),
		)
		.validate_in_pool_for(&account)
		.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;

		if gas_price < base_fee {
			return Err(TransactionValidityError::Invalid(
				InvalidTransaction::Payment,
			));
		}

		ValidTransactionBuilder::default()
			.and_provides((origin, transaction_data.nonce))
			.priority(stbl_tools::misc::truncate_u256_to_u64(gas_price))
			.build()
	}
}
