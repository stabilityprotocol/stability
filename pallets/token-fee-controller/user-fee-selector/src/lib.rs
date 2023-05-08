#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
use sp_core::{H160, U256};

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{
		storage::types::{OptionQuery, StorageMap},
		Blake2_128Concat,
	};
	use pallet_erc20_manager::ERC20Manager;
	use pallet_supported_tokens_manager::SupportedTokensManager;
	use sp_core::H160;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		UnsupportedToken,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type SupportedTokensManager: SupportedTokensManager;
		type ERC20Manager: ERC20Manager;
	}

	#[pallet::storage]
	#[pallet::getter(fn fee_token_storage)]
	pub type FeeTokenStorage<T: Config> = StorageMap<
		_,
		// User
		Blake2_128Concat,
		H160,
		// Fee Token
		H160,
		OptionQuery,
	>;

	impl<T: Config> UserFeeTokenController for Pallet<T> {
		type Error = Error<T>;

		fn get_user_fee_token(account: H160) -> H160 {
			FeeTokenStorage::<T>::get(account).unwrap_or(H160::zero())
			/* let user_token = FeeTokenStorage::<T>::get(account)
				.unwrap_or(T::SupportedTokensManager::get_default_token());

			T::SupportedTokensManager::is_supported_token(user_token)
				.then(|| user_token)
				.unwrap_or(T::SupportedTokensManager::get_default_token()) */
		}

		fn set_user_fee_token(account: H160, token: H160) -> Result<(), Self::Error> {
			/* if !T::SupportedTokensManager::is_supported_token(token) {
				return Err(Error::<T>::UnsupportedToken);
			} */
			FeeTokenStorage::<T>::insert(account, token);
			Ok(())
		}

		fn balance_of(account: H160) -> U256 {
			let token = Self::get_user_fee_token(account);
			T::ERC20Manager::balance_of(token, account)
		}
	}
}

pub trait UserFeeTokenController {
	type Error;
	fn get_user_fee_token(account: H160) -> H160;
	fn set_user_fee_token(account: H160, token: H160) -> Result<(), Self::Error>;
	fn balance_of(account: H160) -> U256;
}

#[cfg(test)]
impl UserFeeTokenController for () {
	type Error = ();
	fn get_user_fee_token(_account: H160) -> H160 {
		Default::default()
	}
	fn set_user_fee_token(_account: H160, _token: H160) -> Result<(), Self::Error> {
		Ok(())
	}
	fn balance_of(_account: H160) -> U256 {
		Default::default()
	}
}
