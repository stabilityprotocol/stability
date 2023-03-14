#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_std::prelude::*;


use frame_support::traits::EnsureOrigin;
use frame_system::pallet_prelude::OriginFor;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_support::dispatch::UnfilteredDispatchable;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::{*}, BoundedVec};

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
	pub type ProposedCode<T: Config> = StorageValue<_, BoundedVec<u8, T::MaxSizeOfCode>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn application_block_number)]
	pub type ApplicationBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// The address received is invalid
		ProposalInProgress,
		/// Fail when trying to save the code
		FailedToSaveCode,

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn propose_code(
			origin: OriginFor<T>,
			code: Vec<u8>
		) -> DispatchResultWithPostInfo {
			T::ControlOrigin::ensure_origin(origin)?;

			let code_saved =  <ProposedCode<T>>::get();
			
			if !code_saved.is_empty() {
				return Err(Error::<T>::ProposalInProgress.into());
			}

			<ProposedCode<T>>::try_mutate::<(), Error<T>, _>(|code_saved|  {
				*code_saved =  BoundedVec::<u8, T::MaxSizeOfCode>::try_from(code).map_err(|_| Error::<T>::FailedToSaveCode)?;
				Ok(())
			})?;
	
			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn set_block_application(
			origin: OriginFor<T>,
			block_number: T::BlockNumber
		) -> DispatchResultWithPostInfo {
			T::ControlOrigin::ensure_origin(origin)?;

			<ApplicationBlockNumber<T>>::put(block_number);

			Ok(Pays::No.into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let code_saved =  <ProposedCode<T>>::get();
			if !code_saved.is_empty() {
				let application_block_number = <ApplicationBlockNumber<T>>::get();
				if n == application_block_number {
					let code = code_saved.to_vec();

					Pallet::<T>::clear_proposed_code();
					Pallet::<T>::clear_application_block_number();


					let call = frame_system::Call::<T>::set_code {
						code: code.into()
					};
					
					let result = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());

					if result.is_err() {
						log::error!("Failed to upgrade runtime");
					}
					else {
						log::info!("Runtime upgraded");
					}

				}
			}
			Weight::zero()
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_proposed_code() -> Vec<u8> {
			<ProposedCode<T>>::get().to_vec()
		}

		pub fn get_application_block_number() -> T::BlockNumber {
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