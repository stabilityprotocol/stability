#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::{H160, H256};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;

	use frame_support::{
		storage::types::{OptionQuery, StorageMap, StorageValue, ValueQuery},
		Blake2_128Concat,
	};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		DefaultTokenNotSupported,
		DefaultTokenCannotBeRemoved,
		AlreadySupportedToken,
		TokenNotSupported,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::storage]
	pub type SupportedTokens<T: Config> = StorageValue<_, Vec<H160>, ValueQuery>;

	#[pallet::storage]
	pub type TokenBalanceSlot<T: Config> = StorageMap<_, Blake2_128Concat, H160, H256, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn default_token)]
	pub type DefaultTokenStorage<T: Config> = StorageValue<_, H160, OptionQuery>;

	impl<T: Config> SupportedTokensManager for Pallet<T> {
		type Error = Error<T>;

		fn get_supported_tokens() -> Vec<H160> {
			SupportedTokens::<T>::get()
		}

		fn add_supported_token(token: H160, slot: H256) -> Result<(), Self::Error> {
			if Self::is_supported_token(token) {
				return Err(Error::<T>::AlreadySupportedToken);
			}
			let mut tokens = SupportedTokens::<T>::get();
			tokens.push(token);
			TokenBalanceSlot::<T>::insert(token, slot);
			SupportedTokens::<T>::put(tokens);
			Ok(())
		}

		fn is_supported_token(token: H160) -> bool {
			let tokens = SupportedTokens::<T>::get();
			tokens.contains(&token)
		}

		fn remove_supported_token(token: H160) -> Result<(), Self::Error> {
			match token {
				token if token.eq(&Self::get_default_token()) => {
					Err(Error::<T>::DefaultTokenCannotBeRemoved)
				}
				token if !Self::is_supported_token(token) => Err(Error::<T>::TokenNotSupported),
				token => {
					let mut tokens = SupportedTokens::<T>::get();
					tokens.retain(|t| token.ne(t));
					TokenBalanceSlot::<T>::remove(token);
					SupportedTokens::<T>::put(tokens);
					Ok(())
				}
			}
		}

		fn get_token_balance_slot(token: H160) -> Option<H256> {
			TokenBalanceSlot::<T>::get(token)
		}

		fn get_default_token() -> H160 {
			DefaultTokenStorage::<T>::get().unwrap_or(H160::zero())
		}

		fn set_default_token(token: H160) -> Result<(), Self::Error> {
			if SupportedTokens::<T>::get().contains(&token) {
				DefaultTokenStorage::<T>::put(token);
				Ok(())
			} else {
				Err(Error::<T>::DefaultTokenNotSupported)
			}
		}
	}

	#[pallet::genesis_config]
	#[cfg(feature = "std")]
	pub struct GenesisConfig {
		pub initial_default_token: H160,
		pub initial_default_token_slot: H256,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				initial_default_token: H160::zero(),
				initial_default_token_slot: H256::zero(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			DefaultTokenStorage::<T>::put(self.initial_default_token);
			SupportedTokens::<T>::put(vec![self.initial_default_token]);
			TokenBalanceSlot::<T>::insert(
				self.initial_default_token,
				self.initial_default_token_slot,
			);
		}
	}
}

pub trait SupportedTokensManager {
	type Error;

	fn get_supported_tokens() -> Vec<H160>;

	fn add_supported_token(token: H160, slot: H256) -> Result<(), Self::Error>;

	fn is_supported_token(token: H160) -> bool;

	fn remove_supported_token(token: H160) -> Result<(), Self::Error>;

	fn get_token_balance_slot(token: H160) -> Option<H256>;

	fn get_default_token() -> H160;

	fn set_default_token(token: H160) -> Result<(), Self::Error>;
}
