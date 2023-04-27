use fp_ethereum::TransactionValidationError;
use fp_evm::InvalidEvmTransactionError;
use sp_runtime::transaction_validity::InvalidTransaction;

pub struct InvalidTransactionWrapper(pub InvalidTransaction);

impl From<InvalidEvmTransactionError> for InvalidTransactionWrapper {
	fn from(validation_error: InvalidEvmTransactionError) -> Self {
		match validation_error {
			InvalidEvmTransactionError::GasLimitTooLow => InvalidTransactionWrapper(
				InvalidTransaction::Custom(TransactionValidationError::GasLimitTooLow as u8),
			),
			InvalidEvmTransactionError::GasLimitTooHigh => InvalidTransactionWrapper(
				InvalidTransaction::Custom(TransactionValidationError::GasLimitTooHigh as u8),
			),
			InvalidEvmTransactionError::GasPriceTooLow => {
				InvalidTransactionWrapper(InvalidTransaction::Payment)
			}
			InvalidEvmTransactionError::PriorityFeeTooHigh => InvalidTransactionWrapper(
				InvalidTransaction::Custom(TransactionValidationError::MaxFeePerGasTooLow as u8),
			),
			InvalidEvmTransactionError::BalanceTooLow => {
				InvalidTransactionWrapper(InvalidTransaction::Payment)
			}
			InvalidEvmTransactionError::TxNonceTooLow => {
				InvalidTransactionWrapper(InvalidTransaction::Stale)
			}
			InvalidEvmTransactionError::TxNonceTooHigh => {
				InvalidTransactionWrapper(InvalidTransaction::Future)
			}
			InvalidEvmTransactionError::InvalidPaymentInput => {
				InvalidTransactionWrapper(InvalidTransaction::Payment)
			}
			InvalidEvmTransactionError::InvalidChainId => InvalidTransactionWrapper(
				InvalidTransaction::Custom(TransactionValidationError::InvalidChainId as u8),
			),
		}
	}
}

impl From<InvalidTransactionWrapper> for InvalidTransaction {
	fn from(wrapper: InvalidTransactionWrapper) -> Self {
		wrapper.0
	}
}
