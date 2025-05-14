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

#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use fp_ethereum::TransactionData;
	use fp_evm::CheckEvmTransactionConfig;
	use fp_evm::FeeCalculator;
	use frame_support::dispatch::GetDispatchInfo;
	use frame_support::pallet_prelude::{StorageMap, *};
	use frame_support::sp_runtime::traits::UniqueSaturatedInto;
	use frame_support::weights::Weight;
	use frame_system::pallet_prelude::*;
	use pallet_erc20_manager::ERC20Manager;
	use pallet_evm::GasWeightMapping;
	use runner::OnChargeDecentralizedNativeTokenFee;
	use sp_core::{H256, U256};
	use sp_std::{vec, vec::Vec};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type SponsorNonce<T: Config> = StorageMap<_, Blake2_128Concat, H160, u64, ValueQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config + pallet_ethereum::Config {
		type RuntimeCall: Parameter + GetDispatchInfo;
		type ERC20Manager: ERC20Manager;
		type DNTFeeController: runner::OnChargeDecentralizedNativeTokenFee;
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
				Call::send_sponsored_transaction {
					transaction,
					meta_trx_sponsor,
					meta_trx_sponsor_signature,
				} => {
					let from =
						Self::ensure_transaction_signature(transaction.clone()).map_err(|_| {
							TransactionValidityError::Invalid(InvalidTransaction::BadProof)
						})?;

					Self::ensure_meta_transaction_sponsor(
						transaction.clone(),
						meta_trx_sponsor.clone(),
						meta_trx_sponsor_signature.clone(),
					)
					.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadProof))?;

					Self::pool_ensure_transaction_unicity(&from, &transaction)
						.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

					let transaction_data: TransactionData = transaction.into();

					let (_, gas_price) = Self::get_transaction_gas_info(&transaction);

					let (transaction_fee_token, _) = Self::get_fee_token_info(&from);

					let calculated_max_gas = match gas_price.checked_mul(transaction_data.gas_limit)
					{
						Some(v) => v,
						_ => {
							return Err(TransactionValidityError::Invalid(
								InvalidTransaction::Custom(1),
							))
						}
					};

					Self::ensure_sponsor_balance(
						meta_trx_sponsor.clone(),
						transaction_fee_token,
						calculated_max_gas,
					)
					.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;

					return sp_runtime::transaction_validity::ValidTransactionBuilder::default()
						.and_provides((from, transaction_data.nonce))
						.priority(stbl_tools::misc::truncate_u256_to_u64(gas_price))
						.build();
				}
				_ => Err(TransactionValidityError::Unknown(
					UnknownTransaction::Custom(0),
				)),
			}
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
		pub fn send_sponsored_transaction(
			_origin: OriginFor<T>,
			transaction: pallet_ethereum::Transaction,
			meta_trx_sponsor: H160,
			meta_trx_sponsor_signature: Vec<u8>,
		) -> DispatchResult {
			SponsorNonce::<T>::mutate(meta_trx_sponsor.clone(), |nonce| {
				*nonce = nonce.saturating_add(1)
			});

			let from = Self::ensure_transaction_signature(transaction.clone())
				.map_err(|_| DispatchError::Other("Invalid transaction signature"))?;

			Self::ensure_meta_transaction_sponsor(
				transaction.clone(),
				meta_trx_sponsor,
				meta_trx_sponsor_signature,
			)
			.map_err(|_| DispatchError::Other("Invalid metatransaction signature"))?;

			Self::block_ensure_transaction_unicity(&from, &transaction)
				.map_err(|_| DispatchError::Other("Transaction object is invalid"))?;

			let (gas_limit, gas_price) = Self::get_transaction_gas_info(&transaction);

			let (transaction_fee_token, conversion_rate) = Self::get_fee_token_info(&from);

			let calculated_gas_usage = match gas_limit.checked_mul(gas_price.into()) {
				Some(a) => a,
				None => return Err(DispatchError::Other("Arithmetic error due to overflow.")),
			};

			Self::transfer_fee_token(
				&transaction_fee_token,
				conversion_rate,
				&meta_trx_sponsor,
				&from,
				calculated_gas_usage,
			)
			.map_err(|_| DispatchError::Other("Failed to borrow fee token"))?;

			let origin: T::RuntimeOrigin =
				pallet_ethereum::Origin::EthereumTransaction(from).into();
			let dispatch = pallet_ethereum::Pallet::<T>::transact(origin, transaction)
				.map_err(|_| DispatchError::Other("Signature doesn't meet with sponsor address"))?;

			let gas_used = Self::gas_from_actual_weight(dispatch.actual_weight.unwrap())
				.map_err(|_| DispatchError::Other("Arithmetic error due to overflow."))?;

			let gas_left = match gas_limit.checked_sub(gas_used.into()) {
				Some(v) => v,
				None => 0.into(),
			};

			let refunding_amount = match gas_left.checked_mul(gas_price.into()) {
				Some(amount) => amount,
				None => return Err(DispatchError::Other("Arithmetic error due to overflow.")),
			};

			Self::transfer_fee_token(
				&transaction_fee_token,
				conversion_rate,
				&from,
				&meta_trx_sponsor,
				refunding_amount,
			)
			.map_err(|_| DispatchError::Other("Failed to refund fee token"))?;

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn ensure_transaction_signature(
			transaction: pallet_ethereum::Transaction,
		) -> Result<H160, ()> {
			match stbl_tools::eth::recover_signer(&transaction) {
				None => Err(()),
				Some(address) => Ok(address),
			}
		}

		fn ensure_meta_transaction_sponsor(
			transaction: pallet_ethereum::Transaction,
			expected_sponsor: H160,
			meta_trx_sponsor_signature: Vec<u8>,
		) -> Result<(), ()> {
			let meta_trx_internal_message: Vec<u8> =
				Self::get_meta_transaction_signing_message(transaction.clone());

			let eip191_message =
				stbl_tools::eth::build_eip191_message_hash(meta_trx_internal_message);

			let meta_trx_signed_address = Self::get_meta_trx_signer(
				meta_trx_sponsor_signature.clone(),
				eip191_message.clone(),
			);

			match meta_trx_signed_address {
				Some(address) if address == expected_sponsor => Ok(()),
				_ => Err(()),
			}
		}

		fn block_ensure_transaction_unicity(
			origin: &H160,
			transaction: &pallet_ethereum::Transaction,
		) -> Result<(), ()> {
			let transaction_data: TransactionData = transaction.into();

			let (base_fee, _) = <T as pallet_evm::Config>::FeeCalculator::min_gas_price();
			let (who, _) = pallet_evm::Pallet::<T>::account_basic(origin);

			fp_evm::CheckEvmTransaction::<pallet_ethereum::InvalidTransactionWrapper>::new(
				CheckEvmTransactionConfig {
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
			.and_then(|v| v.with_base_fee())
			.map_err(|_| ())?;

			Ok(())
		}

		fn pool_ensure_transaction_unicity(
			origin: &H160,
			transaction: &pallet_ethereum::Transaction,
		) -> Result<(), ()> {
			let transaction_data: TransactionData = transaction.into();

			let (base_fee, _) = <T as pallet_evm::Config>::FeeCalculator::min_gas_price();
			let (who, _) = pallet_evm::Pallet::<T>::account_basic(origin);

			fp_evm::CheckEvmTransaction::<pallet_ethereum::InvalidTransactionWrapper>::new(
				CheckEvmTransactionConfig {
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
			.and_then(|v| v.with_base_fee())
			.map_err(|_| ())?;

			Ok(())
		}

		fn get_fee_token_info(from: &H160) -> (H160, (U256, U256)) {
			let transaction_fee_token =
				T::DNTFeeController::get_transaction_fee_token(from.clone());
			let validator = <pallet_evm::Pallet<T>>::find_author();
			let validator_conversion_rate = T::DNTFeeController::get_transaction_conversion_rate(
				from.clone(),
				validator,
				transaction_fee_token,
			);

			(transaction_fee_token, validator_conversion_rate)
		}

		fn get_transaction_gas_info(transaction: &pallet_ethereum::Transaction) -> (U256, U256) {
			let transaction_data: TransactionData = transaction.into();
			let base_fee = <T as pallet_evm::Config>::FeeCalculator::min_gas_price().0;
			let gas_price = stbl_tools::eth::transaction_gas_price(base_fee, transaction, true);

			if transaction_data.input.len() == 0 {
				(runner::TRANSFER_GAS_LIMIT.into(), gas_price)
			} else {
				(transaction_data.gas_limit, gas_price)
			}
		}

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

		fn ensure_sponsor_balance(sponsor: H160, token: H160, amount: U256) -> Result<(), ()> {
			if amount.is_zero() {
				return Ok(());
			}

			let balance = T::ERC20Manager::balance_of(token.clone(), sponsor.clone());
			if balance >= amount {
				Ok(())
			} else {
				Err(())
			}
		}

		fn transfer_fee_token(
			token: &H160,
			conversion_rate: (U256, U256),
			payer: &H160,
			payee: &H160,
			amount: U256,
		) -> Result<(), ()> {
			if amount.is_zero() {
				return Ok(());
			}
			if conversion_rate.1 == U256::zero() {
				return Err(());
			}

			let actual_amount = amount
				.saturating_mul(conversion_rate.0)
				.div_mod(conversion_rate.1)
				.0;
			T::ERC20Manager::withdraw_amount(token.clone(), payer.clone(), actual_amount)
				.map_err(|_| {})?;

			T::ERC20Manager::deposit_amount(token.clone(), payee.clone(), actual_amount)
				.map_err(|_| {})?;

			Ok(())
		}

		pub fn get_meta_transaction_signing_message(trx: pallet_ethereum::Transaction) -> Vec<u8> {
			let mut message: Vec<u8> = Vec::new();

			let trx_hash = trx.hash();
			let trx_bytes_hash = trx_hash.as_bytes();
			let trx_hash_string = hex::encode(trx_bytes_hash);

			message.extend_from_slice(b"I consent to be a sponsor of transaction: 0x");
			message.extend_from_slice(trx_hash_string.as_bytes());

			return message;
		}

		fn get_meta_trx_signer(signature: Vec<u8>, message: H256) -> Option<H160> {
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
