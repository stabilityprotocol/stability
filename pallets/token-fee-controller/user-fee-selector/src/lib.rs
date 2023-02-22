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

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type DefaultFeeToken: Get<H160>;
	}

	#[pallet::storage]
	#[pallet::getter(fn fee_token_storage)]
	pub type FeeTokenStorage<T: Config> = StorageMap<
		_,
		// Owner
		Blake2_128Concat,
		H160,
		// Nonce
		H160,
		ValueQuery,
		T::DefaultFeeToken,
	>;

	impl<T: Config> UserFeeTokenController for Pallet<T> {
		fn get_user_fee_token(account: H160) -> H160 {
			FeeTokenStorage::<T>::get(account)
		}

		fn set_user_fee_token(account: H160, token: H160) {
			FeeTokenStorage::<T>::insert(account, token);
		}
	}
}

pub trait UserFeeTokenController {
	fn get_user_fee_token(account: H160) -> H160;
	fn set_user_fee_token(account: H160, token: H160);
}
