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

#![cfg_attr(not(feature = "std"), no_std)]

use log;
use pallet_evm::TransactionValidationError;
use sp_core::H160;

pub const LOG_TARGET: &'static str = "zero-gas-transactions";

#[derive(Debug, PartialEq)]
pub enum EthereumTxError {
	GasLimitTooLow,
	GasLimitTooHigh,
	GasPriceTooLow,
	PriorityFeeTooHigh,
	BalanceTooLow,
	TxNonceTooLow,
	TxNonceTooHigh,
	InvalidFeeInput,
	InvalidChainId,
	InvalidSignature,
	UnknownError,
}

impl From<TransactionValidationError> for EthereumTxError {
	fn from(e: TransactionValidationError) -> Self {
		match e {
			TransactionValidationError::GasLimitTooLow => EthereumTxError::GasLimitTooLow,
			TransactionValidationError::GasLimitTooHigh => EthereumTxError::GasLimitTooHigh,
			TransactionValidationError::GasPriceTooLow => EthereumTxError::GasPriceTooLow,
			TransactionValidationError::PriorityFeeTooHigh => EthereumTxError::PriorityFeeTooHigh,
			TransactionValidationError::BalanceTooLow => EthereumTxError::BalanceTooLow,
			TransactionValidationError::TxNonceTooLow => EthereumTxError::TxNonceTooLow,
			TransactionValidationError::TxNonceTooHigh => EthereumTxError::TxNonceTooHigh,
			TransactionValidationError::InvalidFeeInput => EthereumTxError::InvalidFeeInput,
			TransactionValidationError::InvalidChainId => EthereumTxError::InvalidChainId,
			TransactionValidationError::InvalidSignature => EthereumTxError::InvalidSignature,
			TransactionValidationError::UnknownError => EthereumTxError::UnknownError,
		}
	}
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use fp_ethereum::TransactionData;
	use fp_evm::FeeCalculator;
	use frame_support::dispatch::GetDispatchInfo;
	use frame_support::pallet_prelude::*;
	use frame_support::sp_runtime::traits::UniqueSaturatedInto;
	use frame_system::pallet_prelude::*;
	use pallet_evm::GasWeightMapping;
	use parity_scale_codec::alloc::string::ToString;
	use sp_core::H256;
	use sp_core::U256;
	use sp_std::vec;
	use sp_std::vec::Vec;

	pub use fp_rpc::TransactionStatus;

