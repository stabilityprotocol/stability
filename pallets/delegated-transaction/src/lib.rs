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

	#[pallet::storage]
	#[pallet::getter(fn nonce_by_account_storage)]
	pub type NonceByAccountStorage<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		H160,
		u64,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn sample_storage)]
	pub type SampleStorage<T: Config> = StorageValue<_, u64, OptionQuery>;


	impl<T: Config> DelegatedTransaction for Pallet<T> {
		type Error = Error<T>;

		fn get_delegated_transaction_current_nonce() -> u64 {
			let dummy_nonce: u64 = 128;

			dummy_nonce
		}

		fn get_delegated_transaction_echo(num: u64) -> u64 {
			num
		}

		fn get_delegated_transaction_compare(num: u64) -> (u64,bool) {
			/* let current_value = SampleStorage::<T>::get().unwrap_or_default();

			let comparison = current_value > num; */

			SampleStorage::<T>::put(num);

			(num, false)
		}
	}
}

pub trait DelegatedTransaction {
	type Error;

	fn get_delegated_transaction_current_nonce() -> u64;

	fn get_delegated_transaction_echo(num: u64) -> u64;

	fn get_delegated_transaction_compare(num: u64) -> (u64, bool);
}