#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_core::{H160, U256};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait ERC20Manager {
	type Error;
	fn withdraw_amount(token: &H160, payer: &H160, fee: U256) -> Result<U256, Self::Error>;
	fn deposit_amount(token: &H160, payee: &H160, amount: U256) -> Result<U256, Self::Error>;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use pallet_supported_tokens_manager::SupportedTokensManager;
	use sha3::Digest;
	use sp_core::Encode;
	use sp_core::{H160, H256, U256};
	use stbl_tools::{map_err, some_or_err};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		UnderflowBalance,
		OverflowBalance,
		FailedTokenConfiguration,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type SupportedTokensManager: SupportedTokensManager;
	}

	impl<T: Config> ERC20Manager for Pallet<T> {
		type Error = Error<T>;

		fn withdraw_amount(token: &H160, payer: &H160, fee: U256) -> Result<U256, Error<T>> {
			if fee.is_zero() {
				return Ok(U256::from(0));
			};

			let slot = map_err!(Self::get_address_balance_storage_slot(token, payer), |_| {
				Error::<T>::FailedTokenConfiguration
			});

			pallet_evm::AccountStorages::<T>::try_mutate(&token, &slot, |stored_value| {
				let current_balance = U256::from_big_endian(stored_value.as_bytes());
				let new_balance = some_or_err!(current_balance.checked_sub(fee), || {
					Error::<T>::UnderflowBalance
				});

				let mut new_balance_bytes: [u8; 32] = [0; 32];
				new_balance.to_big_endian(&mut new_balance_bytes);

				*stored_value = H256::from(new_balance_bytes);

				Ok(fee)
			})?;

			Ok(fee)
		}

		fn deposit_amount(token: &H160, payee: &H160, amount: U256) -> Result<U256, Self::Error> {
			if amount.is_zero() {
				return Ok(U256::from(0));
			};

			let slot = map_err!(Self::get_address_balance_storage_slot(token, payee), |_| {
				Error::<T>::FailedTokenConfiguration
			});

			pallet_evm::AccountStorages::<T>::try_mutate(&token, &slot, |stored_value| {
				let current_balance = U256::from_big_endian(stored_value.as_bytes());
				let new_balance = some_or_err!(current_balance.checked_add(amount), || {
					Error::<T>::OverflowBalance
				});

				let mut new_balance_bytes: [u8; 32] = [0; 32];
				new_balance.to_big_endian(&mut new_balance_bytes);

				*stored_value = H256::from(new_balance_bytes);

				Ok(())
			})?;

			Ok(amount)
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_address_balance_storage_slot(token: &H160, address: &H160) -> Result<H256, ()> {
			let balance_slot = T::SupportedTokensManager::get_token_balance_slot(*token)
				.unwrap_or(H256::from_low_u64_be(0));

			let u256_address = H256::from(*address);
			let address_bytes = u256_address.as_bytes();
			let balance_slot_bytes = balance_slot.as_bytes();

			let input = &[&address_bytes[..], &balance_slot_bytes[..]].concat();

			Ok(sha3::Keccak256::new()
				.chain_update(input)
				.finalize()
				.using_encoded(|x| H256::from_slice(&x[1..])))
		}
	}
}