	use pallet_ethereum::Transaction;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config + pallet_ethereum::Config {
		type RuntimeCall: Parameter + GetDispatchInfo;
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T>
	where
		Result<pallet_ethereum::RawOrigin, <T as frame_system::Config>::RuntimeOrigin>:
			From<<T as frame_system::Config>::RuntimeOrigin>,
		<T as frame_system::Config>::RuntimeOrigin: From<pallet_ethereum::RawOrigin>,
	{
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::send_zero_gas_transaction {
					transaction,
					validator_signature,
				} => {
					let from =
						Self::ensure_transaction_signature(transaction.clone()).map_err(|_| {
							TransactionValidityError::Invalid(InvalidTransaction::BadProof)
						})?;

					let current_block_validator = <pallet_evm::Pallet<T>>::find_author();

					Self::ensure_zero_gas_transaction(
						current_block_validator,
						validator_signature.clone(),
					)
					.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadProof))?;

					let transaction_data: TransactionData = transaction.into();

					Self::pool_ensure_transaction_unicity(&from, transaction)
						.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

					return sp_runtime::transaction_validity::ValidTransactionBuilder::default()
						.and_provides((from, transaction_data.nonce))
						.priority(u64::MAX)
						.build();
				}
				_ => Err(TransactionValidityError::Unknown(
					UnknownTransaction::Custom(0),
				)),
			}?;

			return sp_runtime::transaction_validity::ValidTransactionBuilder::default()
				.and_provides((H160::zero(), U256::zero()))
				.priority(u64::MAX)
				.build();
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		Result<pallet_ethereum::RawOrigin, <T as frame_system::Config>::RuntimeOrigin>:
			From<<T as frame_system::Config>::RuntimeOrigin>,
		<T as frame_system::Config>::RuntimeOrigin: From<pallet_ethereum::RawOrigin>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight({
			let without_base_extrinsic_weight = true;
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				let transaction_data: TransactionData = transaction.into();
				transaction_data.gas_limit.unique_saturated_into()
			}, without_base_extrinsic_weight)
		})]
		pub fn send_zero_gas_transaction(
			_origin: OriginFor<T>,
			transaction: Transaction,
			validator_signature: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let from = Self::ensure_transaction_signature(transaction.clone())
				.map_err(|_| DispatchError::Other("Invalid transaction signature"))?;

			Self::block_ensure_transaction_unicity(&from, &transaction)
				.map_err(|_| DispatchError::Other("Invalid transaction data"))?;

			let current_block_validator = <pallet_evm::Pallet<T>>::find_author();

			Self::ensure_zero_gas_transaction(current_block_validator, validator_signature)
				.map_err(|_| DispatchError::Other("Invalid zero gas transaction signature"))?;

			let origin: T::RuntimeOrigin =
				pallet_ethereum::Origin::EthereumTransaction(from).into();

			let dispatch =
				pallet_ethereum::Pallet::<T>::transact(origin, transaction).map_err(|e| {
					log::debug!(target: LOG_TARGET, "Dispatch transaction error: {:?}", e);
					DispatchError::Other("Signature doesn't meet with sponsor address")
				})?;

			let used_gas = Self::gas_from_actual_weight(dispatch.actual_weight.unwrap())
				.map_err(|_| DispatchError::Other("Arithmetic error due to overflows"))?;

			Ok(frame_support::dispatch::PostDispatchInfo {
				actual_weight: Some(T::GasWeightMapping::gas_to_weight(
					used_gas.unique_saturated_into(),
					true,
				)),
				pays_fee: Pays::No,
			})
		}
	}

	impl<T: Config> Pallet<T> {
		fn gas_from_actual_weight(weight: Weight) -> Result<u64, ()> {
			let actual_weight = match weight.checked_add(
				&T::BlockWeights::get()
					.get(frame_support::dispatch::DispatchClass::Normal)
					.base_extrinsic,
			) {
				Some(v) => v,
				None => return Err(()),
			};

			Ok(<T as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
				actual_weight,
			))
		}

		fn ensure_transaction_signature(
			transaction: pallet_ethereum::Transaction,
		) -> Result<H160, ()> {
			match stbl_tools::eth::recover_signer(&transaction) {
				None => Err(()),
				Some(address) => Ok(address),
			}
		}

		fn pool_ensure_transaction_unicity(
			origin: &H160,
			transaction: &pallet_ethereum::Transaction,
		) -> Result<(), ()> {
			let transaction_data: TransactionData = transaction.into();
			let (base_fee, _) = <T as pallet_evm::Config>::FeeCalculator::min_gas_price();
			let (who, _) = pallet_evm::Pallet::<T>::account_basic(origin);

			fp_evm::CheckEvmTransaction::<EthereumTxError>::new(
				fp_evm::CheckEvmTransactionConfig {
					evm_config: T::config(),
					block_gas_limit: T::BlockGasLimit::get(),
					base_fee,
					chain_id: T::ChainId::get(),
					is_transactional: true,
				},
				transaction_data.into(),
				None,
				None,
			)
			.validate_in_pool_for(&who)
			.and_then(|v| v.with_chain_id())
			.map_err(|e| {
				log::debug!(target: LOG_TARGET, "Transaction validation error: {:?}", e);
				()
			})?;

			Ok(())
		}

		fn block_ensure_transaction_unicity(
			origin: &H160,
			transaction: &pallet_ethereum::Transaction,
		) -> Result<(), ()> {
			let transaction_data: TransactionData = transaction.into();
			let (base_fee, _) = <T as pallet_evm::Config>::FeeCalculator::min_gas_price();
			let (who, _) = pallet_evm::Pallet::<T>::account_basic(origin);

			fp_evm::CheckEvmTransaction::<EthereumTxError>::new(
				fp_evm::CheckEvmTransactionConfig {
					evm_config: T::config(),
					block_gas_limit: T::BlockGasLimit::get(),
					base_fee,
					chain_id: T::ChainId::get(),
					is_transactional: true,
				},
				transaction_data.into(),
				None,
				None,
			)
			.validate_in_block_for(&who)
			.and_then(|v| v.with_chain_id())
			.map_err(|e| {
				log::debug!(target: LOG_TARGET, "Transaction validation error: {:?}", e);
				()
			})?;

			Ok(())
		}

		fn ensure_zero_gas_transaction(
			expected_validator: H160,
			validator_signature: Vec<u8>,
		) -> Result<(), ()> {
			let chain_id = T::ChainId::get();
			let block_number = UniqueSaturatedInto::<u64>::unique_saturated_into(
				frame_system::Pallet::<T>::block_number(),
			);

			let zero_gas_trx_internal_message: Vec<u8> =
				Self::get_zero_gas_transaction_signing_message(block_number.into(), chain_id);

			let eip191_message =
				stbl_tools::eth::build_eip191_message_hash(zero_gas_trx_internal_message);

			let zero_gas_trx_signer_address =
				Self::get_zero_gas_trx_signer(validator_signature.clone(), eip191_message.clone());

			match zero_gas_trx_signer_address {
				Some(address) if address == expected_validator => Ok(()),
				_ => Err(()),
			}
		}

		pub fn get_zero_gas_transaction_signing_message(
			block_number: u64,
			chain_id: u64,
		) -> Vec<u8> {
			b"I consent to validate zero gas transactions in block "
				.iter()
				.chain(block_number.to_string().as_bytes().iter())
				.chain(b" on chain ")
				.chain(chain_id.to_string().as_bytes().iter())
				.cloned()
				.collect()
		}

		fn get_zero_gas_trx_signer(signature: Vec<u8>, message: H256) -> Option<H160> {
			let result = match sp_io::crypto::secp256k1_ecdsa_recover(
				signature.as_slice().try_into().unwrap(),
				message.as_fixed_bytes(),
			) {
				Ok(pubkey) => {
					let mut address = sp_io::hashing::keccak_256(&pubkey);
					address[0..12].copy_from_slice(&[0u8; 12]);
					address.to_vec()
				}
				Err(_) => return None,
			};

			return Some(H160::from_slice(&result[12..32]));
		}
	}
}
