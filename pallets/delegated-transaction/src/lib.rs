#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use sp_core::{H160, U256};
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use pallet_evm::Runner;
	use sp_core::Encode;

	use frame_support::pallet_prelude::{*, ValueQuery};
	use frame_support::{
		storage::types::{StorageMap, StorageValue},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;
	// use sp_runtime::print;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransaction,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//#[pallet::weight(0)]
		/// add transaction data to TransactionsByNonce and increment nonce, then return nonce
		
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn add_transaction(
			origin: OriginFor<T>,
			to: H160,
			input: Vec<u8>,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
		) -> DispatchResultWithPostInfo {
			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn execute_delegated(origin: OriginFor<T>, nonce: u64) -> DispatchResultWithPostInfo {
			Ok(Pays::No.into())
		}
	}
}
