#![cfg_attr(not(feature = "std"), no_std)]

use core::{marker::PhantomData, ops::Mul};

use ethereum::TransactionV2;
use fp_ethereum::TransactionData;
use fp_evm::{CheckEvmTransactionConfig, FeeCalculator, InvalidEvmTransactionError};
use pallet_user_fee_selector::UserFeeTokenController;
use sp_core::{Get, H160};
use sp_runtime::transaction_validity::{
	InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransactionBuilder,
};

mod transaction_error_wrapper;
use transaction_error_wrapper::InvalidTransactionWrapper;

pub struct FallbackTransactionValidator<T>(PhantomData<T>);

impl<T> FallbackTransactionValidator<T>
where
	T: pallet_evm::Config
		+ pallet_ethereum::Config
		+ frame_system::Config
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

			stbl_tools::tertiary_operator!(
				balance >= total_transaction_price,
				Self::build_validity_success_transaction(origin, transaction),
				Err(TransactionValidityError::Invalid(
					InvalidTransaction::Payment
				))
			)
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
		let check: fp_evm::CheckEvmTransaction<InvalidTransactionWrapper> =
			fp_evm::CheckEvmTransaction::<InvalidTransactionWrapper>::new(
				Self::transaction_check_builder(),
				transaction_data.clone().into(),
			);

		let (mut who, _) = pallet_evm::Pallet::<T>::account_basic(origin);

		who.balance = pallet_user_fee_selector::Pallet::<T>::balance_of(*origin);

		check
			.validate_in_pool_for(&who)
			.and_then(|v| v.with_chain_id())
			.and_then(|v| v.with_base_fee())
			.and_then(|v| v.with_balance_for(&who))
			.map_err(|e| TransactionValidityError::Invalid(e.0))?;

		ValidTransactionBuilder::default()
			.and_provides((origin, transaction_data.nonce))
			.priority(stbl_tools::eth::transaction_gas_price(transaction))
			.build()
	}

	fn transaction_check_builder() -> CheckEvmTransactionConfig<'static> {
		CheckEvmTransactionConfig {
			evm_config: <T as pallet_evm::Config>::config(),
			block_gas_limit: <T as pallet_evm::Config>::BlockGasLimit::get(),
			base_fee: <T as pallet_evm::Config>::FeeCalculator::min_gas_price().0,
			chain_id: <T as pallet_evm::Config>::ChainId::get(),
			is_transactional: true,
		}
	}
}
