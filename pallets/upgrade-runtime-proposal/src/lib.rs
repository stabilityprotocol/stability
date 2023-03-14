#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::traits::EnsureOrigin;
use frame_system::pallet_prelude::OriginFor;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::{*}, BoundedVec, storage::unhashed::get};

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
	#[pallet::getter(fn code_of)]
	pub type ProposedCode<T: Config> = StorageValue<_, BoundedVec<u8, T::MaxSizeOfCode>, ValueQuery>;

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
			
			if code_saved.is_empty() {
				return Err(Error::<T>::ProposalInProgress.into());
			}

			<ProposedCode<T>>::try_mutate::<(), Error<T>, _>(|code_saved|  {
				*code_saved =  BoundedVec::<u8, T::MaxSizeOfCode>::try_from(code).map_err(|_| Error::<T>::FailedToSaveCode)?;
				Ok(())
			})?;
	
			Ok(Pays::No.into())
		}
	}

}