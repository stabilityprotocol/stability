#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::H160;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use pallet_supported_tokens_manager::SupportedTokensManager;
	use sp_core::H160;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type SupportedTokensManager: SupportedTokensManager;
	}

	impl<T: Config> RPCTokens for Pallet<T> {
		type Error = Error<T>;

		fn get_supported_tokens() -> Vec<H160> {
			T::SupportedTokensManager::get_supported_tokens()
		}
	}
}

pub trait RPCTokens {
	type Error;
	fn get_supported_tokens() -> Vec<H160>;
}
