#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_core::{H160, U256};
use sp_std::prelude::*;

use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::parameter_types;
use frame_support::dispatch::Pays;
use stbl_tools::{some_or_err, none_or_err};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::{OriginFor, *};
	use pallet_evm::{EnsureAddressOrigin, Runner};
	use pallet_user_fee_selector::UserFeeTokenController;
	use runner::{OnChargeDecentralizedNativeTokenFee, Runner as RunnerImpl, RunnerExt};
	use sp_runtime::traits::AccountIdConversion;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransaction,
		NotSigned,
		DelegateIsNotOrigin,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		// type UserFeeTokenController: UserFeeTokenController;
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
			origin: OriginFor<T>,
			delegate: H160,
			delegator: H160,
			to: H160,
			input: Vec<u8>,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// none_or_err!(who.into(), |_| { Error::<T>::NotSigned });

			let is_origin = T::CallOrigin::ensure_address_origin(&delegate, origin)?;
			// none_or_err!(is_origin, |_| { Error::<T>::DelegateIsNotOrigin });

			let exc_info = RunnerImpl::call_delegated(
				delegator,
				delegate,
				to,
				input,
				U256::from(0),
				gas_limit,
				None,
				None,
				None,
				Vec::new(),
				false,
				false,
				&pallet_evm::EvmConfig::istanbul(),
			)?;

			Ok(().into())
		}
	}
}
