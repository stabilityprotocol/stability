#![cfg_attr(not(feature = "std"), no_std)]



use codec::Encode;
pub use pallet::*;

use frame_support::dispatch::Pays;
use frame_system::pallet_prelude::OriginFor;
use frame_system::ensure_signed;
use sp_core::H256;
use sp_io::hashing::keccak_256;



pub mod migrations {
    use super::*;

    use frame_support::pallet_prelude::*;
    use frame_support::storage_alias;
	use frame_support::dispatch::GetStorageVersion;
	use log::log;
	

    #[storage_alias]
    type MemoryNumber<T: Config> = StorageValue<Pallet<T>, u64, ValueQuery>;

    pub fn migrate_to_v2<T: Config>() -> frame_support::weights::Weight {
		let current = Pallet::<T>::current_storage_version();
		let on_chain = Pallet::<T>::on_chain_storage_version();

		if current == 2 && on_chain == StorageVersion::default() {
			current.put::<Pallet<T>>();
			let number = MemoryNumber::<T>::get();
			let hash = number_to_hash(number);

			MemoryHash::<T>::put(hash);
			frame_support::weights::Weight::default()
		}
		else {
			log!(log::Level::Info, "Skipping v13, should be removed");
			frame_support::weights::Weight::default()
		}
    }
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::{*, ValueQuery}};
	use sp_core::{H256};


	const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn memory_string)]
	pub type MemoryHash<T: Config> = StorageValue<_, H256, ValueQuery>;


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

			MemoryHash::<T>::set(number_to_hash(number));
			Ok(Pays::No.into())
		}
	}
}



fn number_to_hash(number: u64) -> H256 {
	let hash = keccak_256(number.encode().as_slice());

	hash.into()
}


