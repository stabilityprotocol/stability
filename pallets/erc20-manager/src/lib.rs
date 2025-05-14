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

pub use pallet::*;

use sp_core::{H160, U256};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait ERC20Manager {
	type Error;
	fn balance_of(token: H160, payer: H160) -> U256;
	fn withdraw_amount(token: H160, payer: H160, amount: U256) -> Result<U256, Self::Error>;
	fn deposit_amount(token: H160, payee: H160, amount: U256) -> Result<U256, Self::Error>;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use pallet_supported_tokens_manager::SupportedTokensManager;
	use sp_core::{H160, H256, U256};
	use stbl_tools::some_or_err;

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

		fn balance_of(token: H160, user: H160) -> U256 {
			let slot = Self::get_address_balance_storage_slot(token, user);

			let balance = pallet_evm::AccountStorages::<T>::get(token, slot);
			U256::from_big_endian(balance.as_bytes())
		}

		fn withdraw_amount(token: H160, payer: H160, amount: U256) -> Result<U256, Error<T>> {
			if amount.is_zero() {
				return Ok(U256::from(0));
			};

			let slot = Self::get_address_balance_storage_slot(token, payer);

			pallet_evm::AccountStorages::<T>::try_mutate(&token, &slot, |stored_value| {
				let current_balance = U256::from_big_endian(stored_value.as_bytes());
				let new_balance = some_or_err!(current_balance.checked_sub(amount), || {
					Error::<T>::UnderflowBalance
				});

				let mut new_balance_bytes: [u8; 32] = [0; 32];
				new_balance.to_big_endian(&mut new_balance_bytes);

				*stored_value = H256::from(new_balance_bytes);

				Ok(amount)
			})?;

			Ok(amount)
		}

		fn deposit_amount(token: H160, payee: H160, amount: U256) -> Result<U256, Self::Error> {
			if amount.is_zero() {
				return Ok(U256::from(0));
			};

			let slot = Self::get_address_balance_storage_slot(token, payee);

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
		fn get_address_balance_storage_slot(token: H160, address: H160) -> H256 {
			let balance_slot = T::SupportedTokensManager::get_token_balance_slot(token)
				.unwrap_or(H256::from_low_u64_be(0));

			stbl_tools::eth::get_storage_address_for_mapping(address, balance_slot)
		}
	}
}
