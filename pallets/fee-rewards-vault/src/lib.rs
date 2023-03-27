#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_core::{H160, U256};
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config  + pallet_evm::Config {
	}

	// double map
	#[pallet::storage]
	#[pallet::getter(fn claimable_reward)]
	pub(super) type ClaimableReward<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		H160,
		Twox64Concat,
		H160,
		U256,
		ValueQuery,
	>;

	// map
	#[pallet::storage]
	#[pallet::getter(fn whitelist)]
	pub(super) type Whitelist<T: Config> = StorageMap<_, Twox64Concat, H160, bool, ValueQuery>;


	impl<T: Config> Pallet<T> {
		pub fn is_whitelisted(address: H160) -> bool {
			Self::whitelist(address)
		}

		pub fn set_whitelist(address: H160, is_whitelisted: bool) {
			Whitelist::<T>::insert(address, is_whitelisted);
		}

		pub fn get_claimable_reward(address: H160, token: H160) -> U256 {
			Self::claimable_reward(address, token)
		}

		pub fn add_claimable_reward(address: H160, token: H160, amount: U256) {
			let current_amount = Self::claimable_reward(address, token);
			ClaimableReward::<T>::insert(address, token, current_amount + amount);
		}
		
		pub fn sub_claimable_reward(address: H160, token: H160, amount: U256) {
			let current_amount = Self::claimable_reward(address, token);
			ClaimableReward::<T>::insert(address, token, current_amount - amount);
		}
	}



}