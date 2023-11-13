#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::dispatch::UnfilteredDispatchable;
use frame_support::traits::EnsureOrigin;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::pallet_prelude::OriginFor;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, BoundedVec};
	use frame_system::ensure_root;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type ControlOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type MaxSizeOfCode: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn proposed_code)]
	pub type ProposedCode<T: Config> =
		StorageValue<_, BoundedVec<u8, T::MaxSizeOfCode>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn application_block_number)]
	pub type ApplicationBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, OptionQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// The address received is invalid
		ProposalInProgress,
		/// Fail when trying to save the code
		FailedToSaveCode,
		/// Must be a proposed code to set the block number to apply
		NoProposedCode,
		/// The block number to apply the code must be greater than the current block number
		BlockNumberMustBeGreaterThanCurrentBlockNumber,
		// Invalid code
		InvalidCode,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn propose_code(origin: OriginFor<T>, code: Vec<u8>) -> DispatchResultWithPostInfo {
			T::ControlOrigin::ensure_origin(origin)?;

			stbl_tools::none_or_err!(<ProposedCode<T>>::get(), |_| {
				Error::<T>::ProposalInProgress.into()
			});

			frame_system::Pallet::<T>::can_set_code(&code).map_err(|_| Error::<T>::InvalidCode)?;

			<ProposedCode<T>>::try_mutate::<(), Error<T>, _>(|code_saved| {
				*code_saved = Some(
					BoundedVec::<u8, T::MaxSizeOfCode>::try_from(code)
						.map_err(|_| Error::<T>::FailedToSaveCode)?,
				);
				Ok(())
			})?;

			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		pub fn set_block_application(
			origin: OriginFor<T>,
			block_number: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let code_saved = <ProposedCode<T>>::get();

			stbl_tools::some_or_err!(code_saved, || { Error::<T>::NoProposedCode.into() });

			let current_block = frame_system::Pallet::<T>::block_number();

			if current_block >= block_number {
				return Err(Error::<T>::BlockNumberMustBeGreaterThanCurrentBlockNumber.into());
			}

			<ApplicationBlockNumber<T>>::put(block_number);

			Ok(Pays::No.into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn reject_proposed_code(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			stbl_tools::some_or_err!(<ProposedCode<T>>::get(), || {
				Error::<T>::NoProposedCode.into()
			});

			<ProposedCode<T>>::kill();
			<ApplicationBlockNumber<T>>::kill();

			Ok(Pays::No.into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let code_saved_option = <ProposedCode<T>>::get();
			let application_block_number_option = <ApplicationBlockNumber<T>>::get();

			if code_saved_option.is_none() {
				return Weight::zero();
			}
			let code = code_saved_option.unwrap().to_vec();

			if application_block_number_option.is_none() {
				return Weight::zero();
			}
			let application_block_number = application_block_number_option.unwrap();

			if n != application_block_number {
				return Weight::zero();
			}

			let call = frame_system::Call::<T>::set_code { code: code.into() };

			let result = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());

			if result.is_err() {
				log::error!("Failed to upgrade runtime");
			} else {
				log::info!("Runtime upgraded");
			}

			Pallet::<T>::clear_proposed_code();
			Pallet::<T>::clear_application_block_number();

			Weight::zero()
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_proposed_code() -> Option<Vec<u8>> {
			<ProposedCode<T>>::get().map(|code| code.to_vec())
		}

		pub fn get_application_block_number() -> Option<T::BlockNumber> {
			<ApplicationBlockNumber<T>>::get()
		}

		pub fn set_application_block_number(block_number: T::BlockNumber) {
			<ApplicationBlockNumber<T>>::put(block_number);
		}

		pub fn clear_proposed_code() {
			<ProposedCode<T>>::kill();
		}

		pub fn clear_application_block_number() {
			<ApplicationBlockNumber<T>>::kill();
		}
	}
}
