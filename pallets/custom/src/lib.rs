#![cfg_attr(not(feature = "std"), no_std)]



pub use pallet::*;

use frame_support::dispatch::Pays;
use frame_system::pallet_prelude::OriginFor;
use frame_system::ensure_signed;



#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::{*, ValueQuery};


	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn memory_)]
	pub type MemoryNumber<T: Config> = StorageValue<_, u64, ValueQuery>;


	#[pallet::config]
	pub trait Config: frame_system::Config {
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn set_memory(
			origin: OriginFor<T>,
			number: u64
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			MemoryNumber::<T>::set(number);
			Ok(Pays::No.into())
		}
	}
}


