#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_core::{H160, U256};
use sp_std::prelude::*;

use frame_support::parameter_types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_evm::Runner;
	use runner::RunnerExt;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransaction,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//#[pallet::weight(0)]
		/// add transaction data to TransactionsByNonce and increment nonce, then return nonce
		
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn execute_delegated_transaction(
			delegate: OriginFor<T>,
			delegator: H160, 
			to: H160,
			input: Vec<u8>,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(delegate)?;

			let exc_info = T::Runner::delegated_call(
				delegate,
				to,
				delegator,
				input,
				0,
				gas_limit,
				None,
				None,
				None,
				Vec::new(),
				false,
				false,
				&pallet_evm::EvmConfig::istanbul()
			)?;

			OK(())
		}
	}
}
