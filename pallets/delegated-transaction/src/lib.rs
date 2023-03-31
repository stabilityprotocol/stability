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

	#[derive(Encode, Decode, Default, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct PendingTransaction<T: Config> {
		pub sender: T::AccountId,
		pub to: H160,
		pub input: Vec<u8>,
	}

	#[pallet::storage]
	#[pallet::getter(fn transaction_nonce)]
	pub type TransactionNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn transactions_by_nonce)]
	pub type TransactionsByNonce<T: Config> =
		StorageMap<_, Blake2_128Concat, str, PendingTransaction<T>, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransaction,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub linked_accounts: Vec<(T::AccountId, H160)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				linked_accounts: Vec::new(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_linked_accounts(&self.linked_accounts);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//#[pallet::weight(0)]
		/// add transaction data to TransactionsByNonce and increment nonce, then return nonce
		
		#[pallet::call_index(0)]
		#[pallet::weight(Pays::No)]
		pub fn add_transaction(
			origin: OriginFor<T>,
			to: H160,
			input: Vec<u8>,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let tx_nonce = TransactionNonce::<T>::try_get()?;

			let temporary_transaction = PendingTransaction { sender, to, input };

			let validation_response = T::Runner::validate(
				temporary_transaction.sender.clone(), // source
				temporary_transaction.to.clone(),     // target
				temporary_transaction.input.clone(),  // input
				Default::default(),                   // _value
				u64::MAX,                             // _gas_limit
				None,                                 // max_fee_per_gas
				None,                                 // max_priority_fee_per_gas
				None,                                 // nonce
				Default::default(),                   // _access_list
				false,                                // is_transactional
				&pallet_evm::EvmConfig::istanbul(),   // evm_config
			)?;
			ensure!(validation_response.is_valid, Error::<T>::InvalidTransaction);

			TransactionsByNonce::<T>::insert(String::from(tx_nonce), temporary_transaction);

			TransactionNonce::<T>::mutate(|nonce| {
				*nonce += 1;
			});

			Ok(tx_nonce)
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn execute_delegated(origin: OriginFor<T>, nonce: u64) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let transaction = TransactionsByNonce::<T>::try_get(String::from(nonce))?;

			let response = T::Runner::call(
				transaction.sender.clone(),         // source
				transaction.to.clone(),             // target
				transaction.input.clone(),          // input
				Default::default(),                 // _value
				u64::MAX,                           // _gas_limit
				None,                               // max_fee_per_gas
				None,                               // max_priority_fee_per_gas
				None,                               // nonce
				Default::default(),                 // _access_list
				false,                              // is_transactional
				&pallet_evm::EvmConfig::istanbul(), // evm_config
			)?;

			TransactionsByNonce::<T>::try_remove(nonce.clone())?;

			Ok(nonce)
		}
	}
}
