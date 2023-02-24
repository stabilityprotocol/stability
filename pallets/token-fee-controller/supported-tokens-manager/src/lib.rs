#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::vec::Vec;
use sp_core::{Get, H160, H256};

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{
		storage::types::{OptionQuery, StorageMap, StorageValue, ValueQuery},
		Blake2_128Concat,
	};
	

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
	pub type SupportedTokens<T: Config> =
		StorageValue<_, Vec<H160>, ValueQuery, T::InitialSupportedTokens>;

	#[pallet::storage]
	pub type TokenBalanceSlot<T: Config> = StorageMap<_, Blake2_128Concat, H160, H256, OptionQuery>;

	impl<T: Config> SupportedTokensManager for Pallet<T> {
		fn get_supported_tokens() -> Vec<H160> {
			SupportedTokens::<T>::get()
		}

		fn add_supported_token(token: H160, slot: H256) {
			let mut tokens = SupportedTokens::<T>::get();
			tokens.push(token);
			TokenBalanceSlot::<T>::insert(token, slot);
			SupportedTokens::<T>::put(tokens);
		}

		fn is_supported_token(token: H160) -> bool {
			let tokens = SupportedTokens::<T>::get();
			tokens.contains(&token)
		}

		fn remove_supported_token(token: H160) {
			let mut tokens = SupportedTokens::<T>::get();
			tokens.retain(|t| token.eq(t));
			TokenBalanceSlot::<T>::remove(token);
			SupportedTokens::<T>::put(tokens);
		}

		fn get_token_balance_slot(token: H160) -> Option<H256> {
			TokenBalanceSlot::<T>::get(token)
		}
	}
}

pub trait SupportedTokensManager {
	fn get_supported_tokens() -> Vec<H160>;

	fn add_supported_token(token: H160, slot: H256);

	fn is_supported_token(token: H160) -> bool;

	fn remove_supported_token(token: H160);

	fn get_token_balance_slot(token: H160) -> Option<H256>;
}
