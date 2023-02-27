#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::H160;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::traits::Hooks;
	use frame_support::{
		storage::types::{OptionQuery, StorageMap, StorageValue},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::BlockNumberFor;
	use pallet_supported_tokens_manager::SupportedTokensManager;
	use sp_core::H160;
	use sp_std::vec::Vec;

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

	#[pallet::storage]
	#[pallet::getter(fn pending_token_updates)]
	pub type PendingTokenUpdatesStorage<T: Config> =
		StorageValue<_, Vec<(H160, H160)>, OptionQuery>;

	impl<T: Config> UserFeeTokenController for Pallet<T> {
		type Error = Error<T>;

		fn get_user_fee_token(account: H160) -> H160 {
			FeeTokenStorage::<T>::get(account)
				.unwrap_or(T::SupportedTokensManager::get_default_token())
		}

		fn set_user_fee_token(account: H160, token: H160) -> Result<(), Self::Error> {
			if !T::SupportedTokensManager::is_supported_token(token) {
				return Err(Error::<T>::UnsupportedToken);
			}
			let mut pending_updates = PendingTokenUpdatesStorage::<T>::get().unwrap_or_default();
			pending_updates.push((account, token));
			PendingTokenUpdatesStorage::<T>::put(pending_updates);
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			let pending_updates = PendingTokenUpdatesStorage::<T>::get().unwrap_or_default();
			for (account, token) in pending_updates {
				FeeTokenStorage::<T>::insert(account, token);
			}
			PendingTokenUpdatesStorage::<T>::kill();
		}
	}
}

pub trait UserFeeTokenController {
	type Error;
	fn get_user_fee_token(account: H160) -> H160;
	fn set_user_fee_token(account: H160, token: H160) -> Result<(), Self::Error>;
}
