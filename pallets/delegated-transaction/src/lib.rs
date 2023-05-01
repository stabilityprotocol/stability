#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_core::{H160, U256};



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
	pub enum Error<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {}


	impl<T: Config> DelegatedTransaction for Pallet<T> {
		type Error = Error<T>;

		fn get_delegated_transaction_current_nonce() -> u64 {
			let nonce: u64 = 167;
			
			nonce
		}
	}
}

pub trait DelegatedTransaction {
	type Error;

	fn get_delegated_transaction_current_nonce() -> u64;
}