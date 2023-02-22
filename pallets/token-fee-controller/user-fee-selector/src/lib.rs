#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::H160;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{
		storage::types::{StorageMap, ValueQuery},
		Blake2_128Concat,
	};
	use sp_core::{Get, H160};
	use pallet_supported_tokens_manager::SupportedTokensManager;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		UnsupportedToken,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type DefaultFeeToken: Get<H160>;
		type SupportedTokensManager: SupportedTokensManager;
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
		ValueQuery,
		T::DefaultFeeToken,
	>;

	impl<T: Config> UserFeeTokenController for Pallet<T> {

		type Error = Error<T>;

		fn get_user_fee_token(account: H160) -> H160 {
			FeeTokenStorage::<T>::get(account)
		}

		fn set_user_fee_token(account: H160, token: H160) -> Result<(), Self::Error> {
			if !T::SupportedTokensManager::is_supported_token(token) {
				return Err(Error::<T>::UnsupportedToken);
			} else {
				FeeTokenStorage::<T>::insert(account, token);
				Ok(())
			}
		}
	}
}

pub trait UserFeeTokenController {
	type Error;
	fn get_user_fee_token(account: H160) -> H160;
	fn set_user_fee_token(account: H160, token: H160) -> Result<(), Self::Error>;
}
