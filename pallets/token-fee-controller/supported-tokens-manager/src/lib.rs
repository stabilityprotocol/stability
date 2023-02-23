#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::H160;
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{
		storage::types::{StorageValue, ValueQuery},
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
		type InitialSupportedTokens: Get<Vec<H160>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn fee_token_storage)]
	pub type SupportedTokens<T: Config> =
		StorageValue<_, Vec<H160>, ValueQuery, T::InitialSupportedTokens>;

	pub trait SupportedTokensManager {
		fn get_supported_tokens() -> Vec<H160>;

		fn add_supported_token(token: H160);

		fn is_supported_token(token: H160) -> bool;

		fn remove_supported_token(token: H160);
	}

	impl<T: Config> SupportedTokensManager for Pallet<T> {
		fn get_supported_tokens() -> Vec<H160> {
			SupportedTokens::<T>::get()
		}

		fn add_supported_token(token: H160) {
			let mut tokens = SupportedTokens::<T>::get();
			tokens.push(token);
			SupportedTokens::<T>::put(tokens);
		}

		fn is_supported_token(token: H160) -> bool {
			let tokens = SupportedTokens::<T>::get();
			tokens.contains(&token)
		}

		fn remove_supported_token(token: H160) {
			let mut tokens = SupportedTokens::<T>::get();
			tokens.retain(|t| token.eq(t));
			SupportedTokens::<T>::put(tokens);
		}
	}
}
